use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;

use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::context::Env;
use crate::exec::{RunOptions, run};
use crate::paths::{Paths, dirname, is_absolute, join, normalize};
use crate::platform::Platform;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema: u32,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub os: String,
    #[serde(default)]
    pub arch: String,
    #[serde(default)]
    pub bin_link: String,
    #[serde(default)]
    pub installed_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub created_dirs: Option<Vec<String>>,
}

pub fn read_manifest(paths: &Paths) -> Option<Manifest> {
    let m: Manifest = serde_json::from_str(&fs::read_to_string(&paths.manifest).ok()?).ok()?;
    (m.schema == 1).then_some(m)
}

fn write_atomic(path: &str, text: &str) -> std::io::Result<()> {
    let tmp = format!("{path}.tmp-{}", std::process::id());
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}

/// Like JavaScript's `toISOString`.
pub fn iso_now() -> String {
    jiff::Timestamp::now().strftime("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

pub fn write_manifest(
    paths: &Paths,
    version: &str,
    platform: Platform,
    created: &[String],
) -> anyhow::Result<()> {
    let now = iso_now();
    let previous = read_manifest(paths);
    let outside = |d: &String| *d != paths.data && !d.starts_with(&format!("{}/", paths.data));
    let mut dirs: Vec<String> = Vec::new();
    let earlier = previous.as_ref().and_then(|p| p.created_dirs.clone()).unwrap_or_default();
    for d in earlier.into_iter().chain(created.iter().filter(|d| outside(d)).cloned()) {
        if !dirs.contains(&d) {
            dirs.push(d);
        }
    }
    let manifest = Manifest {
        schema: 1,
        version: version.to_string(),
        os: platform.os.as_str().to_string(),
        arch: platform.arch.as_str().to_string(),
        bin_link: paths.bin_link.clone(),
        installed_at: Some(previous.and_then(|p| p.installed_at).unwrap_or_else(|| now.clone())),
        updated_at: Some(now),
        created_dirs: Some(dirs),
    };
    write_atomic(&paths.manifest, &(serde_json::to_string_pretty(&manifest)? + "\n"))?;
    Ok(())
}

pub fn version_dir(paths: &Paths, version: &str) -> String {
    join(&[&paths.versions, version])
}

/// The directories it created, outermost first.
pub fn mkdirp(path: &str) -> std::io::Result<Vec<String>> {
    let mut created = Vec::new();
    let mut p = path.to_string();
    while !Path::new(&p).exists() {
        created.insert(0, p.clone());
        let parent = dirname(&p);
        if parent == p {
            break;
        }
        p = parent;
    }
    fs::create_dir_all(path)?;
    Ok(created)
}

pub fn remove_if_empty(dirs: &[String]) {
    let mut sorted = dirs.to_vec();
    sorted.sort_by_key(|d| std::cmp::Reverse(d.len()));
    for dir in sorted {
        let _ = fs::remove_dir(dir);
    }
}

pub struct Placed {
    pub path: String,
    pub created: Vec<String>,
}

pub fn place_binary(paths: &Paths, version: &str, source: &str) -> anyhow::Result<Placed> {
    let dir = version_dir(paths, version);
    let created = mkdirp(&dir)?;
    let target = join(&[&dir, "aicentre"]);
    let tmp = format!("{target}.tmp-{}", std::process::id());
    fs::copy(source, &tmp)?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755))?;
    fs::rename(&tmp, &target)?;
    Ok(Placed { path: target, created })
}

pub async fn smoke_test(binary: &str, version: &str, env: &Env) -> Option<String> {
    let r = run(binary, &["--version"], RunOptions::new(env).timeout_ms(10_000)).await;
    let got = r.stdout.trim();
    if r.code != 0 || got != version {
        let shown = if got.is_empty() { r.stderr.trim() } else { got };
        return Some(format!(
            "{binary} --version printed \"{shown}\" (exit {}), expected \"{version}\"",
            r.code
        ));
    }
    None
}

fn remove_file_force(path: &str) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        r => r,
    }
}

fn swap_link(target: &str, link: &str) -> std::io::Result<()> {
    let tmp = format!("{link}.tmp-{}", std::process::id());
    remove_file_force(&tmp)?;
    symlink(target, &tmp)?;
    fs::rename(&tmp, link)
}

pub fn activate(paths: &Paths, version: &str) -> std::io::Result<()> {
    swap_link(&join(&["versions", version]), &paths.current)
}

fn inside(path: &str, root: &str) -> bool {
    path.strip_prefix(root).is_some_and(|rest| rest.starts_with('/'))
}

fn resolve_from(dir: &str, target: &str) -> String {
    let full = if is_absolute(target) { normalize(target) } else { join(&[dir, target]) };
    match full.trim_end_matches('/') {
        "" => "/".into(),
        trimmed => trimmed.to_string(),
    }
}

