use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use super::ssh::{SshVersion, at_least};
use crate::check::clusters::{ClusterId, DTU, DTU_NO_ANSWER_HINT, GEFION, tsh_login};
use crate::check::types::Status;
use crate::context::Env;
use crate::exec::{RunOptions, run};

pub fn probe_args(
    alias: &str,
    version: Option<&SshVersion>,
    script: &str,
    socket: Option<&str>,
) -> Vec<String> {
    let mut opts = vec![
        "BatchMode=yes",
        "ConnectTimeout=10",
        "ConnectionAttempts=1",
        "ControlMaster=no",
        "RequestTTY=no",
        "ClearAllForwardings=yes",
    ];
    if version.is_some_and(|v| at_least(v, 7, 6)) {
        opts.push("RemoteCommand=none");
    }
    let mut args: Vec<String> = Vec::new();
    for o in opts {
        args.push("-o".into());
        args.push(o.into());
    }
    if let Some(socket) = socket {
        args.push("-S".into());
        args.push(socket.into());
    }
    args.extend(["-T".into(), "-n".into(), alias.into(), format!("sh -c '{script}'")]);
    args
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub kind: &'static str,
    pub status: Status,
    pub message: String,
    pub hint: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureContext {
    pub alias: String,
    pub host: String,
    pub via_tsh: bool,
    pub timed_out: bool,
    pub timeout_hint: Vec<String>,
    pub teleport_hint: Vec<String>,
}

pub fn provider_context(
    provider: ClusterId,
    alias: &str,
    user: Option<&str>,
    host: Option<&str>,
) -> FailureContext {
    let gefion = provider == ClusterId::Gefion;
    FailureContext {
        alias: alias.to_string(),
        host: host.unwrap_or(if gefion { GEFION.login } else { DTU.login }).to_string(),
        via_tsh: gefion,
        timed_out: false,
        timeout_hint: if gefion {
            vec!["Teleport answered, but the login node did not. Try again in a minute, or ask DCAI support (support@dcai.dk).".into()]
        } else {
            vec![DTU_NO_ANSWER_HINT.into()]
        },
        teleport_hint: if gefion {
            vec![tsh_login(user.filter(|u| !u.is_empty()))]
        } else {
            vec![]
        },
    }
}

enum Test {
    Re(Regex),
    Password,
}

impl Test {
    fn matches(&self, text: &str) -> bool {
        match self {
            Test::Re(re) => re.is_match(text),
            Test::Password => asks_password(text),
        }
    }
}

struct Rule {
    kind: &'static str,
    test: Test,
    status: Status,
    message: &'static str,
    hint: fn(&FailureContext) -> Vec<String>,
}

fn re(pattern: &str) -> Test {
    Test::Re(Regex::new(&format!("(?i){pattern}")).unwrap())
}

// JavaScript's `.`
const DOT: &str = r"[^\n\r\u{2028}\u{2029}]";

static PERMISSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Permission denied \(").unwrap());

fn asks_password(text: &str) -> bool {
    PERMISSION.find_iter(text).any(|m| {
        let rest = &text[m.end()..];
        let Some(close) = rest.find(')') else { return false };
        let methods = rest[..close].to_lowercase();
        !methods.contains("publickey")
            && (methods.contains("password") || methods.contains("keyboard-interactive"))
    })
}

fn s(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|l| l.to_string()).collect()
}

static RULES: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    vec![
        Rule {
            kind: "teleport",
            test: re(&format!(
                r#"ERROR:{DOT}*(expired|not logged in|access denied)|exec: "?tsh"?: not found|tsh: (command )?not found|Connection closed by UNKNOWN port 65535"#
            )),
            status: Status::Fail,
            message: "Teleport refused the connection",
            hint: |c| c.teleport_hint.clone(),
        },
        Rule {
            kind: "dns",
            test: re(
                "Could not resolve hostname|Name or service not known|nodename nor servname provided|Temporary failure in name resolution",
            ),
            status: Status::Fail,
            message: "the host name does not resolve",
            hint: |c| {
                vec![format!(
                    "Check the HostName in Host {}, and that this machine is online.",
                    c.alias
                )]
            },
        },
        Rule {
            kind: "timeout",
            test: re("(Connection|Operation) timed out|timed out during banner exchange"),
            status: Status::Fail,
            message: "the login node did not answer",
            hint: |c| c.timeout_hint.clone(),
        },
        Rule {
            kind: "refused",
            test: re("Connection refused|No route to host|Network is unreachable"),
            status: Status::Fail,
            message: "the connection was refused",
            hint: |c| c.timeout_hint.clone(),
        },
        Rule {
            kind: "reset",
            test: re(
                "kex_exchange_identification|Connection reset by peer|Connection closed by remote host",
            ),
            status: Status::Fail,
            message: "the server closed the connection before login",
            hint: |c| c.timeout_hint.clone(),
        },
        Rule {
            kind: "host-key-changed",
            test: re(&format!(
                "REMOTE HOST IDENTIFICATION HAS CHANGED|Host key for {DOT}* has changed"
            )),
            status: Status::Fail,
            message: "the host key changed since you last connected",
            hint: |c| {
                vec![
                    "If the provider announced a new key, remove the old one and connect again:"
                        .into(),
                    format!("  ssh-keygen -R {}", c.host),
                    "If it did not, stop and ask the compute coordinator.".into(),
                ]
            },
        },
        Rule {
            kind: "host-key-unknown",
            test: re("Host key verification failed"),
            status: Status::Warn,
            message: "this machine has not seen the host key yet",
            hint: |c| vec![format!("Connect once by hand and answer yes: ssh {}", c.alias)],
        },
        Rule {
            kind: "unprotected-key",
            test: re("UNPROTECTED PRIVATE KEY FILE|bad permissions"),
            status: Status::Fail,
            message: "ssh ignored a key because others can read it",
            hint: |_| s(&["chmod 600 the key file that ssh named, then try again."]),
        },
        Rule {
            kind: "too-many-keys",
            test: re("Too many authentication failures"),
            status: Status::Fail,
            message: "ssh offered too many keys before the right one",
            hint: |c| {
                vec![format!(
                    "Add \"IdentitiesOnly yes\" and the right IdentityFile to Host {}.",
                    c.alias
                )]
            },
        },
        Rule {
            kind: "password",
            test: Test::Password,
            status: Status::Fail,
            message: "your key was accepted, and the login node also asks for your password",
            hint: |_| {
                s(&[
                    "Off DTU's network, login9 wants your DTU password after the key. On DTU's network or the DTU VPN, the key is enough.",
                    "In a terminal, aicentre asks for it once and keeps the login open for 30 min.",
                    "aicentre list jobs, or any session, does that.",
                ])
            },
        },
        Rule {
            kind: "key-refused",
            test: re(r"Permission denied \([^)]*\)"),
            status: Status::Fail,
            message: "the login node did not accept your key",
            hint: |c| {
                if c.via_tsh {
                    return c.teleport_hint.clone();
                }
                vec![
                    format!(
                        "Check User in Host {}, then copy your public key to the login node:",
                        c.alias
                    ),
                    format!("  ssh-copy-id {}", c.alias),
                    "If your key has a passphrase, load it into the agent first: ssh-add".into(),
                ]
            },
        },
    ]
});

