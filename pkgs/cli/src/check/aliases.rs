use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

use regex::Regex;
use tokio::sync::OnceCell;

use super::clusters::{ClusterId, DTU, GEFION};
use super::probes::ssh::{SshGResult, is_configured, port_of, ssh_g};
use crate::config::Config;
use crate::context::Ctx;
use crate::paths::{basename, dirname, is_absolute, join};

pub struct Target {
    pub host: &'static str,
    pub port: Option<u16>,
}

pub fn target(id: ClusterId) -> Option<Target> {
    match id {
        ClusterId::Dtu => Some(Target { host: DTU.login, port: None }),
        ClusterId::Gefion => Some(Target { host: GEFION.login, port: Some(GEFION.port) }),
        ClusterId::Ngc => None,
    }
}

fn default_alias(id: ClusterId) -> Option<&'static str> {
    match id {
        ClusterId::Dtu => Some(DTU.alias),
        ClusterId::Gefion => Some(GEFION.alias),
        ClusterId::Ngc => None,
    }
}

static GLOB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[*?\[]").unwrap());
static SPECIAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[.+^${}()|\\]").unwrap());

fn expand_include(pattern: &str, home: &str) -> Vec<String> {
    let mut p = match pattern.strip_prefix("~/") {
        Some(rest) => join(&[home, rest]),
        None => pattern.to_string(),
    };
    if !is_absolute(&p) {
        p = join(&[home, ".ssh", &p]);
    }
    let name = basename(&p);
    if !GLOB.is_match(&name) {
        return if std::path::Path::new(&p).exists() { vec![p] } else { vec![] };
    }
    let escaped = SPECIAL.replace_all(&name, r"\$0").replace('*', ".*").replace('?', ".");
    let Ok(re) = Regex::new(&format!("^{escaped}$")) else { return vec![] };
    let dir = dirname(&p);
    let Ok(entries) = std::fs::read_dir(&dir) else { return vec![] };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|f| re.is_match(f))
        .collect();
    names.sort();
    let mut files = Vec::new();
    for f in names {
        let full = join(&[&dir, &f]);
        match std::fs::metadata(&full) {
            Ok(st) if st.is_file() => files.push(full),
            Ok(_) => {}
            Err(_) => return vec![],
        }
    }
    files
}