pub fn is_our_link(link: &str, paths: &Paths) -> bool {
    let Ok(meta) = fs::symlink_metadata(link) else { return false };
    if !meta.file_type().is_symlink() {
        return false;
    }
    let Ok(target) = fs::read_link(link) else { return false };
    let full = resolve_from(&dirname(link), &target.to_string_lossy());
    full == paths.data || inside(&full, &paths.data)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Linked {
    Created,
    Kept,
    Foreign,
}

pub struct LinkResult {
    pub result: Linked,
    pub created: Vec<String>,
}

pub fn link_bin(paths: &Paths) -> std::io::Result<LinkResult> {
    let target = join(&[&paths.current, "aicentre"]);
    let exists = fs::symlink_metadata(&paths.bin_link).is_ok();
    if exists && !is_our_link(&paths.bin_link, paths) {
        return Ok(LinkResult { result: Linked::Foreign, created: vec![] });
    }
    if exists && fs::read_link(&paths.bin_link)?.to_string_lossy() == target {
        return Ok(LinkResult { result: Linked::Kept, created: vec![] });
    }
    let created = mkdirp(&paths.bin_dir)?;
    swap_link(&target, &paths.bin_link)?;
    Ok(LinkResult { result: Linked::Created, created })
}

pub fn safe_remove(path: &str, paths: &Paths) -> anyhow::Result<()> {
    let roots = [&paths.data, &paths.config, &paths.state, &paths.cache, &paths.runtime];
    let within = [&paths.data, &paths.cache].iter().any(|root| inside(path, root));
    if !is_absolute(path) || !(roots.iter().any(|r| *r == path) || within) {
        bail!("refusing to remove {path}: aicentre does not own it");
    }
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(path)?,
        Ok(_) => fs::remove_file(path)?,
    }
    Ok(())
}

pub fn prune_versions(paths: &Paths, keep: &str) -> anyhow::Result<Vec<String>> {
    let Ok(entries) = fs::read_dir(&paths.versions) else { return Ok(vec![]) };
    let mut names: Vec<String> =
        entries.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    let mut removed = Vec::new();
    for name in names {
        if name == keep {
            continue;
        }
        safe_remove(&join(&[&paths.versions, &name]), paths)?;
        removed.push(name);
    }
    Ok(removed)
}

pub fn remove_link(path: &str) -> std::io::Result<()> {
    fs::remove_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::resolve_paths;

    fn sandbox() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().canonicalize().unwrap().to_string_lossy().into_owned();
        let env: Env = [("HOME".to_string(), home)].into_iter().collect();
        let paths = resolve_paths(&env, 1000).unwrap();
        (dir, paths)
    }

    #[test]
    fn safe_remove_stays_inside_its_own_directories() {
        let (_dir, paths) = sandbox();
        let err = safe_remove(&paths.home, &paths).unwrap_err().to_string();
        assert_eq!(err, format!("refusing to remove {}: aicentre does not own it", paths.home));
        assert!(safe_remove(&format!("{}/x", paths.config), &paths).is_err());
        assert!(safe_remove("relative", &paths).is_err());
        let file = format!("{}/versions/0.1.0/aicentre", paths.data);
        fs::create_dir_all(dirname(&file)).unwrap();
        fs::write(&file, "x").unwrap();
        safe_remove(&format!("{}/versions/0.1.0", paths.data), &paths).unwrap();
        assert!(!Path::new(&file).exists());
        safe_remove(&format!("{}/gone", paths.cache), &paths).unwrap();
    }

    #[test]
    fn links_are_ours_only_when_they_point_into_data() {
        let (_dir, paths) = sandbox();
        fs::create_dir_all(&paths.bin_dir).unwrap();
        let link = format!("{}/l", paths.bin_dir);
        symlink("../share/aicentre/current/aicentre", &link).unwrap();
        assert!(is_our_link(&link, &paths));
        let other = format!("{}/o", paths.bin_dir);
        symlink("/usr/bin/aicentre", &other).unwrap();
        assert!(!is_our_link(&other, &paths));
        assert!(!is_our_link(&paths.bin_dir, &paths));
    }

    #[test]
    fn link_bin_leaves_a_foreign_command_alone() {
        let (_dir, paths) = sandbox();
        let first = link_bin(&paths).unwrap();
        assert_eq!(first.result, Linked::Created);
        assert_eq!(first.created, [format!("{}/.local", paths.home), paths.bin_dir.clone()]);
        assert_eq!(link_bin(&paths).unwrap().result, Linked::Kept);
        fs::remove_file(&paths.bin_link).unwrap();
        fs::write(&paths.bin_link, "#!/bin/sh\n").unwrap();
        assert_eq!(link_bin(&paths).unwrap().result, Linked::Foreign);
    }
}
