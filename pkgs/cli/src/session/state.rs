use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::check::clusters::GPU_HOST;
use crate::paths::{Paths, join, state_dir};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionNote {
    #[serde(default)]
    pub jobid: String,
    #[serde(default)]
    pub app: String,
    pub pid: i32,
    #[serde(default)]
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

fn dir(paths: &Paths) -> String {
    join(&[&paths.state, "gpu"])
}

pub fn write_private(path: &str, text: &str) -> std::io::Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(text.as_bytes())
}

pub fn save_note(paths: &Paths, note: &SessionNote) -> anyhow::Result<()> {
    let file = join(&[&state_dir(paths, &["gpu"])?, &format!("{}.json", note.jobid)]);
    write_private(&file, &format!("{}\n", serde_json::to_string_pretty(note)?))?;
    Ok(())
}

pub fn drop_note(paths: &Paths, jobid: &str) {
    let _ = std::fs::remove_file(join(&[&dir(paths), &format!("{jobid}.json")]));
}

fn alive(pid: i32) -> bool {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok()
}

pub fn live_notes(paths: &Paths) -> BTreeMap<String, SessionNote> {
    let mut notes = BTreeMap::new();
    let Ok(entries) = std::fs::read_dir(dir(paths)) else { return notes };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let note = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<SessionNote>(&t).ok());
        match note {
            Some(note) if alive(note.pid) => {
                notes.insert(note.jobid.clone(), note);
            }
            _ => {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    notes
}

pub static SECRET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}$").unwrap());

pub static NODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9]([A-Za-z0-9.-]{0,251}[A-Za-z0-9])?$").unwrap());

pub static URL_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(/[A-Za-z0-9._~%/-]*(\?[A-Za-z_]+=[A-Za-z0-9]+)?)?$").unwrap());

pub static HOST_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^ssh-ed25519 [A-Za-z0-9+/]+={0,2}( [A-Za-z0-9_.@-]+)?$").unwrap()
});

pub fn known_hosts_file(paths: &Paths) -> String {
    join(&[&paths.state, "known_hosts"])
}

pub fn write_known_hosts(paths: &Paths) -> anyhow::Result<()> {
    let lines: String = live_notes(paths)
        .values()
        .filter_map(|note| {
            let key = note.host_key.as_deref().filter(|k| HOST_KEY.is_match(k))?;
            Some(format!("{GPU_HOST}-{},{GPU_HOST} {key}\n", note.jobid))
        })
        .collect();
    write_private(&join(&[&state_dir(paths, &[])?, "known_hosts"]), &lines)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    use super::*;
    use crate::paths::resolve_paths;

    #[test]
    fn state_is_private_and_only_ed25519_host_keys_are_trusted() {
        let root = tempfile::tempdir().unwrap();
        let env = [("HOME".to_string(), root.path().to_string_lossy().into_owned())].into();
        let paths = resolve_paths(&env, nix::unistd::getuid().as_raw()).unwrap();
        std::fs::create_dir_all(&paths.state).unwrap();
        std::fs::set_permissions(&paths.state, std::fs::Permissions::from_mode(0o755)).unwrap();
        let gpu = state_dir(&paths, &["gpu"]).unwrap();
        assert_eq!(std::fs::metadata(&paths.state).unwrap().mode() & 0o777, 0o700);
        assert_eq!(std::fs::metadata(&gpu).unwrap().mode() & 0o777, 0o700);

        assert!(HOST_KEY.is_match("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHdw+/= aicentre-gpu"));
        assert!(!HOST_KEY.is_match("ssh-rsa AAAAB3Nza aicentre-gpu"));
        assert!(!HOST_KEY.is_match("ssh-ed25519 AAAA\n@cert-authority * ssh-ed25519 AAAA"));
        assert!(!HOST_KEY.is_match("ssh-ed25519 AAAA aicentre gpu"));
    }

    #[test]
    fn notes_of_ended_sessions_are_dropped() {
        let root = tempfile::tempdir().unwrap();
        let env = [("HOME".to_string(), root.path().to_string_lossy().into_owned())].into();
        let paths = resolve_paths(&env, nix::unistd::getuid().as_raw()).unwrap();
        let mine = SessionNote {
            jobid: "1".into(),
            app: "code".into(),
            pid: std::process::id() as i32,
            started_at: "2026-11-13T10:00:00Z".into(),
            host_key: Some("ssh-ed25519 AAAA aicentre-gpu".into()),
            ..Default::default()
        };
        save_note(&paths, &mine).unwrap();
        save_note(&paths, &SessionNote { jobid: "2".into(), pid: i32::MAX, ..mine.clone() })
            .unwrap();
        std::fs::write(join(&[&dir(&paths), "3.json"]), "{").unwrap();
        let text = std::fs::read_to_string(join(&[&dir(&paths), "1.json"])).unwrap();
        assert!(text.starts_with("{\n  \"jobid\": \"1\",\n  \"app\": \"code\",\n  \"pid\": "));
        assert_eq!(live_notes(&paths).into_keys().collect::<Vec<_>>(), ["1"]);
        assert!(!std::path::Path::new(&join(&[&dir(&paths), "3.json"])).exists());
        write_known_hosts(&paths).unwrap();
        assert_eq!(
            std::fs::read_to_string(known_hosts_file(&paths)).unwrap(),
            "aicentre-gpu-1,aicentre-gpu ssh-ed25519 AAAA aicentre-gpu\n"
        );
    }
}
