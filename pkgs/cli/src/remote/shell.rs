use std::sync::LazyLock;

use regex::Regex;

use super::master::Master;
use crate::context::{Ctx, Env};
use crate::exec::{RunOptions, RunResult, Streamed, attach, run, stream};

static LOGIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^bash -lc ('(?:[^']|'\\'')*'|[^\s']+)$").unwrap());

pub fn unwrap_login(command: &str) -> String {
    match LOGIN.captures(command) {
        Some(m) => format!("bash -c {}", &m[1]),
        None => command.to_string(),
    }
}

#[derive(Clone)]
pub struct LocalShell {
    env: Env,
}

impl LocalShell {
    pub fn new(ctx: &Ctx) -> LocalShell {
        LocalShell { env: ctx.env.clone() }
    }

    pub async fn run(&self, command: &str, timeout_ms: u64) -> RunResult {
        let args = ["-c".to_string(), unwrap_login(command)];
        run("bash", &args, RunOptions::new(&self.env).timeout_ms(timeout_ms)).await
    }

    pub async fn run_input(&self, command: &str, input: &str, timeout_ms: u64) -> RunResult {
        let args = ["-c".to_string(), unwrap_login(command)];
        run("bash", &args, RunOptions::new(&self.env).timeout_ms(timeout_ms).input(input)).await
    }

    pub fn stream(&self, command: &str) -> Streamed {
        stream("bash", &["-c".to_string(), unwrap_login(command)], &self.env)
    }

    pub async fn attach(&self, command: &str) -> i32 {
        attach("bash", &["-c".to_string(), unwrap_login(command)], &self.env).await
    }

    pub async fn forward(&self, _local: u16, _host: &str, _port: u16) -> RunResult {
        RunResult {
            code: 1,
            stderr: "there is no port to forward on the cluster itself".into(),
            ..Default::default()
        }
    }

    pub async fn close(&self) {}
}

#[derive(Clone)]
pub enum Shell {
    Master(Master),
    Local(LocalShell),
}

impl Shell {
    pub async fn run(&self, command: &str, timeout_ms: u64) -> RunResult {
        match self {
            Shell::Master(m) => m.run(command, timeout_ms).await,
            Shell::Local(l) => l.run(command, timeout_ms).await,
        }
    }

    pub async fn run_input(&self, command: &str, input: &str, timeout_ms: u64) -> RunResult {
        match self {
            Shell::Master(m) => m.run_input(command, input, timeout_ms).await,
            Shell::Local(l) => l.run_input(command, input, timeout_ms).await,
        }
    }

    pub fn stream(&self, command: &str, tty: bool) -> Streamed {
        match self {
            Shell::Master(m) => m.stream(command, tty),
            Shell::Local(l) => l.stream(command),
        }
    }

    pub async fn attach(&self, command: &str) -> i32 {
        match self {
            Shell::Master(m) => m.attach(command).await,
            Shell::Local(l) => l.attach(command).await,
        }
    }

    pub async fn forward(&self, local: u16, host: &str, port: u16) -> RunResult {
        match self {
            Shell::Master(m) => m.forward(local, host, port).await,
            Shell::Local(l) => l.forward(local, host, port).await,
        }
    }

    pub async fn close(&self) {
        match self {
            Shell::Master(m) => m.close().await,
            Shell::Local(l) => l.close().await,
        }
    }
}