static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""([^"]*)"|([^\s=]+)"#).unwrap());
static COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(^|\s)#[^\n\r\u{2028}\u{2029}]*$").unwrap());
static WILD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[*?!]").unwrap());

fn words(line: &str) -> Vec<String> {
    WORD.captures_iter(line)
        .map(|m| m.get(1).or(m.get(2)).map_or("", |x| x.as_str()).to_string())
        .collect()
}

pub fn aliases_for(host: &str, home: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let want = host.to_lowercase();
    scan(&join(&[home, ".ssh", "config"]), 0, home, &want, &mut seen, &mut found);
    found
}

fn scan(
    path: &str,
    depth: usize,
    home: &str,
    want: &str,
    seen: &mut HashSet<String>,
    found: &mut Vec<String>,
) {
    if depth > 16 || !seen.insert(path.to_string()) {
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else { return };
    let mut names: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let stripped = COMMENT.replace(raw, "");
        let line = stripped.trim();
        if line.is_empty() {
            continue;
        }
        let w = words(line);
        let (keyword, args) = match w.split_first() {
            Some((k, a)) => (k.to_lowercase(), a),
            None => (String::new(), &[] as &[String]),
        };
        match keyword.as_str() {
            "host" => names = args.iter().filter(|a| !WILD.is_match(a)).cloned().collect(),
            "match" => names.clear(),
            "hostname" if args.first().is_some_and(|a| a.to_lowercase() == want) => {
                for n in &names {
                    if !found.contains(n) {
                        found.push(n.clone());
                    }
                }
            }
            "include" => {
                for pattern in args {
                    for f in expand_include(pattern, home) {
                        scan(&f, depth + 1, home, want, seen, found);
                    }
                }
            }
            _ => {}
        }
    }
}

fn points_at(res: &SshGResult, target: &Target) -> bool {
    let Ok(cfg) = res else { return false };
    if cfg.get("hostname").map(str::to_lowercase).as_deref() != Some(target.host) {
        return false;
    }
    target.port.is_none_or(|p| port_of(cfg) == p)
}

pub async fn find_alias(id: ClusterId, ctx: &Ctx, config: &Config, r: &Resolver) -> Option<String> {
    if let Some(alias) = config.cluster(id).and_then(|c| c.alias.clone()) {
        return Some(alias);
    }
    let target = target(id)?;
    let fallback = default_alias(id);
    if let Some(f) = fallback
        && points_at(&r.ssh_g(f).await, &target)
    {
        return Some(f.to_string());
    }
    let mut host_only: Option<String> = None;
    for alias in aliases_for(target.host, &ctx.paths.home) {
        let res = r.ssh_g(&alias).await;
        if points_at(&res, &target) {
            return Some(alias);
        }
        if host_only.is_none()
            && let Ok(cfg) = &res
            && cfg.get("hostname").map(str::to_lowercase).as_deref() == Some(target.host)
        {
            host_only = Some(alias);
        }
    }
    if host_only.is_some() {
        return host_only;
    }
    if let Some(f) = fallback
        && let Ok(cfg) = r.ssh_g(f).await
        && is_configured(&cfg, f)
    {
        return Some(f.to_string());
    }
    None
}

struct Inner {
    ctx: Ctx,
    config: Config,
    configs: Mutex<HashMap<String, Arc<OnceCell<SshGResult>>>>,
    aliases: Mutex<HashMap<ClusterId, Arc<OnceCell<Option<String>>>>>,
}

#[derive(Clone)]
pub struct Resolver(Arc<Inner>);

pub fn resolver(ctx: &Ctx, config: &Config) -> Resolver {
    Resolver(Arc::new(Inner {
        ctx: ctx.clone(),
        config: config.clone(),
        configs: Mutex::new(HashMap::new()),
        aliases: Mutex::new(HashMap::new()),
    }))
}

impl Resolver {
    pub async fn ssh_g(&self, alias: &str) -> SshGResult {
        let cell = {
            let mut configs = crate::lock(&self.0.configs);
            configs.entry(alias.to_string()).or_default().clone()
        };
        cell.get_or_init(|| ssh_g(alias, &self.0.ctx.env)).await.clone()
    }

    pub async fn alias(&self, id: ClusterId) -> Option<String> {
        let cell = {
            let mut aliases = crate::lock(&self.0.aliases);
            aliases.entry(id).or_default().clone()
        };
        cell.get_or_init(|| find_alias(id, &self.0.ctx, &self.0.config, self)).await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_names_are_found_by_hostname_through_include_in_file_order() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().to_string_lossy().into_owned();
        std::fs::create_dir_all(format!("{home}/.ssh/config.d")).unwrap();
        std::fs::write(
            format!("{home}/.ssh/config"),
            [
                "Host first",
                "  HostName login9.hpc.dtu.dk",
                "Include ~/.ssh/config.d/* missing/*",
                "Host other",
                "  HostName=login9.hpc.dtu.dk # comment",
            ]
            .join("\n"),
        )
        .unwrap();
        std::fs::write(
            format!("{home}/.ssh/config.d/a"),
            "Host \"gbar.p1\" p1 *.wild !neg\n  hostname LOGIN9.hpc.dtu.dk\nInclude ../config\n",
        )
        .unwrap();
        assert_eq!(aliases_for("login9.hpc.dtu.dk", &home), ["first", "gbar.p1", "p1", "other"]);
        assert!(aliases_for("slurm-login-01.hpc.ite.dcai.dk", &home).is_empty());
    }
}
