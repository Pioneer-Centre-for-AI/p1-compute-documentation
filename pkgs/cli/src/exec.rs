use std::collections::HashSet;
use std::ffi::OsStr;
use std::io::IsTerminal;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::context::Env;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub missing: bool,
}

#[derive(Clone)]
pub struct RunOptions<'a> {
    pub env: &'a Env,
    pub timeout: Duration,
    pub detached: bool,
    pub input: Option<String>,
}

impl<'a> RunOptions<'a> {
    pub fn new(env: &'a Env) -> Self {
        RunOptions { env, timeout: Duration::from_millis(15_000), detached: false, input: None }
    }

    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout = Duration::from_millis(ms);
        self
    }

    pub fn detached(mut self) -> Self {
        self.detached = true;
        self
    }

    pub fn input(mut self, text: impl Into<String>) -> Self {
        self.input = Some(text.into());
        self
    }
}

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(on: bool) {
    VERBOSE.store(on, Ordering::Relaxed);
}

fn verbose(env: &Env) -> bool {
    VERBOSE.load(Ordering::Relaxed)
        || env.get("AICENTRE_DEBUG").is_some_and(|v| !v.is_empty() && v != "0")
}

const SHOWN: usize = 240;

pub fn command_line<S: AsRef<OsStr>>(cmd: &str, args: &[S]) -> String {
    let mut words = vec![cmd.to_string()];
    for a in args {
        let a = a.as_ref().to_string_lossy();
        let cut: String = a.chars().take(SHOWN).collect();
        let more = a.chars().count().saturating_sub(SHOWN);
        words.push(if more > 0 { format!("{}…(+{more})", shq(&cut)) } else { shq(&a) });
    }
    words.join(" ")
}

fn say(env: &Env, text: &str) {
    let plain = env.get("NO_COLOR").is_some_and(|v| !v.is_empty())
        || env.get("TERM").is_some_and(|t| t == "dumb")
        || !std::io::stderr().is_terminal();
    if plain {
        eprintln!("[aicentre] {text}");
    } else {
        eprintln!("\x1b[2m[aicentre] {text}\x1b[22m");
    }
}

fn trace(env: &Env, text: &str) {
    if verbose(env) {
        say(env, text);
    }
}

fn took(since: Instant) -> String {
    format!("{:.2} s", since.elapsed().as_secs_f64())
}

static LIVE: LazyLock<Mutex<HashSet<i32>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

fn live() -> std::sync::MutexGuard<'static, HashSet<i32>> {
    LIVE.lock().unwrap_or_else(|e| e.into_inner())
}

fn kill(pid: i32, group: bool) {
    if !live().contains(&pid) {
        return;
    }
    // SAFETY: kill(2) on a pid we spawned and have not yet reaped.
    unsafe {
        if !group || libc::kill(-pid, libc::SIGKILL) != 0 {
            libc::kill(pid, libc::SIGKILL);
        }
    }
}

pub fn kill_all() {
    let pids: Vec<i32> = live().iter().copied().collect();
    for pid in pids {
        // SAFETY: as in `kill`.
        unsafe {
            if libc::kill(-pid, libc::SIGKILL) != 0 {
                libc::kill(pid, libc::SIGKILL);
            }
        }
    }
}

pub fn signal(pid: i32, sig: i32) {
    if live().contains(&pid) {
        // SAFETY: kill(2) on a pid we spawned and have not yet reaped.
        unsafe {
            libc::kill(pid, sig);
        }
    }
}

fn command<S: AsRef<OsStr>>(cmd: &str, args: &[S], env: &Env) -> Command {
    let mut c = Command::new(cmd);
    c.args(args).env_clear().envs(env);
    c
}

fn setsid(c: &mut Command) {
    // SAFETY: setsid is async-signal-safe and runs in the child before exec.
    unsafe {
        c.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
}

fn exit_code(status: std::io::Result<std::process::ExitStatus>) -> i32 {
    match status {
        Ok(s) => s.code().unwrap_or(128),
        Err(_) => 1,
    }
}

async fn read_all<R: AsyncRead + Unpin>(r: Option<R>) -> String {
    let mut buf = Vec::new();
    if let Some(mut r) = r {
        let _ = r.read_to_end(&mut buf).await;
    }
    String::from_utf8_lossy(&buf).into_owned()
}

pub async fn run<S: AsRef<OsStr>>(cmd: &str, args: &[S], opts: RunOptions<'_>) -> RunResult {
    let (env, line, since) = (opts.env, command_line(cmd, args), Instant::now());
    let mut c = command(cmd, args, opts.env);
    c.stdin(if opts.input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if opts.detached {
        setsid(&mut c);
    }
    let mut child = match c.spawn() {
        Ok(child) => child,
        Err(e) => {
            trace(env, &format!("{line}: {e}"));
            return RunResult {
                code: 127,
                stderr: format!("spawn {cmd} {}", e),
                missing: e.kind() == std::io::ErrorKind::NotFound,
                ..Default::default()
            };
        }
    };
    let pid = child.id().map(|p| p as i32).unwrap_or(0);
    live().insert(pid);
    if let (Some(input), Some(mut stdin)) = (opts.input, child.stdin.take()) {
        tokio::spawn(async move {
            let _ = stdin.write_all(input.as_bytes()).await;
        });
    }
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let finished = async {
        let (out, err, status) = tokio::join!(read_all(stdout), read_all(stderr), child.wait());
        (out, err, status)
    };
    tokio::pin!(finished);
    let mut timed_out = false;
    let (stdout, stderr, status) = tokio::select! {
        r = &mut finished => r,
        _ = tokio::time::sleep(opts.timeout) => {
            timed_out = true;
            kill(pid, opts.detached);
            finished.await
        }
    };
    live().remove(&pid);
    let code = exit_code(status);
    let how = if timed_out { "timed out".to_string() } else { format!("exit {code}") };
    trace(env, &format!("{line}  ({how}, {})", took(since)));
    RunResult { code, stdout, stderr, timed_out, missing: false }
}

pub struct Streamed {
    pub pid: Option<i32>,
    pub lines: mpsc::UnboundedReceiver<String>,
    pub exit: JoinHandle<i32>,
}

async fn split_lines<R: AsyncRead + Unpin>(r: Option<R>, tx: mpsc::UnboundedSender<String>) {
    let Some(mut r) = r else { return };
    let mut rest: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match r.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                rest.extend_from_slice(&chunk[..n]);
                while let Some(i) = rest.iter().position(|b| *b == b'\n') {
                    let mut line: Vec<u8> = rest.drain(..=i).collect();
                    line.pop();
                    if line.last() == Some(&b'\r') {
                        line.pop();
                    }
                    let _ = tx.send(String::from_utf8_lossy(&line).into_owned());
                }
            }
        }
    }
    if !rest.is_empty() {
        let _ = tx.send(String::from_utf8_lossy(&rest).into_owned());
    }
}

