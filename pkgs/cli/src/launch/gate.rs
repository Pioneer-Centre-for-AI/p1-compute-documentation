use std::net::IpAddr;
use std::os::fd::OwnedFd;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;

use crate::context::Env;

const LIVE_MAX: usize = 16;
const WAITING_MAX: usize = 4;
const SECRET_SECONDS: u64 = 5;

#[derive(Clone)]
pub enum Behind {
    Port(u16, TlsAcceptor),
    Sshd { program: String, config: String, env: Env },
}

pub struct Gate {
    port: u16,
    busy: Arc<AtomicUsize>,
    task: JoinHandle<()>,
}

/// Takes a port for the gate by listening on it. Trying is the only sure
/// test: on a busy node a port can be in use with nothing listening on it.
pub async fn listen(bind: &str) -> std::io::Result<TcpListener> {
    let mut last = std::io::Error::other("no port to try");
    for _ in 0..50 {
        let Some(port) = super::listener::any_port() else { break };
        match TcpListener::bind((bind, port)).await {
            Ok(server) => return Ok(server),
            Err(e) => last = e,
        }
    }
    Err(last)
}

impl Gate {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn busy(&self) -> usize {
        self.busy.load(Ordering::Relaxed)
    }

    pub fn stopped(&self) -> bool {
        self.task.is_finished()
    }
}

struct Counted(Arc<AtomicUsize>);

impl Counted {
    fn one(count: &Arc<AtomicUsize>) -> Counted {
        count.fetch_add(1, Ordering::Relaxed);
        Counted(count.clone())
    }
}

