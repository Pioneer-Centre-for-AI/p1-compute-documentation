#![cfg(target_os = "linux")]

mod common;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use common::write_mode;
use regex::Regex;

const LAUNCHER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/session/launcher.sh");

const LISTEN: &str = r#"import os, socket
s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", int(os.environ["PORT"])))
s.listen(16)
held = []
while True:
    c, _ = s.accept()
    try:
        c.sendall(b"fake server\n")
    except OSError:
        pass
    held.append(c)
"#;

fn real(tool: &str) -> Option<String> {
    let path = std::env::var("PATH").unwrap_or_default();
    which::which_in(tool, Some(path), "/").ok().map(|p| p.to_string_lossy().into_owned())
}

struct Root {
    _dir: tempfile::TempDir,
    path: String,
    bin: String,
}

fn root() -> Root {
    let dir = tempfile::Builder::new().prefix("aicentre-launcher-").tempdir().unwrap();
    let path = dir.path().canonicalize().unwrap().to_string_lossy().into_owned();
    let bin = format!("{path}/bin");
    std::fs::create_dir_all(&bin).unwrap();
    write_mode(
        &format!("{path}/home/.ssh/authorized_keys"),
        "ssh-ed25519 AAAAtest laptop\n",
        0o644,
    );
    std::fs::create_dir_all(format!("{path}/work")).unwrap();
    let fake = |name: &str, body: &str| {
        write_mode(&format!("{bin}/{name}"), &format!("#!/bin/sh\n{body}\n"), 0o755);
    };
    fake("uv", "exit 0");
    fake(
        "uvx",
        &format!(
            "echo \"$*\" > {path}/uvx.args\necho \"JUPYTER_TOKEN=$JUPYTER_TOKEN\" > {path}/uvx.env\n\
             port=$(echo \"$*\" | sed -n 's/.*--port \\([0-9]*\\).*/\\1/p')\n\
             PORT=$port exec python3 -c '{LISTEN}'"
        ),
    );
    fake(
        "sshd",
        &format!(
            "[ \"$1\" = -t ] && exit 0\ncp \"$4\" {path}/sshd_config\nprintf 'SSH-2.0-fake\\r\\n'\nsleep 1"
        ),
    );
    Root { _dir: dir, path, bin }
}

struct Launched {
    tags: HashMap<String, String>,
    child: Child,
    _lines: mpsc::Receiver<String>,
}

impl Launched {
    fn tag(&self, name: &str) -> Option<&str> {
        self.tags.get(name).map(String::as_str)
    }

    fn port(&self, name: &str) -> u16 {
        self.tag(name).unwrap().parse().unwrap()
    }

    fn stop(mut self) {
        let pid = nix::unistd::Pid::from_raw(self.child.id() as i32);
        let _ = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM);
        let _ = self.child.wait();
    }
}

fn launch(root: &Root, app: &str, target: &str, env: &[(&str, &str)]) -> Launched {
    let data = format!("{}/data", root.path);
    let mut child = Command::new("bash")
        .args([LAUNCHER, app, target, "vscode", &data, &root.path, "127.0.0.1"])
        .env_clear()
        .env("PATH", format!("{}:{}", root.bin, std::env::var("PATH").unwrap_or_default()))
        .env("HOME", format!("{}/home", root.path))
        .env("USER", "tester")
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    let tag = Regex::new(r"^@@([A-Z]+) ?(.*)$").unwrap();
    let mut tags = HashMap::new();
    let deadline = Instant::now() + Duration::from_secs(15);
    while !tags.contains_key("PATH") && !tags.contains_key("ERROR") {
        let left = deadline.saturating_duration_since(Instant::now());
        let Ok(line) = rx.recv_timeout(left) else { break };
        if let Some(m) = tag.captures(&line) {
            tags.entry(m[1].to_string()).or_insert_with(|| m[2].to_string());
        }
    }
    Launched { tags, child, _lines: rx }
}

fn mode(path: &str) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

fn closes(port: u16) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(300));
        if TcpStream::connect(("127.0.0.1", port)).is_err() {
            return true;
        }
    }
    false
}

fn knock(port: u16, secret: &str) -> String {
    let Ok(mut sock) = TcpStream::connect(("127.0.0.1", port)) else { return String::new() };
    let _ = sock.write_all(secret.as_bytes());
    sock.set_read_timeout(Some(Duration::from_secs(8))).unwrap();
    let mut buf = [0u8; 4096];
    match sock.read(&mut buf) {
        Ok(n) => String::from_utf8_lossy(&buf[..n]).into_owned(),
        Err(_) => String::new(),
    }
}

