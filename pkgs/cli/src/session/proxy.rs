use std::io::Read;
use std::path::Path;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_rustls::TlsConnector;

use super::state::{SECRET, known_hosts_file, live_notes, write_private};
use crate::check::clusters::{GPU_HOST, GpuHost, gpu_host_block};
use crate::check::probes::ssh::{SshConfig, ssh_g};
use crate::context::Ctx;
use crate::paths::{Paths, join, state_dir, tilde};
use crate::platform::is_nix_store;
use crate::remote::master::shared_socket;
use crate::seal::{connector, name};

static DEFAULT_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(^|/)id_(rsa|dsa|ecdsa|ecdsa_sk|ed25519|ed25519_sk|xmss|mldsa[A-Za-z0-9_]*)$")
        .unwrap()
});

pub fn ssh_config_file(paths: &Paths) -> String {
    join(&[&paths.state, "ssh_config"])
}

pub fn include_hint(paths: &Paths) -> Vec<String> {
    let file = tilde(&ssh_config_file(paths), &paths.home);
    vec![
        "Add this line at the top of ~/.ssh/config, before any Host:".into(),
        String::new(),
        format!("  Include {file}"),
        String::new(),
        "Home Manager: the aicentre module adds it with programs.ssh enabled, or set:".into(),
        String::new(),
        format!("  programs.ssh.includes = [ \"{file}\" ];"),
    ]
}

fn proxy_bin(ctx: &Ctx) -> String {
    if Path::new(&ctx.paths.bin_link).exists() {
        return tilde(&ctx.paths.bin_link, &ctx.paths.home);
    }
    if ctx.compiled && !is_nix_store(&ctx.exec_path) {
        return ctx.exec_path.clone();
    }
    "aicentre".into()
}

pub fn identity_files(ctx: &Ctx, login: Option<&SshConfig>) -> Vec<String> {
    let home = &ctx.paths.home;
    let expand = |path: &str| match path.strip_prefix("~/") {
        Some(rest) => join(&[home, rest]),
        None => path.to_string(),
    };
    login
        .map(|cfg| cfg.all("identityfile"))
        .unwrap_or_default()
        .iter()
        .filter(|path| !DEFAULT_KEY.is_match(path) && Path::new(&expand(path)).exists())
        .map(|path| expand(path))
        .collect()
}

pub fn write_ssh_config(ctx: &Ctx, user: &str, login: Option<&SshConfig>) -> anyhow::Result<()> {
    let home = &ctx.paths.home;
    let keys: Vec<String> = identity_files(ctx, login).iter().map(|p| tilde(p, home)).collect();
    let block = gpu_host_block(&GpuHost {
        user,
        bin: &proxy_bin(ctx),
        known_hosts: &tilde(&known_hosts_file(&ctx.paths), home),
        identity_files: &keys,
    });
    state_dir(&ctx.paths, &[])?;
    let mut lines = vec!["# Written by aicentre, which keeps it up to date.".to_string()];
    lines.extend(block);
    lines.push(String::new());
    write_private(&ssh_config_file(&ctx.paths), &lines.join("\n"))?;
    Ok(())
}

pub struct Relay(JoinHandle<()>);

impl Relay {
    pub fn close(&self) {
        self.0.abort();
    }
}

async fn relay(mut client: TcpStream, upstream: u16, secret: String, seal: TlsConnector) {
    let Ok(up) = TcpStream::connect(("127.0.0.1", upstream)).await else { return };
    let connect = tokio::time::timeout(Duration::from_secs(15), seal.connect(name(), up));
    let Ok(Ok(mut up)) = connect.await else { return };
    if up.write_all(secret.as_bytes()).await.is_err() {
        return;
    }
    let _ = tokio::io::copy_bidirectional(&mut client, &mut up).await;
}

pub async fn secret_proxy(
    listen: u16,
    upstream: u16,
    secret: &str,
    fingerprint: &str,
) -> std::io::Result<Relay> {
    let server = TcpListener::bind(("127.0.0.1", listen)).await?;
    let seal = connector(fingerprint).map_err(std::io::Error::other)?;
    let secret = secret.to_string();
    Ok(Relay(tokio::spawn(async move {
        while let Ok((client, _)) = server.accept().await {
            tokio::spawn(relay(client, upstream, secret.clone(), seal.clone()));
        }
    })))
}

static GPU_PROXY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\baicentre\b.*\bproxy\b").unwrap());