pub fn stream<S: AsRef<OsStr>>(cmd: &str, args: &[S], env: &Env) -> Streamed {
    let (tx, rx) = mpsc::unbounded_channel();
    let (since, name) = (Instant::now(), cmd.to_string());
    let loud = verbose(env).then(|| env.clone());
    trace(env, &format!("{}  (started)", command_line(cmd, args)));
    let mut c = command(cmd, args, env);
    c.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = match c.spawn() {
        Ok(child) => child,
        Err(_) => {
            return Streamed { pid: None, lines: rx, exit: tokio::spawn(async { 127 }) };
        }
    };
    let pid = child.id().map(|p| p as i32);
    if let Some(pid) = pid {
        live().insert(pid);
    }
    let out = tokio::spawn(split_lines(child.stdout.take(), tx.clone()));
    let err = tokio::spawn(split_lines(child.stderr.take(), tx));
    let exit = tokio::spawn(async move {
        let status = child.wait().await;
        let _ = out.await;
        let _ = err.await;
        if let Some(pid) = pid {
            live().remove(&pid);
        }
        let code = exit_code(status);
        if let Some(env) = loud {
            say(&env, &format!("{name} ended  (exit {code}, {})", took(since)));
        }
        code
    });
    Streamed { pid, lines: rx, exit }
}

pub async fn attach<S: AsRef<OsStr>>(cmd: &str, args: &[S], env: &Env) -> i32 {
    let since = Instant::now();
    trace(env, &format!("{}  (attached)", command_line(cmd, args)));
    let mut c = command(cmd, args, env);
    c.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let code = match c.status().await {
        Ok(s) => s.code().unwrap_or(128),
        Err(_) => 127,
    };
    trace(env, &format!("{cmd} ended  (exit {code}, {})", took(since)));
    code
}

pub fn which(cmd: &str, env: &Env) -> Option<String> {
    let path = env.get("PATH").map(String::as_str).unwrap_or("");
    if path.is_empty() && !cmd.contains('/') {
        return None;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| "/".into());
    which::which_in(cmd, Some(path), cwd).ok().map(|p| p.to_string_lossy().into_owned())
}

pub fn shq(word: &str) -> String {
    let plain = !word.is_empty()
        && word.chars().all(|c| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c));
    if plain { word.to_string() } else { format!("'{}'", word.replace('\'', "'\\''")) }
}

pub fn login_shell(command: &str) -> String {
    format!("bash -lc {}", shq(command))
}

pub fn errno_name(e: &std::io::Error) -> String {
    match e.raw_os_error() {
        Some(n) => {
            nix::errno::Errno::from_raw(n).to_string().split(':').next().unwrap_or("").to_string()
        }
        None => e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> Env {
        let mut e = Env::new();
        e.insert("PATH".into(), std::env::var("PATH").unwrap_or_default());
        e
    }

    #[test]
    fn quoting() {
        assert_eq!(shq("abc/def-1.2"), "abc/def-1.2");
        assert_eq!(shq(""), "''");
        assert_eq!(shq("a b"), "'a b'");
        assert_eq!(shq("it's"), "'it'\\''s'");
        assert_eq!(login_shell("echo hi"), "bash -lc 'echo hi'");
    }

    #[tokio::test]
    async fn run_collects_output_and_codes() {
        let e = env();
        let r = run("sh", &["-c", "echo out; echo err >&2; exit 3"], RunOptions::new(&e)).await;
        assert_eq!((r.code, r.stdout.as_str(), r.stderr.as_str()), (3, "out\n", "err\n"));
        let r = run("cat", &[] as &[&str], RunOptions::new(&e).input("hello")).await;
        assert_eq!(r.stdout, "hello");
        let r = run("no-such-command-here", &[] as &[&str], RunOptions::new(&e)).await;
        assert!(r.missing);
        assert_eq!(r.code, 127);
        let r = run("sh", &["-c", "sleep 5"], RunOptions::new(&e).timeout_ms(100).detached()).await;
        assert!(r.timed_out);
        assert_eq!(r.code, 128);
    }

    #[tokio::test]
    async fn stream_splits_lines_and_keeps_the_last() {
        let e = env();
        let mut s = stream("sh", &["-c", "printf 'a\\r\\nb\\nc'"], &e);
        let mut lines = Vec::new();
        while let Some(l) = s.lines.recv().await {
            lines.push(l);
        }
        assert_eq!(lines, ["a", "b", "c"]);
        assert_eq!(s.exit.await.unwrap(), 0);
    }
}