#[test]
fn jupyter_gets_its_token_from_the_environment_opens_the_notebook_by_url_and_logs_privately() {
    let r = root();
    std::fs::write(format!("{}/work/my nb.ipynb", r.path), "{}").unwrap();
    let job = launch(&r, "jupyter", "work/my nb.ipynb", &[]);
    let path = job.tag("PATH").unwrap_or_default().to_string();
    let token =
        Regex::new(r"token=([0-9a-f]{48})$").unwrap().captures(&path).unwrap()[1].to_string();
    assert_eq!(job.tag("ERROR"), None);
    assert_eq!(path, format!("/lab/tree/my%20nb.ipynb?token={token}"));
    let args = std::fs::read_to_string(format!("{}/uvx.args", r.path)).unwrap();
    assert!(
        args.starts_with("--from jupyterlab jupyter-lab --no-browser --ip 127.0.0.1 --port "),
        "{args}"
    );
    assert!(!args.contains(&token));
    assert_eq!(
        std::fs::read_to_string(format!("{}/uvx.env", r.path)).unwrap(),
        format!("JUPYTER_TOKEN={token}\n")
    );
    assert_eq!(mode(&format!("{}/data", r.path)), 0o700);
    assert_eq!(mode(&format!("{}/data/logs", r.path)), 0o700);
    let log = std::fs::read_dir(format!("{}/data/logs", r.path))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .find(|f| f.ends_with("-jupyter.log"))
        .unwrap();
    assert!(log.starts_with("login-"), "{log}");
    assert_eq!(mode(&format!("{}/data/logs/{log}", r.path)), 0o600);
    assert_eq!(job.tag("GATE"), None);
    job.stop();
}

#[test]
fn a_path_that_lsf_would_split_arrives_whole_when_sent_as_hex() {
    let r = root();
    let name = "my  nb;touch pwned $(id).ipynb";
    std::fs::write(format!("{}/work/{name}", r.path), "{}").unwrap();
    let sent = format!("hex:{}", hex::encode(format!("work/{name}")));
    let job = launch(&r, "jupyter", &sent, &[]);
    assert_eq!(job.tag("ERROR"), None);
    let path = job.tag("PATH").unwrap_or_default().to_string();
    assert!(
        path.starts_with("/lab/tree/my%20%20nb%3Btouch%20pwned%20%24%28id%29.ipynb?"),
        "{path}"
    );
    assert!(!std::path::Path::new(&format!("{}/work/pwned", r.path)).exists());
    job.stop();
}

#[test]
fn a_download_that_is_not_the_pinned_file_is_deleted_and_never_run() {
    if real("curl").is_none() {
        return;
    }
    let r = root();
    let from = format!("{}/mirror", r.path);
    std::fs::create_dir_all(&from).unwrap();
    std::fs::write(format!("{from}/stable"), "#!/bin/sh\ntouch pwned\n").unwrap();
    write_mode(&format!("{}/uname", r.bin), "#!/bin/sh\necho x86_64\n", 0o755);
    let job = launch(&r, "code", "work", &[("AICENTRE_FETCH_FROM", &format!("file://{from}"))]);
    let error = job.tag("ERROR").unwrap_or_default().to_string();
    assert!(error.contains("is not the file this aicentre was built for (sha256 "), "{error}");
    assert!(error.contains("so it was deleted and nothing ran"), "{error}");
    let left: Vec<String> = walk(&format!("{}/data/vscode", r.path));
    assert!(left.iter().all(|f| !f.ends_with(".tar.gz") && !f.ends_with("/code")), "{left:?}");
    job.stop();
}

fn walk(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path().to_string_lossy().into_owned();
        if e.path().is_dir() {
            out.extend(walk(&p));
        }
        out.push(p);
    }
    out
}

#[test]
fn a_port_that_someone_else_listens_on_is_never_reported_with_the_token() {
    let r = root();
    write_mode(
        &format!("{}/ss", r.bin),
        "#!/bin/sh\necho 'LISTEN 0 16 127.0.0.1:1 0.0.0.0:*'\n",
        0o755,
    );
    let job = launch(&r, "jupyter", "work", &[]);
    assert!(job.tag("ERROR").unwrap_or_default().contains("is not this session's own"));
    assert_eq!(job.tag("PATH"), None);
    assert_eq!(job.tag("PORT"), None);
    job.stop();
}