impl Drop for Counted {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Compares all bytes whatever they are, so timing tells nothing of the secret.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn knocked(conn: &mut (impl AsyncRead + Unpin), secret: &[u8]) -> bool {
    let mut got = vec![0u8; secret.len()];
    conn.read_exact(&mut got).await.is_ok() && same(&got, secret)
}

async fn in_time<T>(doing: impl Future<Output = Option<T>>) -> Option<T> {
    tokio::time::timeout(Duration::from_secs(SECRET_SECONDS), doing).await.ok().flatten()
}

async fn forward(conn: impl AsyncRead + AsyncWrite, port: u16) {
    let Ok(mut server) = TcpStream::connect(("127.0.0.1", port)).await else { return };
    let (mut from_conn, mut to_conn) = tokio::io::split(conn);
    let (mut from_server, mut to_server) = server.split();
    tokio::select! {
        _ = tokio::io::copy(&mut from_conn, &mut to_server) => {}
        _ = tokio::io::copy(&mut from_server, &mut to_conn) => {}
    }
}

async fn sshd(conn: TcpStream, program: &str, config: &str, env: &Env, log: &std::fs::File) {
    let Ok(socket) = conn.into_std() else { return };
    if socket.set_nonblocking(false).is_err() {
        return;
    }
    let input = OwnedFd::from(socket);
    let (Ok(output), Ok(log)) = (input.try_clone(), log.try_clone()) else { return };
    let child = tokio::process::Command::new(program)
        .args(["-i", "-e", "-f", config])
        .env_clear()
        .envs(env)
        .stdin(Stdio::from(input))
        .stdout(Stdio::from(output))
        .stderr(Stdio::from(log))
        .kill_on_drop(true)
        .spawn();
    if let Ok(mut child) = child {
        let _ = child.wait().await;
    }
}

pub fn open(
    server: TcpListener,
    allowed: Vec<IpAddr>,
    secret: String,
    behind: Behind,
    log: std::fs::File,
) -> std::io::Result<Gate> {
    let port = server.local_addr()?.port();
    let live = Arc::new(AtomicUsize::new(0));
    let busy = Arc::new(AtomicUsize::new(0));
    let counted = busy.clone();
    let task = tokio::spawn(async move {
        loop {
            let Ok((mut conn, peer)) = server.accept().await else { return };
            let open = live.load(Ordering::Relaxed);
            let waiting = open.saturating_sub(busy.load(Ordering::Relaxed));
            if open >= LIVE_MAX || waiting >= WAITING_MAX || !allowed.contains(&peer.ip()) {
                continue;
            }
            let _open = Counted::one(&live);
            let (secret, behind, busy) = (secret.clone(), behind.clone(), busy.clone());
            let Ok(log) = log.try_clone() else { return };
            tokio::spawn(async move {
                let _open = _open;
                let secret = secret.as_bytes();
                match &behind {
                    Behind::Port(port, seal) => {
                        let let_in = in_time(async {
                            let mut tls = seal.accept(conn).await.ok()?;
                            knocked(&mut tls, secret).await.then_some(tls)
                        });
                        let Some(tls) = let_in.await else { return };
                        let _let_in = Counted::one(&busy);
                        forward(tls, *port).await;
                    }
                    Behind::Sshd { program, config, env } => {
                        let let_in =
                            in_time(async { knocked(&mut conn, secret).await.then_some(()) });
                        if let_in.await.is_none() {
                            return;
                        }
                        let _let_in = Counted::one(&busy);
                        sshd(conn, program, config, env, &log).await;
                    }
                }
            });
        }
    });
    Ok(Gate { port, busy: counted, task })
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::seal::{Sealed, connector, name, sealed};

    async fn said(port: u16, pin: &str, secret: &str) -> String {
        let Ok(conn) = TcpStream::connect(("127.0.0.1", port)).await else {
            return String::new();
        };
        let Ok(mut tls) = connector(pin).unwrap().connect(name(), conn).await else {
            return String::new();
        };
        let _ = tls.write_all(secret.as_bytes()).await;
        let mut line = [0u8; 11];
        let read = tokio::time::timeout(Duration::from_secs(8), tls.read_exact(&mut line));
        match read.await {
            Ok(Ok(_)) => String::from_utf8_lossy(&line).into_owned(),
            _ => String::new(),
        }
    }

    async fn server() -> u16 {
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = server.local_addr().unwrap().port();
        tokio::spawn(async move {
            let mut held = Vec::new();
            while let Ok((mut conn, _)) = server.accept().await {
                let _ = conn.write_all(b"the server\n").await;
                held.push(conn);
            }
        });
        port
    }

    async fn gate(allowed: &str, behind: u16) -> (Gate, u16, String) {
        let log = tempfile::tempfile().unwrap();
        let allowed = vec![allowed.parse().unwrap()];
        let Sealed { acceptor, fingerprint } = sealed().unwrap();
        let behind = Behind::Port(behind, acceptor);
        let server = listen("127.0.0.1").await.unwrap();
        let gate = open(server, allowed, "s".repeat(64), behind, log).unwrap();
        let port = gate.port();
        assert!((20_000..50_000).contains(&port));
        (gate, port, fingerprint)
    }

    #[tokio::test]
    async fn the_gate_takes_another_port_when_one_is_in_use() {
        let mut held = Vec::new();
        for _ in 0..20 {
            held.push(listen("127.0.0.1").await.unwrap());
        }
        let mut ports: Vec<u16> = held.iter().map(|s| s.local_addr().unwrap().port()).collect();
        ports.sort();
        ports.dedup();
        assert_eq!(ports.len(), 20);
    }

    #[test]
    fn secrets_compare_whole() {
        assert!(same(b"abc", b"abc"));
        assert!(!same(b"abc", b"abd") && !same(b"abc", b"ab") && !same(b"", b"a"));
    }

    #[tokio::test]
    async fn only_the_secret_from_an_allowed_address_reaches_the_server() {
        let behind = server().await;
        let (open_gate, port, pin) = gate("127.0.0.1", behind).await;
        assert_eq!(said(port, &pin, &"s".repeat(64)).await, "the server\n");
        assert_eq!(said(port, &pin, &"x".repeat(64)).await, "");
        assert!(!open_gate.stopped());

        let (_closed, port, pin) = gate("10.11.12.13", behind).await;
        assert_eq!(said(port, &pin, &"s".repeat(64)).await, "");
    }

    #[tokio::test]
    async fn the_secret_sent_in_the_clear_gets_nothing() {
        let behind = server().await;
        let (_gate, port, _pin) = gate("127.0.0.1", behind).await;
        let mut conn = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        conn.write_all("s".repeat(64).as_bytes()).await.unwrap();
        let mut got = Vec::new();
        let read = tokio::time::timeout(Duration::from_secs(8), conn.read_to_end(&mut got));
        let _ = read.await;
        assert!(!String::from_utf8_lossy(&got).contains("the server"));
    }

    #[tokio::test]
    async fn strangers_fill_four_places_and_never_count_as_connected() {
        let behind = server().await;
        let (gate, port, pin) = gate("127.0.0.1", behind).await;
        let mut strangers = Vec::new();
        for _ in 0..8 {
            strangers.push(TcpStream::connect(("127.0.0.1", port)).await.unwrap());
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(gate.busy(), 0);
        assert_eq!(said(port, &pin, &"s".repeat(64)).await, "");
        drop(strangers);
        tokio::time::sleep(Duration::from_millis(300)).await;

        let conn = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let mut held = connector(&pin).unwrap().connect(name(), conn).await.unwrap();
        held.write_all("s".repeat(64).as_bytes()).await.unwrap();
        let mut hello = [0u8; 11];
        held.read_exact(&mut hello).await.unwrap();
        assert_eq!(gate.busy(), 1);
    }
}
