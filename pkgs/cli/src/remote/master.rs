use std::process::Stdio;
use std::sync::{Arc, Mutex};

use sha1::{Digest, Sha1};

use crate::context::Ctx;
use crate::exec::{RunOptions, RunResult, Streamed, attach, run, stream};
use crate::paths::{ensure_runtime_dir, join, resolve_paths};

pub const KEEP: &str = "30m";

const SOCKET_MAX: usize = 100;

pub fn shared_socket(ctx: &Ctx, alias: &str) -> anyhow::Result<String> {
    let digest = hex::encode(Sha1::digest(alias.as_bytes()));
    let name = format!("cm-{}", &digest[..12]);
    let path = join(&[&ensure_runtime_dir(&ctx.paths, ctx.uid)?, &name]);
    if path.encode_utf16().count() <= SOCKET_MAX {
        return Ok(path);
    }
    let mut env = ctx.env.clone();
    env.remove("XDG_RUNTIME_DIR");
    env.insert("TMPDIR".into(), "/tmp".into());
    let short = resolve_paths(&env, ctx.uid)?;
    Ok(join(&[&ensure_runtime_dir(&short, ctx.uid)?, &name]))
}

pub async fn is_open(ctx: &Ctx, alias: &str, socket: &str) -> bool {
    let args = ["-S", socket, "-O", "check", alias];
    run("ssh", &args, RunOptions::new(&ctx.env).timeout_ms(5_000)).await.code == 0
}

pub async fn log_in(ctx: &Ctx, alias: &str, socket: &str) -> Result<(), String> {
    let log = format!("{socket}.log");
    let _ = std::fs::remove_file(&log);
    let mut args: Vec<String> = Vec::new();
    if !ctx.interactive {
        args.extend(["-o".into(), "BatchMode=yes".into()]);
    }
    args.extend([
        "-o".into(),
        "ControlMaster=yes".into(),
        "-o".into(),
        format!("ControlPath={socket}"),
        "-o".into(),
        format!("ControlPersist={KEEP}"),
        "-o".into(),
        "ServerAliveInterval=30".into(),
        "-o".into(),
        "ServerAliveCountMax=4".into(),
        "-E".into(),
        log.clone(),
        alias.into(),
        "true".into(),
    ]);
    let mut c = tokio::process::Command::new("ssh");
    c.args(&args)
        .env_clear()
        .envs(&ctx.env)
        .stdin(if ctx.interactive { Stdio::inherit() } else { Stdio::null() })
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let code = match c.status().await {
        Ok(status) => status.code().unwrap_or(1),
        Err(_) => 127,
    };
    let stderr = std::fs::read_to_string(&log).unwrap_or_default();
    let _ = std::fs::remove_file(&log);
    if code == 0 && is_open(ctx, alias, socket).await {
        return Ok(());
    }
    Err(if stderr.is_empty() { format!("ssh exited {code}") } else { stderr })
}

#[derive(Clone)]
pub struct Master {
    pub alias: String,
    socket: String,
    ctx: Arc<Ctx>,
    forwards: Arc<Mutex<Vec<String>>>,
}

impl Master {
    pub fn new(alias: &str, socket: &str, ctx: &Ctx) -> Master {
        Master {
            alias: alias.to_string(),
            socket: socket.to_string(),
            ctx: Arc::new(ctx.clone()),
            forwards: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn socket(&self) -> &str {
        &self.socket
    }

    fn client(&self, args: &[&str]) -> Vec<String> {
        let mut all: Vec<String> =
            ["-S", &self.socket, "-o", "ControlMaster=no"].iter().map(|s| s.to_string()).collect();
        all.extend(args.iter().map(|s| s.to_string()));
        all
    }

    pub async fn start(&self) -> Result<(), String> {
        if is_open(&self.ctx, &self.alias, &self.socket).await {
            return Ok(());
        }
        log_in(&self.ctx, &self.alias, &self.socket).await
    }

    pub async fn run(&self, command: &str, timeout_ms: u64) -> RunResult {
        let args = self.client(&["-T", &self.alias, command]);
        run("ssh", &args, RunOptions::new(&self.ctx.env).timeout_ms(timeout_ms)).await
    }

    pub async fn run_input(&self, command: &str, input: &str, timeout_ms: u64) -> RunResult {
        let args = self.client(&["-T", &self.alias, command]);
        let opts = RunOptions::new(&self.ctx.env).timeout_ms(timeout_ms).input(input);
        run("ssh", &args, opts).await
    }

    pub fn stream(&self, command: &str, tty: bool) -> Streamed {
        let args = self.client(&[
            if tty { "-tt" } else { "-T" },
            "-o",
            "LogLevel=ERROR",
            &self.alias,
            command,
        ]);
        stream("ssh", &args, &self.ctx.env)
    }

    pub async fn attach(&self, command: &str) -> i32 {
        let args = self.client(&["-t", "-o", "LogLevel=ERROR", &self.alias, command]);
        attach("ssh", &args, &self.ctx.env).await
    }

    pub async fn forward(&self, local: u16, host: &str, port: u16) -> RunResult {
        let spec = format!("127.0.0.1:{local}:{host}:{port}");
        let args = ["-S", &self.socket, "-O", "forward", "-L", &spec, &self.alias];
        let r = run("ssh", &args, RunOptions::new(&self.ctx.env).timeout_ms(10_000)).await;
        if r.code == 0 {
            self.forwards.lock().unwrap_or_else(|e| e.into_inner()).push(spec);
        }
        r
    }

    pub async fn close(&self) {
        let specs: Vec<String> =
            std::mem::take(&mut *self.forwards.lock().unwrap_or_else(|e| e.into_inner()));
        for spec in specs {
            let args = ["-S", &self.socket, "-O", "cancel", "-L", &spec, &self.alias];
            run("ssh", &args, RunOptions::new(&self.ctx.env).timeout_ms(5_000)).await;
        }
    }
}