#[test]
fn in_a_job_a_browser_server_stays_on_127_0_0_1_and_the_gate_forwards_to_it_after_the_secret() {
    let r = root();
    let job = launch(
        &r,
        "jupyter",
        "work",
        &[("LSB_JOBID", "777"), ("AICENTRE_GATE_ALLOW", "127.0.0.1")],
    );
    assert_eq!(job.tag("ERROR"), None);
    let args = std::fs::read_to_string(format!("{}/uvx.args", r.path)).unwrap();
    assert!(args.contains("--ip 127.0.0.1 --port "), "{args}");
    assert_ne!(job.port("GATE"), job.port("PORT"));
    assert_eq!(job.tag("HOSTKEY"), None);
    assert_eq!(knock(job.port("GATE"), &"0".repeat(64)), "");
    assert_eq!(knock(job.port("GATE"), job.tag("SECRET").unwrap()), "fake server\n");
    assert_eq!(mode(&format!("{}/data/logs/777-gate.log", r.path)), 0o600);
    job.stop();
}

#[test]
fn the_gate_closes_a_connection_from_anywhere_but_the_submitting_login_node_secret_or_not() {
    let r = root();
    let job = launch(
        &r,
        "jupyter",
        "work",
        &[("LSB_JOBID", "777"), ("AICENTRE_GATE_ALLOW", "10.11.12.13")],
    );
    assert_eq!(knock(job.port("GATE"), job.tag("SECRET").unwrap()), "");
    job.stop();
}

#[test]
fn connections_that_never_send_the_secret_do_not_keep_the_job_alive_and_one_that_did_does() {
    let r = root();
    let env = [
        ("LSB_JOBID", "777"),
        ("AICENTRE_GATE_ALLOW", "127.0.0.1"),
        ("AICENTRE_IDLE_SECONDS", "3"),
    ];
    let job = launch(&r, "jupyter", "work", &env);
    assert_eq!(job.tag("ERROR"), None);
    let gate = job.port("GATE");
    let strangers: Vec<TcpStream> =
        (0..8).filter_map(|_| TcpStream::connect(("127.0.0.1", gate)).ok()).collect();
    assert!(closes(job.port("PORT")));
    drop(strangers);

    let job = launch(&r, "jupyter", "work", &env);
    let mut held = TcpStream::connect(("127.0.0.1", job.port("GATE"))).unwrap();
    held.write_all(job.tag("SECRET").unwrap().as_bytes()).unwrap();
    std::thread::sleep(Duration::from_secs(6));
    assert!(TcpStream::connect(("127.0.0.1", job.port("PORT"))).is_ok());
    drop(held);
    assert!(closes(job.port("PORT")));
}

#[test]
fn in_a_job_a_browser_server_ends_once_no_browser_has_been_connected_for_the_idle_limit() {
    let r = root();
    let job = launch(
        &r,
        "jupyter",
        "work",
        &[("LSB_JOBID", "777"), ("LSB_SUB_HOST", "10.11.12.13"), ("AICENTRE_IDLE_SECONDS", "2")],
    );
    assert_eq!(job.tag("ERROR"), None);
    assert!(closes(job.port("PORT")));
}

