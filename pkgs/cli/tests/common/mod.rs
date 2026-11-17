#![allow(dead_code)]

pub mod fakes;

use std::cell::Cell;
use std::fs;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt, symlink};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use aicentre::check::probes::net::{NetReason, NetResult};
use aicentre::context::{Captured, Ctx, Env, Output};
use aicentre::paths::resolve_paths;
use aicentre::platform::{Arch, Os, Platform};

const REAL_TOOLS: &[&str] =
    &["sh", "bash", "tar", "gzip", "cat", "env", "sleep", "touch", "rm", "mkdir", "head"];

pub struct Sandbox {
    _dir: tempfile::TempDir,
    pub root: String,
    pub home: String,
    pub fake: String,
    pub env: Env,
    pub out: Captured,
    pub err: Captured,
    pub tty: Cell<bool>,
}

pub fn write_mode(path: &str, text: &str, mode: u32) {
    if let Some(parent) = Path::new(path).parent() {
        fs::DirBuilder::new().recursive(true).mode(0o700).create(parent).unwrap();
    }
    if mode & 0o111 == 0 {
        fs::write(path, text).unwrap();
    } else {
        // ETXTBSY: no fork of this process may hold an executable open for writing.
        let draft = format!("{path}.draft");
        fs::write(&draft, text).unwrap();
        assert!(std::process::Command::new("cp").args([&draft, path]).status().unwrap().success());
        fs::remove_file(&draft).unwrap();
    }
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

impl Sandbox {
    pub fn new() -> Sandbox {
        let dir = tempfile::Builder::new().prefix("aicentre-test-").tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap().to_string_lossy().into_owned();
        let home = format!("{root}/home");
        let fake = format!("{root}/fake");
        let bin = format!("{fake}/bin");
        let tools = format!("{fake}/tools");
        for d in [&home, &bin, &tools, &format!("{fake}/answers"), &format!("{root}/run")] {
            fs::create_dir_all(d).unwrap();
            fs::set_permissions(d, fs::Permissions::from_mode(0o700)).unwrap();
        }
        for (name, text) in [
            ("ssh", fakes::FAKE_SSH),
            ("ssh-keygen", fakes::FAKE_SSH_KEYGEN),
            ("ssh-add", fakes::FAKE_SSH_ADD),
            ("tsh", fakes::FAKE_TSH),
        ] {
            write_mode(&format!("{bin}/{name}"), text, 0o755);
        }
        let path = std::env::var("PATH").unwrap_or_default();
        for tool in REAL_TOOLS {
            if let Ok(real) = which::which_in(tool, Some(&path), "/") {
                symlink(real, format!("{tools}/{tool}")).unwrap();
            }
        }
        let env: Env = [
            ("HOME", home.clone()),
            ("PATH", format!("{bin}:{tools}")),
            ("FAKE", fake.clone()),
            ("XDG_RUNTIME_DIR", format!("{root}/run")),
            ("TERM", "dumb".into()),
            ("USER", "tester".into()),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        Sandbox {
            _dir: dir,
            root,
            home,
            fake,
            env,
            out: Captured::default(),
            err: Captured::default(),
            tty: Cell::new(false),
        }
    }

    pub fn ctx(&self) -> Ctx {
        self.ctx_with(|_| {})
    }

    pub fn ctx_env(&self, extra: &[(&str, &str)]) -> Ctx {
        let mut env = self.env.clone();
        for (k, v) in extra {
            env.insert(k.to_string(), v.to_string());
        }
        self.ctx_with(|c| c.env = env)
    }

    pub fn ctx_with(&self, change: impl FnOnce(&mut Ctx)) -> Ctx {
        let uid = nix::unistd::getuid().as_raw();
        let answers = format!("{}/answers", self.fake);
        let mut ctx = Ctx {
            env: self.env.clone(),
            paths: resolve_paths(&self.env, uid).unwrap(),
            uid,
            username: "tester".into(),
            out: Output::new(Box::new(self.out.clone()), self.tty.get(), Some(100)),
            err: Output::new(Box::new(self.err.clone()), false, Some(100)),
            interactive: false,
            exec_path: "/nonexistent/aicentre".into(),
            compiled: true,
            version: "0.1.0".into(),
            platform: Platform { os: Os::Linux, arch: Arch::X64 },
            tcp: Arc::new(move |host: String, port: u16, _timeout: Duration| {
                let answer = format!("{answers}/tcp-{host}-{port}");
                Box::pin(async move {
                    match fs::read_to_string(&answer) {
                        Err(_) => NetResult::Ok,
                        Ok(text) => NetResult::Failed {
                            reason: NetReason::Timeout,
                            message: text.trim().to_string(),
                        },
                    }
                })
            }),
        };
        change(&mut ctx);
        if ctx.env != self.env {
            ctx.paths = resolve_paths(&ctx.env, uid).unwrap();
        }
        ctx
    }

    pub fn file(&self, path: &str, text: &str) -> String {
        self.file_mode(path, text, 0o600)
    }

    pub fn file_mode(&self, path: &str, text: &str, mode: u32) -> String {
        let full =
            if path.starts_with('/') { path.to_string() } else { format!("{}/{path}", self.home) };
        write_mode(&full, text, mode);
        full
    }

    pub fn answer(&self, name: &str, text: &str) {
        fs::write(format!("{}/answers/{name}", self.fake), text).unwrap();
    }

    pub fn read(&self, name: &str) -> String {
        fs::read_to_string(format!("{}/{name}", self.fake)).unwrap_or_default()
    }

    pub fn norm(&self, text: &str) -> String {
        text.replace(&self.root, "<ROOT>")
    }

    pub async fn run(&self, argv: &[&str], ctx: &Ctx) -> i32 {
        let argv: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
        aicentre::run(&argv, ctx).await
    }
}

pub fn ssh_g(fields: &[(&str, &[&str])]) -> String {
    let mut out = String::new();
    for (k, values) in fields {
        for v in *values {
            out.push_str(&format!("{k} {v}\n"));
        }
    }
    out
}

pub fn dtu_host(sb: &Sandbox) {
    sb.answer(
        "ssh-G-dtu",
        &ssh_g(&[
            ("host", &["dtu"]),
            ("hostname", &["login9.hpc.dtu.dk"]),
            ("user", &["s123456"]),
            ("port", &["22"]),
        ]),
    );
}

pub fn gefion_ready(sb: &Sandbox, valid_to: jiff::Timestamp) {
    let tsh = format!("{}/.tsh", sb.home);
    let cert = format!("{tsh}/keys/secureremote.dcai.dk/jdoe-ssh/secureremote.dcai.dk-cert.pub");
    sb.file(".tsh/secureremote.dcai.dk.yaml", "profile");
    sb.file(&cert, "cert");
    sb.file(".tsh/known_hosts", "@cert-authority *.secureremote.dcai.dk ssh-rsa AAAA\n");
    let known = format!("{tsh}/known_hosts");
    sb.answer(
        "ssh-G-gefion",
        &ssh_g(&[
            ("host", &["gefion"]),
            ("hostname", &["slurm-login-01.hpc.ite.dcai.dk"]),
            ("user", &["jdoe"]),
            ("port", &["3022"]),
            (
                "proxycommand",
                &["tsh proxy ssh --cluster=secureremote.dcai.dk --proxy=secureremote.dcai.dk:443 %r@%h:%p"],
            ),
            ("certificatefile", &[&cert]),
            ("identityfile", &["~/.tsh/keys/secureremote.dcai.dk/jdoe"]),
            ("userknownhostsfile", &[&known]),
        ]),
    );
    let local = valid_to.to_zoned_datetime(jiff::tz::TimeZone::system()).datetime();
    let local = local.strftime("%Y-%m-%dT%H:%M:%S").to_string();
    sb.answer("cert-valid", &format!("from 2026-01-01T00:00:00 to {local}"));
}

pub fn in_hours(h: i64) -> jiff::Timestamp {
    jiff::Timestamp::now() + jiff::SignedDuration::from_hours(h)
}