pub fn is_gpu_host(cfg: &SshConfig) -> bool {
    GPU_PROXY.is_match(cfg.get("proxycommand").unwrap_or_default())
}

pub async fn gpu_host_ready(ctx: &Ctx) -> bool {
    ssh_g(GPU_HOST, &ctx.env).await.is_ok_and(|cfg| is_gpu_host(&cfg))
}

static GPU_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("^{}(?:-([0-9]+))?$", regex::escape(GPU_HOST))).unwrap());

pub async fn proxy(ctx: &Ctx, host: &str) -> anyhow::Result<i32> {
    let Some(m) = GPU_NAME.captures(host) else {
        ctx.err.write(&format!("aicentre proxy: {host} is not {GPU_HOST} or {GPU_HOST}-<job>\n"));
        return Ok(2);
    };
    let job = m.get(1).map(|j| j.as_str());
    let note = live_notes(&ctx.paths).into_values().find(|n| {
        let usable = n.alias.as_deref().is_some_and(|a| !a.is_empty())
            && n.node.as_deref().is_some_and(|a| !a.is_empty())
            && n.port.is_some_and(|p| p != 0)
            && n.host_key.as_deref().is_some_and(|a| !a.is_empty())
            && SECRET.is_match(n.secret.as_deref().unwrap_or_default());
        usable
            && match job {
                Some(job) => n.jobid == job,
                None => n.host.as_deref() == Some(GPU_HOST),
            }
    });
    let Some(note) = note else {
        let which = job.map(|j| format!(" for job {j}")).unwrap_or_default();
        ctx.err.write(&format!(
            "aicentre proxy: no aicentre code --gpu session{which} is running on this machine\n"
        ));
        return Ok(1);
    };
    let alias = note.alias.unwrap_or_default();
    let socket = shared_socket(ctx, &alias)?;
    let target = format!("{}:{}", note.node.unwrap_or_default(), note.port.unwrap_or_default());
    let args = ["-S", &socket, "-o", "ControlMaster=no", "-W", &target, &alias];
    Ok(gated("ssh", &args, &note.secret.unwrap_or_default(), ctx).await)
}

fn stdin_chunks() -> mpsc::UnboundedReceiver<Vec<u8>> {
    let (tx, rx) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 16 * 1024];
        loop {
            match stdin.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
            }
        }
    });
    rx
}

async fn gated(cmd: &str, args: &[&str], secret: &str, ctx: &Ctx) -> i32 {
    let mut c = tokio::process::Command::new(cmd);
    c.args(args)
        .env_clear()
        .envs(&ctx.env)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let Ok(mut child) = c.spawn() else { return 127 };
    let Some(mut input) = child.stdin.take() else { return 127 };
    let secret = secret.to_string();
    let feed = tokio::spawn(async move {
        if input.write_all(secret.as_bytes()).await.is_err() {
            return;
        }
        let mut chunks = stdin_chunks();
        while let Some(chunk) = chunks.recv().await {
            if input.write_all(&chunk).await.is_err() {
                return;
            }
        }
    });
    let status = child.wait().await;
    feed.abort();
    match status {
        Ok(s) => s.code().unwrap_or(128),
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use tokio::io::AsyncReadExt;

    use super::*;

    #[tokio::test]
    async fn the_relay_sends_the_secret_before_anything_the_browser_sends() {
        let gate = crate::seal::sealed().unwrap();
        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let up_port = upstream.local_addr().unwrap().port();
        let received = tokio::spawn(async move {
            let (sock, _) = upstream.accept().await.unwrap();
            let mut sock = gate.acceptor.accept(sock).await.unwrap();
            let mut data = Vec::new();
            let _ = sock.read_to_end(&mut data).await;
            String::from_utf8_lossy(&data).into_owned()
        });
        let pin = gate.fingerprint.clone();
        // Another test can take the free port before the relay binds it.
        let (listen, relay) = loop {
            let listen = crate::browser::free_port().await.unwrap();
            if let Ok(relay) = secret_proxy(listen, up_port, &"c".repeat(64), &pin).await {
                break (listen, relay);
            }
        };
        let mut client = TcpStream::connect(("127.0.0.1", listen)).await.unwrap();
        client.write_all(b"GET / HTTP/1.1\r\n\r\n").await.unwrap();
        client.shutdown().await.unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let text = tokio::time::timeout_at(deadline.into(), received).await.unwrap().unwrap();
        relay.close();
        assert_eq!(text, format!("{}GET / HTTP/1.1\r\n\r\n", "c".repeat(64)));
    }
}
