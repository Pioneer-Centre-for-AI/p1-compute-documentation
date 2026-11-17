use std::fs;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};

use anyhow::{Result, bail};

use crate::context::Env;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    pub home: String,
    pub bin_dir: String,
    pub bin_link: String,
    pub data: String,
    pub versions: String,
    pub current: String,
    pub manifest: String,
    pub config: String,
    pub config_file: String,
    pub state: String,
    pub cache: String,
    pub runtime: String,
}

pub fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
}

pub fn join(parts: &[&str]) -> String {
    let joined = parts.iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join("/");
    if joined.is_empty() {
        return ".".into();
    }
    normalize(&joined)
}

pub fn normalize(path: &str) -> String {
    if path.is_empty() {
        return ".".into();
    }
    let absolute = path.starts_with('/');
    let trailing = path.ends_with('/');
    let mut out: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if out.last().is_some_and(|p| *p != "..") {
                    out.pop();
                } else if !absolute {
                    out.push("..");
                }
            }
            p => out.push(p),
        }
    }
    let mut s = out.join("/");
    if s.is_empty() && !absolute {
        s = ".".into();
    }
    if trailing && !s.is_empty() && s != "." {
        s.push('/');
    } else if trailing && s == "." {
        s = "./".into();
    }
    if absolute { format!("/{s}") } else { s }
}

pub fn dirname(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return if path.starts_with('/') { "/".into() } else { ".".into() };
    }
    match trimmed.rfind('/') {
        Some(0) => "/".into(),
        Some(i) => trimmed[..i].trim_end_matches('/').to_string(),
        None => ".".into(),
    }
}

pub fn basename(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    trimmed.rsplit('/').next().unwrap_or("").to_string()
}

fn xdg(env: &Env, name: &str, fallback: String) -> String {
    match env.get(name) {
        Some(v) if is_absolute(v) => v.clone(),
        _ => fallback,
    }
}

pub fn resolve_paths(env: &Env, uid: u32) -> Result<Paths> {
    let home = match env.get("HOME") {
        Some(h) if is_absolute(h) => h.clone(),
        _ => bail!("HOME is not set to an absolute path"),
    };
    let data = join(&[&xdg(env, "XDG_DATA_HOME", join(&[&home, ".local/share"])), "aicentre"]);
    let config = join(&[&xdg(env, "XDG_CONFIG_HOME", join(&[&home, ".config"])), "aicentre"]);
    let bin_dir = xdg(env, "XDG_BIN_HOME", join(&[&home, ".local/bin"]));
    let runtime = match env.get("XDG_RUNTIME_DIR") {
        Some(base) if is_absolute(base) => join(&[base, "aicentre"]),
        _ => join(&[&xdg(env, "TMPDIR", "/tmp".into()), &format!("aicentre-{uid}")]),
    };
    Ok(Paths {
        bin_link: join(&[&bin_dir, "aicentre"]),
        versions: join(&[&data, "versions"]),
        current: join(&[&data, "current"]),
        manifest: join(&[&data, "install.json"]),
        config_file: join(&[&config, "config.toml"]),
        state: join(&[&xdg(env, "XDG_STATE_HOME", join(&[&home, ".local/state"])), "aicentre"]),
        cache: join(&[&xdg(env, "XDG_CACHE_HOME", join(&[&home, ".cache"])), "aicentre"]),
        home,
        bin_dir,
        data,
        config,
        runtime,
    })
}

fn mkdir_private(dir: &str) -> std::io::Result<()> {
    fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)
}

pub fn state_dir(paths: &Paths, parts: &[&str]) -> Result<String> {
    mkdir_private(&paths.state)?;
    let mode = fs::metadata(&paths.state)?.mode();
    if mode & 0o077 != 0 {
        fs::set_permissions(&paths.state, fs::Permissions::from_mode(0o700))?;
    }
    let mut all = vec![paths.state.as_str()];
    all.extend_from_slice(parts);
    let dir = join(&all);
    mkdir_private(&dir)?;
    Ok(dir)
}

pub fn ensure_runtime_dir(paths: &Paths, uid: u32) -> Result<String> {
    mkdir_private(&paths.runtime)?;
    let st = fs::symlink_metadata(&paths.runtime)?;
    if st.file_type().is_symlink() || !st.is_dir() {
        bail!("{} is not a directory", paths.runtime);
    }
    if st.uid() != uid {
        bail!("{} belongs to another user", paths.runtime);
    }
    if st.mode() & 0o077 != 0 {
        bail!("{} is open to other users (mode {:o})", paths.runtime, st.mode() & 0o777);
    }
    Ok(paths.runtime.clone())
}

pub fn tilde(path: &str, home: &str) -> String {
    if path == home {
        "~".into()
    } else if let Some(rest) = path.strip_prefix(home).filter(|r| r.starts_with('/')) {
        format!("~{rest}")
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> Env {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn xdg_paths_only_when_absolute() {
        let p = resolve_paths(
            &env(&[("HOME", "/h"), ("XDG_DATA_HOME", "rel"), ("XDG_CONFIG_HOME", "/c")]),
            1000,
        )
        .unwrap();
        assert_eq!(p.data, "/h/.local/share/aicentre");
        assert_eq!(p.config_file, "/c/aicentre/config.toml");
        assert_eq!(p.runtime, "/tmp/aicentre-1000");
        assert_eq!(p.bin_link, "/h/.local/bin/aicentre");
        assert!(resolve_paths(&env(&[("HOME", "h")]), 1).is_err());
    }
}
