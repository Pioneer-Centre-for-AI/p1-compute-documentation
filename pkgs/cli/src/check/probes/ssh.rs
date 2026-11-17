use std::collections::HashMap;
use std::sync::LazyLock;

use regex::{Captures, Regex};

use crate::context::Env;
use crate::exec::{RunOptions, run};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshVersion {
    pub major: u32,
    pub minor: u32,
    pub text: String,
}

static OPENSSH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"OpenSSH_(?:for_Windows_)?([0-9]+)\.([0-9]+)").unwrap());

pub fn parse_ssh_version(output: &str) -> Option<SshVersion> {
    let m = OPENSSH.captures(output)?;
    Some(SshVersion {
        major: m[1].parse().unwrap_or(u32::MAX),
        minor: m[2].parse().unwrap_or(u32::MAX),
        text: format!("{}.{}", &m[1], &m[2]),
    })
}

pub fn at_least(v: &SshVersion, major: u32, minor: u32) -> bool {
    v.major > major || (v.major == major && v.minor >= minor)
}

pub async fn ssh_version(env: &Env) -> Option<SshVersion> {
    let r = run("ssh", &["-V"], RunOptions::new(env).timeout_ms(5_000)).await;
    if r.missing {
        return None;
    }
    parse_ssh_version(&format!("{}{}", r.stderr, r.stdout))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SshConfig(HashMap<String, Vec<String>>);

impl SshConfig {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.first()).map(String::as_str)
    }

    pub fn all(&self, key: &str) -> &[String] {
        self.0.get(key).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn words(&self, key: &str) -> Vec<String> {
        self.all(key).iter().flat_map(|v| v.split_whitespace()).map(str::to_string).collect()
    }
}

pub fn parse_ssh_g(output: &str) -> SshConfig {
    let mut cfg: HashMap<String, Vec<String>> = HashMap::new();
    for line in output.split('\n') {
        let Some(i) = line.find(' ').filter(|i| *i > 0) else { continue };
        let key = line[..i].to_lowercase();
        let value = line[i + 1..].trim().to_string();
        cfg.entry(key).or_default().push(value);
    }
    SshConfig(cfg)
}

pub fn port_of(cfg: &SshConfig) -> u16 {
    cfg.get("port").map_or(22, |p| p.parse().unwrap_or(0))
}

pub type SshGResult = Result<SshConfig, String>;

pub async fn ssh_g(alias: &str, env: &Env) -> SshGResult {
    let r = run("ssh", &["-G", alias], RunOptions::new(env).timeout_ms(5_000)).await;
    if r.missing {
        return Err("ssh is not installed".into());
    }
    if r.code != 0 {
        let lines: Vec<&str> =
            r.stderr.trim().split('\n').filter(|l| !l.is_empty()).take(3).collect();
        return Err(if lines.is_empty() {
            format!("ssh -G exited {}", r.code)
        } else {
            lines.join("\n")
        });
    }
    Ok(parse_ssh_g(&r.stdout))
}

pub fn is_configured(cfg: &SshConfig, alias: &str) -> bool {
    cfg.get("hostname").is_some_and(|h| h != alias.to_lowercase())
}

#[derive(Clone, Debug, Default)]
pub struct Tokens {
    pub home: String,
    pub local_user: String,
    pub remote_user: Option<String>,
    pub host: Option<String>,
    pub uid: Option<u32>,
}

static TOKEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%([%dhilru])").unwrap());

pub fn expand_path(path: &str, t: &Tokens) -> String {
    let p = if let Some(rest) = path.strip_prefix("~/") {
        format!("{}/{rest}", t.home)
    } else if path == "~" {
        t.home.clone()
    } else {
        path.to_string()
    };
    TOKEN
        .replace_all(&p, |c: &Captures| {
            let whole = c[0].to_string();
            match &c[1] {
                "%" => "%".to_string(),
                "d" => t.home.clone(),
                "u" => t.local_user.clone(),
                "r" => t.remote_user.clone().unwrap_or(whole),
                "h" => t.host.clone().unwrap_or(whole),
                "i" => t.uid.map(|u| u.to_string()).unwrap_or(whole),
                _ => whole,
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_g_output_and_unknown_hosts() {
        let cfg = parse_ssh_g(
            "host dtu\nhostname dtu\nidentityfile ~/.ssh/id_rsa\nidentityfile ~/.ssh/id_ed25519\nuserknownhostsfile /a /b\n",
        );
        assert_eq!(cfg.all("identityfile"), ["~/.ssh/id_rsa", "~/.ssh/id_ed25519"]);
        assert_eq!(cfg.words("userknownhostsfile"), ["/a", "/b"]);
        assert!(!is_configured(&cfg, "dtu"));
        assert!(is_configured(&parse_ssh_g("hostname login9.hpc.dtu.dk\n"), "dtu"));
    }
}
