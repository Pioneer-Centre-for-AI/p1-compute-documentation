use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

use crate::context::{Env, for_tests};
use crate::exec::{RunOptions, run, which};
use crate::paths::join;

pub struct Sshd {
    pub program: String,
    pub config: String,
    pub host_key: String,
}

/// The sshd reads this file. It listens nowhere: the gate hands it each
/// connection, so there is no Port and no ListenAddress.
pub fn config(keys: &str, allow: &[String], env: &Env) -> String {
    let mut lines = vec![
        format!("HostKey {keys}/host"),
        "PasswordAuthentication no".into(),
        format!("AllowUsers {}", allow.join(" ")),
        "AllowTcpForwarding local".into(),
        "AllowAgentForwarding no".into(),
        "X11Forwarding no".into(),
        "PermitTunnel no".into(),
        "LoginGraceTime 20".into(),
        "Subsystem sftp internal-sftp".into(),
    ];
    let kept: Vec<String> = ["CUDA_VISIBLE_DEVICES", "LSB_JOBID"]
        .iter()
        .filter_map(|name| env.get(*name).map(|value| format!("{name}={value}")))
        .collect();
    if !kept.is_empty() {
        lines.push(format!("SetEnv {}", kept.join(" ")));
    }
    if let Some(file) = for_tests(env, "AICENTRE_AUTHORIZED_KEYS") {
        lines.push(format!("AuthorizedKeysFile {file}"));
        lines.push("StrictModes no".into());
    }
    lines.join("\n") + "\n"
}

pub async fn prepare(env: &Env, keys: &str, allow: &[String], host: &str) -> Result<Sshd, String> {
    let program = which("sshd", env)
        .or_else(|| which("/usr/sbin/sshd", env))
        .ok_or_else(|| format!("{host} has no sshd for your editor to connect to"))?;
    let home = env.get("HOME").map(String::as_str).unwrap_or_default();
    let authorized = match for_tests(env, "AICENTRE_AUTHORIZED_KEYS") {
        Some(file) => file.to_string(),
        None => join(&[home, ".ssh", "authorized_keys"]),
    };
    if !std::fs::metadata(&authorized).is_ok_and(|m| m.len() > 0) {
        return Err(
            "~/.ssh/authorized_keys on the cluster is empty, so your editor cannot log in".into()
        );
    }
    let key = format!("{keys}/host");
    let made = run(
        "ssh-keygen",
        &["-q", "-t", "ed25519", "-N", "", "-C", "aicentre-gpu", "-f", &key],
        RunOptions::new(env).timeout_ms(30_000),
    )
    .await;
    let host_key = std::fs::read_to_string(format!("{key}.pub")).unwrap_or_default();
    if made.code != 0 || host_key.trim().is_empty() {
        return Err("cannot make a host key".into());
    }
    let file = format!("{keys}/sshd_config");
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&file)
        .and_then(|mut f| f.write_all(config(keys, allow, env).as_bytes()))
        .map_err(|e| format!("cannot write {file}: {e}"))?;
    let tested = run(&program, &["-t", "-f", &file], RunOptions::new(env).timeout_ms(30_000)).await;
    if tested.code != 0 {
        let said = format!("{}{}", tested.stdout, tested.stderr);
        let first = said.lines().next().unwrap_or_default();
        return Err(format!("this node's sshd rejects the session's config: {first}"));
    }
    Ok(Sshd { program, config: file, host_key: host_key.trim().to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_config_lets_one_user_in_from_the_login_node_and_listens_nowhere() {
        let env: Env = [("CUDA_VISIBLE_DEVICES", "2"), ("LSB_JOBID", "777"), ("PATH", "/bin")]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let allow = ["s1@10.0.0.1".to_string(), "s1@10.0.0.2".to_string()];
        let text = config("/tmp/k", &allow, &env);
        assert!(text.starts_with("HostKey /tmp/k/host\nPasswordAuthentication no\n"));
        assert!(text.contains("AllowUsers s1@10.0.0.1 s1@10.0.0.2\n"));
        assert!(text.contains("AllowTcpForwarding local\nAllowAgentForwarding no\n"));
        assert!(text.ends_with("SetEnv CUDA_VISIBLE_DEVICES=2 LSB_JOBID=777\n"));
        assert!(!text.contains("Port") && !text.contains("ListenAddress"));
        assert!(!text.contains("StrictModes"));

        let mut own = Env::new();
        own.insert("AICENTRE_AUTHORIZED_KEYS".into(), "/tmp/keys".into());
        let text = config("/tmp/k", &allow, &own);
        assert!(text.ends_with("AuthorizedKeysFile /tmp/keys\nStrictModes no\n"));
        assert!(!text.contains("SetEnv"));
    }
}