pub fn classify_ssh_failure(stderr: &str, c: &FailureContext) -> Failure {
    if c.timed_out {
        return Failure {
            kind: "timeout",
            status: Status::Fail,
            message: "ssh gave up after 25 s".into(),
            hint: if c.via_tsh { c.teleport_hint.clone() } else { c.timeout_hint.clone() },
        };
    }
    for rule in RULES.iter() {
        if rule.kind == "teleport" && !c.via_tsh {
            continue;
        }
        if rule.test.matches(stderr) {
            return Failure {
                kind: rule.kind,
                status: rule.status,
                message: rule.message.into(),
                hint: (rule.hint)(c),
            };
        }
    }
    let last = stderr.trim().split('\n').rfind(|l| !l.is_empty());
    Failure {
        kind: "other",
        status: Status::Fail,
        message: last.unwrap_or("ssh failed without saying why").to_string(),
        hint: vec![format!("Run ssh -v {} to see where it stops.", c.alias)],
    }
}

pub type Tags = HashMap<String, String>;

static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^@@([A-Z0-9]+) ?(.*)$").unwrap());

pub fn parse_tags(stdout: &str) -> Tags {
    let mut tags = Tags::new();
    for line in stdout.split('\n') {
        if let Some(m) = TAG.captures(line.trim()) {
            tags.insert(m[1].to_string(), m[2].to_string());
        }
    }
    tags
}

pub async fn ssh_probe(
    version: Option<&SshVersion>,
    script: &str,
    c: &FailureContext,
    env: &Env,
    socket: Option<&str>,
) -> Result<Tags, Failure> {
    let args = probe_args(&c.alias, version, script, socket);
    let r = run("ssh", &args, RunOptions::new(env).timeout_ms(25_000).detached()).await;
    let tags = parse_tags(&r.stdout);
    if r.code == 0 && tags.contains_key("HOST") {
        return Ok(tags);
    }
    let c = FailureContext { timed_out: r.timed_out, ..c.clone() };
    Err(classify_ssh_failure(&r.stderr, &c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_failures_are_classified_from_stderr() {
        let c = FailureContext {
            alias: "dtu".into(),
            host: "login9.hpc.dtu.dk".into(),
            via_tsh: false,
            timed_out: false,
            timeout_hint: vec!["VPN".into()],
            teleport_hint: vec!["tsh login".into()],
        };
        let tsh = FailureContext { via_tsh: true, ..c.clone() };
        let late = FailureContext { timed_out: true, ..c.clone() };
        let cases: [(&str, &str, &FailureContext); 11] = [
            (
                "ssh: Could not resolve hostname login9.hpc.dtu.dk: Name or service not known",
                "dns",
                &c,
            ),
            ("ssh: connect to host x port 22: Connection timed out", "timeout", &c),
            ("ssh: connect to host x port 22: Connection refused", "refused", &c),
            (
                "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!\nHost key verification failed.",
                "host-key-changed",
                &c,
            ),
            ("Host key verification failed.", "host-key-unknown", &c),
            ("s1@x: Permission denied (publickey,gssapi-keyex).", "key-refused", &c),
            ("s1@x: Permission denied (keyboard-interactive).", "password", &c),
            (
                "WARNING: UNPROTECTED PRIVATE KEY FILE!\nPermissions 0644 for '/h/.ssh/id' are too open.",
                "unprotected-key",
                &c,
            ),
            ("ERROR: Active profile expired.", "teleport", &tsh),
            ("", "timeout", &late),
            ("something new", "other", &c),
        ];
        for (stderr, kind, ctx) in cases {
            assert_eq!(classify_ssh_failure(stderr, ctx).kind, kind, "{stderr}");
        }
        assert_ne!(classify_ssh_failure("ERROR: Active profile expired.", &c).kind, "teleport");
        assert_eq!(
            classify_ssh_failure("x: Permission denied (publickey,password).", &c).kind,
            "key-refused"
        );
        assert_eq!(classify_ssh_failure("something new\nlast words\n", &c).message, "last words");
    }
}
