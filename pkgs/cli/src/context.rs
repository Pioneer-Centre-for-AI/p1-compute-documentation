use std::collections::BTreeMap;
use std::io::{IsTerminal, Write};
use std::sync::{Arc, Mutex};

use crate::check::probes::net::{TcpProbe, tcp_probe};
use crate::paths::{Paths, resolve_paths};
use crate::platform::{Platform, current_platform};
use crate::version::VERSION;

pub type Env = BTreeMap<String, String>;

pub fn for_tests<'a>(env: &'a Env, name: &str) -> Option<&'a str> {
    // Debug builds too, so `cargo run` and `cargo test` build the same binary.
    if !cfg!(any(feature = "test-overrides", debug_assertions)) {
        return None;
    }
    env.get(name).map(String::as_str).filter(|v| !v.is_empty())
}

#[derive(Clone, Copy)]
enum Size {
    Fixed { columns: Option<usize>, rows: Option<usize> },
    Terminal(i32),
}

#[derive(Clone)]
pub struct Output {
    sink: Arc<Mutex<Box<dyn Write + Send>>>,
    pub is_tty: bool,
    size: Size,
}

impl Output {
    pub fn new(sink: Box<dyn Write + Send>, is_tty: bool, columns: Option<usize>) -> Output {
        Output {
            sink: Arc::new(Mutex::new(sink)),
            is_tty,
            size: Size::Fixed { columns, rows: None },
        }
    }

    pub fn stdout() -> Output {
        let is_tty = std::io::stdout().is_terminal();
        Output {
            sink: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
            is_tty,
            size: if is_tty {
                Size::Terminal(1)
            } else {
                Size::Fixed { columns: None, rows: None }
            },
        }
    }

    pub fn stderr() -> Output {
        let is_tty = std::io::stderr().is_terminal();
        Output {
            sink: Arc::new(Mutex::new(Box::new(std::io::stderr()))),
            is_tty,
            size: if is_tty {
                Size::Terminal(2)
            } else {
                Size::Fixed { columns: None, rows: None }
            },
        }
    }

    pub fn capture(is_tty: bool, columns: Option<usize>) -> (Output, Captured) {
        let buf = Captured(Arc::new(Mutex::new(Vec::new())));
        (Output::new(Box::new(buf.clone()), is_tty, columns), buf)
    }

    pub fn write(&self, text: &str) {
        let mut sink = crate::lock(&self.sink);
        let _ = sink.write_all(text.as_bytes());
        let _ = sink.flush();
    }

    pub fn columns(&self) -> Option<usize> {
        match self.size {
            Size::Fixed { columns, .. } => columns,
            Size::Terminal(fd) => window(fd).map(|(c, _)| c),
        }
    }

    pub fn rows(&self) -> Option<usize> {
        match self.size {
            Size::Fixed { rows, .. } => rows,
            Size::Terminal(fd) => window(fd).map(|(_, r)| r),
        }
    }
}

fn window(fd: i32) -> Option<(usize, usize)> {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    // SAFETY: TIOCGWINSZ fills a winsize struct we own.
    let ok = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) } == 0;
    (ok && ws.ws_col > 0).then_some((ws.ws_col as usize, ws.ws_row as usize))
}

#[derive(Clone, Default)]
pub struct Captured(Arc<Mutex<Vec<u8>>>);

impl Captured {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&crate::lock(&self.0)).into_owned()
    }

    pub fn clear(&self) {
        crate::lock(&self.0).clear();
    }
}

impl Write for Captured {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        crate::lock(&self.0).extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct Ctx {
    pub env: Env,
    pub paths: Paths,
    pub uid: u32,
    pub username: String,
    pub out: Output,
    pub err: Output,
    pub interactive: bool,
    pub exec_path: String,
    pub compiled: bool,
    pub version: String,
    pub platform: Platform,
    pub tcp: TcpProbe,
}

impl Ctx {
    pub fn from_process() -> anyhow::Result<Ctx> {
        let env: Env = std::env::vars_os()
            .filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?)))
            .collect();
        let uid = nix::unistd::getuid().as_raw();
        let username = env.get("USER").or_else(|| env.get("LOGNAME")).cloned().unwrap_or_default();
        let exec_path = std::env::current_exe()
            .and_then(|p| p.canonicalize())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "aicentre".into());
        Ok(Ctx {
            paths: resolve_paths(&env, uid)?,
            env,
            uid,
            username,
            out: Output::stdout(),
            err: Output::stderr(),
            interactive: std::io::stdin().is_terminal() && std::io::stdout().is_terminal(),
            exec_path,
            compiled: !cfg!(debug_assertions),
            version: VERSION.to_string(),
            platform: current_platform(),
            tcp: tcp_probe(),
        })
    }

    pub fn env(&self, name: &str) -> Option<&str> {
        self.env.get(name).map(String::as_str)
    }

    pub fn env_set(&self, name: &str) -> Option<&str> {
        self.env(name).filter(|v| !v.is_empty())
    }
}
