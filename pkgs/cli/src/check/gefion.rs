use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use super::clusters::{ClusterId, GEFION, host_block, teleport_cert, teleport_key, tsh_login};
use super::env::{CheckEnv, block_hint, cli_there, expander, reached};
use super::probes::connect::provider_context;
use super::probes::net::reach_timeout;
use super::probes::ssh::{SshConfig, is_configured, port_of};
use super::probes::tsh::{
    TshVersion, cert_validity, duration, left_ms, profile_users, tsh_home, tsh_version,
};
use super::types::{Group, Outcome, Status};
use crate::paths::{join, tilde};
use crate::session::sync::VERSION_PROBE;

const HOUR: i64 = 3_600_000;

static TSH_PROXY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\btsh\b.*\bproxy\s+ssh\b").unwrap());

pub async fn check_gefion(g: &mut Group, c: &CheckEnv<'_>) {
    let alias = c.alias(ClusterId::Gefion).await.unwrap_or_else(|| GEFION.alias.to_string());
    let want_user = c.config.gefion.as_ref().and_then(|s| s.user.clone());
    let home = &c.ctx.paths.home;
    let tsh = tsh_home(&c.ctx.env, home);
    let guess_user =
        want_user.clone().or_else(|| profile_users(&tsh, GEFION.proxy).into_iter().next());

    g.step("tsh", "Teleport client (tsh)", async {
        Ok(match tsh_version(&c.ctx.env).await {
            TshVersion::Missing => Outcome::fail("not installed").hint(vec![
                "Install tsh: https://goteleport.com/docs/connect-your-client/tsh/".into(),
            ]),
            TshVersion::Unknown => Outcome::ok("installed"),
            TshVersion::Known { text, .. } => Outcome::ok(format!("tsh {text}")),
        })
    })
    .await;

    if !c.machine.ssh_usable {
        g.skip("host", "Host in ~/.ssh/config", "needs a working ssh").await;
        return;
    }

    let mut cfg: Option<SshConfig> = None;
    let host = g
        .step("host", "Host in ~/.ssh/config", async {
            let conf = match c.ssh_g(&alias).await {
                Err(error) => return Ok(Outcome::fail(error)),
                Ok(conf) => conf,
            };
            let block = host_block(ClusterId::Gefion, &alias, guess_user.as_deref());
            if !is_configured(&conf, &alias) {
                return Ok(Outcome::fail(format!("none points at {}", GEFION.login)).hint(
                    block_hint(
                        "Add this to ~/.ssh/config. The Host name is yours to choose:",
                        &block,
                    ),
                ));
            }
            let mut problems: Vec<String> = Vec::new();
            let hostname = conf.get("hostname").unwrap_or_default().to_string();
            let port = port_of(&conf);
            if hostname != GEFION.login {
                problems.push(format!("HostName is {hostname}"));
            }
            if port != GEFION.port {
                problems.push(format!("Port is {port}, not {}", GEFION.port));
            }
            if !TSH_PROXY.is_match(conf.get("proxycommand").unwrap_or_default()) {
                problems.push("no tsh ProxyCommand".into());
            }
            cfg = Some(conf);
            if !problems.is_empty() {
                return Ok(Outcome::fail(problems.join("; "))
                    .hint(block_hint(&format!("Compare Host {alias} with:"), &block)));
            }
            Ok(Outcome::ok(format!("{alias} → {hostname}:{port} through Teleport")))
        })
        .await;
    let Some(conf) = cfg.filter(|_| host.status != Status::Fail) else { return };
    let user = conf.get("user").unwrap_or_default().to_string();
    let expand = expander(c, Some(&user), conf.get("hostname"));
    let cert = teleport_cert(&tsh, &user);
    let key = teleport_key(&tsh, &user);
    let known_hosts = join(&[&tsh, "known_hosts"]);

    g.step("paths", &format!("Teleport files in Host {alias}"), async {
        if user.contains('<') || user.contains('>') {
            return Ok(Outcome::fail(format!("User is still the placeholder {user}"))
                .hint(vec![format!("Set User to your Gefion username in Host {alias}.")]));
        }
        if let Some(want) = want_user.as_deref()
            && user != want
        {
            return Ok(Outcome::fail(format!("User is {user}, but config.toml says {want}")));
        }
        let mut problems: Vec<String> = Vec::new();
        if !conf.all("certificatefile").iter().any(|p| expand(p) == cert) {
            problems.push(format!("CertificateFile is not {}", tilde(&cert, home)));
        }
        if !conf.all("identityfile").iter().any(|p| expand(p) == key) {
            problems.push(format!("IdentityFile is not {}", tilde(&key, home)));
        }
        if !conf.words("userknownhostsfile").iter().any(|p| expand(p) == known_hosts) {
            problems.push(format!("UserKnownHostsFile is not {}", tilde(&known_hosts, home)));
        }
        if !problems.is_empty() {
            return Ok(Outcome::fail(problems.join("; ")).hint(block_hint(
                &format!("Compare Host {alias} with:"),
                &host_block(ClusterId::Gefion, &alias, Some(&user)),
            )));
        }
        Ok(Outcome::ok(format!("for {user}")))
    })
    .await;

    let named = (!user.is_empty()).then_some(user.as_str());
    let mut logged_in = false;
    g.step("login", "Teleport login", async {
        let fix = vec![tsh_login(named), "A login lasts about 12 hours.".into()];
        if !Path::new(&cert).exists() {
            return Ok(Outcome::fail("no certificate yet").hint(fix));
        }
        let Some(validity) = cert_validity(&cert, &c.ctx.env).await else {
            return Ok(Outcome::warn(format!("cannot read {}", tilde(&cert, home))).hint(fix));
        };
        let left = left_ms(&validity);
        if let Some(left) = left.filter(|l| *l <= 0) {
            return Ok(Outcome::fail(format!("expired {} ago", duration(left))).hint(fix));
        }
        logged_in = true;
        Ok(match left {
            Some(left) if left < HOUR => {
                Outcome::warn(format!("expires in {}", duration(left))).hint(vec![tsh_login(named)])
            }
            Some(left) => Outcome::ok(format!("valid for {}", duration(left))),
            None => Outcome::ok("valid"),
        })
    })
    .await;

    let reach = g
        .step("reach", &format!("Network path to {}", GEFION.proxy), async {
            let r =
                (c.ctx.tcp)(GEFION.proxy.into(), GEFION.proxy_port, reach_timeout(c.connect)).await;
            Ok(c.reach(
                r,
                GEFION.proxy_port,
                "Teleport needs outbound HTTPS. Check that this machine is online.",
            ))
        })
        .await;

    if !c.connect {
        return;
    }

    let mut there: Option<Option<String>> = None;
    if reach.status != Status::Ok {
        g.skip("ssh", "Login", "needs the network path").await;
    } else if !logged_in {
        g.skip("ssh", "Login", "needs a valid Teleport login").await;
    } else {
        g.step("ssh", "Login", async {
            let script = format!(r#"printf "@@HOST %s\n" "$(hostname)"; {VERSION_PROBE}"#);
            let context = provider_context(ClusterId::Gefion, &alias, Some(&user), None);
            Ok(match c.login(&script, &context).await? {
                Err(outcome) => outcome,
                Ok(tags) => {
                    there = Some(tags.get("AICENTRE").cloned());
                    reached(&tags)
                }
            })
        })
        .await;
        if let Some(there) = there {
            let version = c.ctx.version.clone();
            g.step(
                "cli",
                "aicentre there",
                async move { Ok(cli_there(&version, there.as_deref())) },
            )
            .await;
        }
    }
}