#[test]
fn sshd_answers_only_behind_the_gate_takes_only_the_submitting_login_node_and_leaves_nothing_behind()
 {
    let r = root();
    let job = launch(
        &r,
        "sshd",
        "work",
        &[
            ("LSB_JOBID", "777"),
            ("AICENTRE_GATE_ALLOW", "127.0.0.1"),
            ("CUDA_VISIBLE_DEVICES", "2"),
        ],
    );
    assert_eq!(job.tag("ERROR"), None);
    assert!(job.tag("HOSTKEY").unwrap().starts_with("ssh-ed25519 "));
    let secret = job.tag("SECRET").unwrap().to_string();
    assert!(Regex::new(r"^[0-9a-f]{64}$").unwrap().is_match(&secret), "{secret}");
    assert_eq!(job.tag("DIR"), Some(format!("{}/work", r.path).as_str()));
    assert_eq!(job.tag("PATH"), Some(""));
    let port = job.port("PORT");
    let copied = format!("{}/sshd_config", r.path);
    assert_eq!(knock(port, &"0".repeat(64)), "");
    assert!(!Path::new(&copied).exists());
    assert_eq!(knock(port, &secret), "SSH-2.0-fake\r\n");
    let config = std::fs::read_to_string(&copied).unwrap();
    assert!(!config.contains("Port"));
    assert!(!config.contains("ListenAddress"));
    assert!(!config.contains("MaxAuthTries"));
    for line in [
        "AllowUsers tester@127.0.0.1",
        "PasswordAuthentication no",
        "AllowTcpForwarding local",
        "AllowAgentForwarding no",
        "SetEnv CUDA_VISIBLE_DEVICES=2 LSB_JOBID=777",
    ] {
        assert!(config.contains(&format!("{line}\n")), "{line}");
    }
    if let Some(sshd) = real("sshd") {
        let checked = Command::new(sshd).args(["-t", "-f", &copied]).output().unwrap();
        assert!(!String::from_utf8_lossy(&checked.stderr).contains("Bad configuration"));
        assert_eq!(checked.status.code(), Some(0));
    }
    let key = Regex::new(r"HostKey (\S+)").unwrap().captures(&config).unwrap()[1].to_string();
    let keys = Path::new(&key).parent().unwrap().to_string_lossy().into_owned();
    assert_eq!(mode(&keys), 0o700);
    assert_eq!(mode(&format!("{keys}/secret")), 0o600);
    job.stop();
    assert!(!Path::new(&keys).exists());
}

const RELAY: &str = r#"
import os, select, socket, sys
s = socket.create_connection(("127.0.0.1", int(sys.argv[1])))
s.sendall(sys.argv[2].encode())
open_in = True
while True:
    ready = select.select([s] + ([0] if open_in else []), [], [])[0]
    if s in ready:
        data = s.recv(65536)
        if not data:
            break
        os.write(1, data)
    if 0 in ready:
        data = os.read(0, 65536)
        if data:
            s.sendall(data)
        else:
            open_in = False
            s.shutdown(socket.SHUT_WR)
"#;

fn e2e() -> bool {
    std::env::var("AICENTRE_E2E").as_deref() == Ok("1")
        && cfg!(target_os = "linux")
        && real("sshd").is_some()
        && real("ssh").is_some()
        && real("python3").is_some()
}

#[test]
fn end_to_end_a_real_ssh_client_reaches_a_real_sshd_i_only_through_the_gate() {
    if !e2e() {
        return;
    }
    let r = root();
    std::fs::remove_file(format!("{}/sshd", r.bin)).unwrap();
    std::fs::write(format!("{}/relay.py", r.path), RELAY).unwrap();
    let client = format!("{}/client", r.path);
    let made = Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f", &client])
        .status()
        .unwrap();
    assert!(made.success());
    let user = nix::unistd::User::from_uid(nix::unistd::getuid()).unwrap().unwrap().name;
    let job = launch(
        &r,
        "sshd",
        "work",
        &[
            ("LSB_JOBID", "777"),
            ("CUDA_VISIBLE_DEVICES", "3"),
            ("AICENTRE_GATE_ALLOW", "127.0.0.1"),
            ("AICENTRE_AUTHORIZED_KEYS", &format!("{client}.pub")),
            ("USER", &user),
        ],
    );
    assert_eq!(job.tag("ERROR"), None);
    let known = format!("{}/known_hosts", r.path);
    std::fs::write(&known, format!("aicentre-gpu {}\n", job.tag("HOSTKEY").unwrap())).unwrap();
    let ssh = |secret: &str| {
        Command::new("ssh")
            .args([
                "-F",
                "/dev/null",
                "-i",
                &client,
                "-o",
                "IdentitiesOnly=yes",
                "-o",
                "BatchMode=yes",
                "-o",
                &format!("UserKnownHostsFile={known}"),
                "-o",
                "StrictHostKeyChecking=yes",
                "-o",
                "HostKeyAlias=aicentre-gpu",
                "-o",
                &format!("ProxyCommand=python3 {}/relay.py {} {secret}", r.path, job.port("PORT")),
                "-l",
                &user,
                "aicentre-gpu",
                "echo \"$LSB_JOBID $CUDA_VISIBLE_DEVICES\"",
            ])
            .stdin(Stdio::null())
            .output()
            .unwrap()
    };
    let good = ssh(job.tag("SECRET").unwrap());
    assert_eq!(String::from_utf8_lossy(&good.stderr), "");
    assert_eq!(String::from_utf8_lossy(&good.stdout), "777 3\n");
    assert_eq!(ssh(&"0".repeat(64)).status.code(), Some(255));
    job.stop();
}
