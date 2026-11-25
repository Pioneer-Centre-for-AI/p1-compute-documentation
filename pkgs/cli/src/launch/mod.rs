pub mod activity;
pub mod apps;
pub mod fetch;
pub mod gate;
pub mod listener;
pub mod pyenv;
pub mod sshd;
pub mod watch;

use std::io::Write;
use std::net::IpAddr;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use tokio::signal::unix::{SignalKind, signal};

use self::gate::{Behind, Gate};
use self::listener::{Owner, free_port, owner, random_bytes};
use crate::context::{Ctx, Env, for_tests};
use crate::exec::{RunOptions, run, which};
use crate::paths::{basename, dirname, join};

const ENDED: i32 = 143;
const KEEP_LOGS_DAYS: u64 = 14;
const READY_SECONDS: u32 = 600;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct LaunchRequest {
    pub login: bool,
    pub needs: bool,
    pub approved: bool,
    pub app: String,
    pub target: String,
    pub server: String,
    pub data: String,
    pub base: String,
    pub bind: String,
}

pub struct Stop(pub i32);

pub type Step<T> = Result<T, Stop>;

#[derive(Clone, Copy)]
pub struct Out;

impl Out {
    fn line(&self, text: &str) -> Step<()> {
        let mut out = std::io::stdout().lock();
        out.write_all(text.as_bytes()).and_then(|()| out.flush()).map_err(|_| Stop(ENDED))
    }

    pub fn tag(&self, name: &str, value: &str) -> Step<()> {
        self.line(&format!("@@{name} {value}\n"))
    }

    pub fn die<T>(&self, why: &str) -> Step<T> {
        self.tag("ERROR", why)?;
        Err(Stop(1))
    }

    fn or_die<T>(&self, done: Result<T, String>) -> Step<T> {
        match done {
            Ok(value) => Ok(value),
            Err(why) => self.die(&why),
        }
    }
}

#[derive(Default)]
struct Leftovers {
    server: Option<i32>,
    keys: Option<String>,
}

impl Leftovers {
    fn clear(&mut self) {
        if let Some(pid) = self.server.take() {
            // The server leads its own process group, so its children go too.
            let group = nix::unistd::Pid::from_raw(-pid);
            if nix::sys::signal::kill(group, nix::sys::signal::Signal::SIGTERM).is_err() {
                crate::exec::signal(pid, libc::SIGTERM);
            }
        }
        if let Some(keys) = self.keys.take() {
            let _ = std::fs::remove_dir_all(keys);
        }
    }
}

/// LSF splits a job's command line into words again, so the laptop sends
/// any word a shell would change as hex.
pub fn plain(word: &str) -> String {
    match word.strip_prefix("hex:").and_then(|h| hex::decode(h).ok()) {
        Some(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        None => word.to_string(),
    }
}

pub fn resolve(target: &str, home: &str, base: &str) -> String {
    if target.starts_with('/') {
        target.to_string()
    } else if target == "~" {
        home.to_string()
    } else if let Some(rest) = target.strip_prefix("~/") {
        format!("{home}/{rest}")
    } else {
        format!("{base}/{target}")
    }
}

fn hostname() -> String {
    nix::unistd::gethostname().map(|h| h.to_string_lossy().into_owned()).unwrap_or_default()
}

fn private_dir(dir: &str) -> std::io::Result<()> {
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

fn private_file(path: &str, text: &str) -> std::io::Result<()> {
    let mut file =
        std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
    file.write_all(text.as_bytes())
}

fn new_log(path: &str) -> std::io::Result<std::fs::File> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

fn forget_old(dir: &str, days: u64, named: impl Fn(&str) -> bool) {
    let limit = Duration::from_secs(days * 24 * 3600);
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let old = entry
            .metadata()
            .ok()
            .filter(|m| m.is_file())
            .and_then(|m| m.modified().ok())
            .and_then(|at| SystemTime::now().duration_since(at).ok())
            .is_some_and(|age| age > limit);
        if old && named(&name) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn last_lines(path: &str, count: usize) -> Vec<String> {
    let text = std::fs::read(path).unwrap_or_default();
    let text = String::from_utf8_lossy(&text);
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(count)..].iter().map(|l| l.to_string()).collect()
}

async fn gpu_line(env: &Env) -> Option<String> {
    let args = ["--query-gpu=name,memory.used,memory.total", "--format=csv,noheader,nounits"];
    let said = run("nvidia-smi", &args, RunOptions::new(env).timeout_ms(10_000)).await;
    let line = said.stdout.lines().next().filter(|l| said.code == 0 && !l.is_empty())?;
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    let (used, total): (i64, i64) = (fields.get(1)?.parse().ok()?, fields.get(2)?.parse().ok()?);
    Some(format!(
        "{}, {} of {} GiB free, shared with other jobs",
        fields[0],
        (total - used) / 1024,
        total / 1024
    ))
}

async fn login_node(env: &Env) -> Vec<IpAddr> {
    let named: Vec<String> = match for_tests(env, "AICENTRE_GATE_ALLOW") {
        Some(list) => list.split(',').map(str::to_string).collect(),
        None => {
            let host = env.get("LSB_SUB_HOST").map(String::as_str).unwrap_or_default();
            if host.is_empty() {
                return vec![];
            }
            let asked = ["ahostsv4", host];
            let said = run("getent", &asked, RunOptions::new(env).timeout_ms(10_000)).await;
            if said.missing {
                let found = tokio::net::lookup_host((host, 0)).await;
                found.into_iter().flatten().map(|a| a.ip().to_string()).collect()
            } else {
                said.stdout
                    .lines()
                    .filter_map(|l| l.split_whitespace().next())
                    .map(str::to_string)
                    .collect()
            }
        }
    };
    // A name that resolves to this node itself would let every user here in.
    let given = for_tests(env, "AICENTRE_GATE_ALLOW").is_some();
    let mut ips: Vec<IpAddr> = named
        .iter()
        .filter_map(|n| n.trim().parse::<IpAddr>().ok())
        .filter(|ip| given || (ip.is_ipv4() && !ip.is_loopback()))
        .collect();
    ips.sort();
    ips.dedup();
    ips
}

struct Session<'a> {
    out: Out,
    env: &'a Env,
    user: &'a str,
    bind: &'a str,
    keys: &'a str,
    gate_log: &'a str,
}

enum Door {
    Server(u16),
    Sshd,
}

impl Session<'_> {
    async fn open_gate(&self, door: Door) -> Step<Gate> {
        let out = self.out;
        let ips = login_node(self.env).await;
        if ips.is_empty() {
            let host = self.env.get("LSB_SUB_HOST").filter(|h| !h.is_empty());
            let who = host.map_or("the submitting login node", String::as_str);
            return out
                .die(&format!("cannot resolve {who}, so the gate cannot limit who connects"));
        }
        let secret = match random_bytes::<32>() {
            Ok(bytes) => hex::encode(bytes),
            Err(_) => return out.die("cannot make a secret"),
        };
        if private_file(&format!("{}/secret", self.keys), &secret).is_err() {
            return out.die("cannot make a secret");
        }
        let Ok(log) = new_log(self.gate_log) else {
            return out.die(&format!("cannot create {}", self.gate_log));
        };
        let mut host_key = None;
        let mut certificate = None;
        let behind = match door {
            Door::Server(port) => {
                let seal = out.or_die(crate::seal::sealed())?;
                certificate = Some(seal.fingerprint);
                Behind::Port(port, seal.acceptor)
            }
            Door::Sshd => {
                let allow: Vec<String> =
                    ips.iter().map(|ip| format!("{}@{ip}", self.user)).collect();
                let made = sshd::prepare(self.env, self.keys, &allow, &hostname()).await;
                let sshd = out.or_die(made)?;
                host_key = Some(sshd.host_key);
                Behind::Sshd { program: sshd.program, config: sshd.config, env: self.env.clone() }
            }
        };
        let opened = match gate::listen(self.bind).await {
            Ok(server) => gate::open(server, ips, secret.clone(), behind, log),
            Err(e) => Err(e),
        };
        let gate = match opened {
            Ok(gate) => gate,
            Err(e) => return out.die(&format!("the gate cannot listen on {}: {e}", hostname())),
        };
        let port = gate.port();
        if let Some(key) = host_key {
            out.tag("HOSTKEY", &key)?;
        }
        if let Some(print) = certificate {
            out.tag("GATECERT", &print)?;
        }
        out.tag("SECRET", &secret)?;
        out.tag("GATE", &port.to_string())?;
        Ok(gate)
    }
}

fn start(
    env: &Env,
    server: &apps::Server,
    log: &std::fs::File,
    nice: bool,
) -> std::io::Result<tokio::process::Child> {
    let program = which(&server.argv[0], env)
        .ok_or_else(|| std::io::Error::other(format!("{} is not installed", server.argv[0])))?;
    let mut command = tokio::process::Command::new(program);
    command
        .args(&server.argv[1..])
        .env_clear()
        .envs(env)
        .envs(server.env.iter().map(|(k, v)| (k, v)))
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log.try_clone()?));
    // SAFETY: setsid and nice are async-signal-safe and run in the child before exec.
    unsafe {
        command.pre_exec(move || {
            libc::setsid();
            if nice {
                libc::nice(10);
            }
            Ok(())
        });
    }
    command.spawn()
}

fn exit_code(status: std::io::Result<std::process::ExitStatus>) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    match status {
        Ok(s) => s.code().unwrap_or_else(|| 128 + s.signal().unwrap_or(0)),
        Err(_) => 1,
    }
}

async fn shell(
    env: &mut Env,
    out: Out,
    data: &str,
    dir: &str,
    gpu: Option<&str>,
    python: &pyenv::Python,
) -> Step<i32> {
    let installing = |bin: &str| println!("Installing uv into {bin}...");
    out.or_die(fetch::need_uv(env, data, installing).await)?;
    if !python.exists() {
        println!("Making the default Python {} environment...", pyenv::PYTHON);
    }
    out.or_die(pyenv::ensure(env, python).await)?;
    python.activate(env);
    match python {
        pyenv::Python::Project => println!("Python: this folder's project; uv run uses it."),
        pyenv::Python::Own(venv) => println!("Python: {venv}, this folder's own."),
        pyenv::Python::Default(venv) => println!(
            "Python: {venv}, shared by your sessions. pip install adds to it; a folder with a pyproject.toml or a .venv uses its own."
        ),
    }
    let job = ["LSB_JOBID", "SLURM_JOB_ID"].iter().find_map(|k| env.get(*k)).cloned();
    match job.filter(|j| !j.is_empty()) {
        Some(job) => {
            let idle = watch::seconds(env, "AICENTRE_IDLE_SECONDS", 600);
            println!("GPU: {}", gpu.unwrap_or("none visible"));
            println!(
                "Shell on {}, job {job}, in {dir}. 'exit' ends the job, and so does {} min at an idle prompt.",
                hostname(),
                idle / 60
            );
            env.insert("TMOUT".into(), idle.to_string());
        }
        None => println!(
            "Shell on {}, a login node, in {dir}. Keep heavy work for --gpu or batch jobs.",
            hostname()
        ),
    }
    let mut bash = std::process::Command::new("bash");
    match python.venv() {
        // The user's own startup files run first and may put another Python
        // on PATH, a conda or pixi one for example. So the environment is
        // activated after them, from a startup file of its own.
        Some(venv) => {
            let rc = format!("{data}/shellrc");
            let text = format!(
                "[ -f ~/.bashrc ] && . ~/.bashrc\n. {}\n",
                crate::exec::shq(&format!("{venv}/bin/activate"))
            );
            if std::fs::write(&rc, text).is_err() {
                return out.die(&format!("cannot write {rc}"));
            }
            bash.args(["--rcfile", &rc, "-i"]);
        }
        None => {
            bash.arg("-l");
        }
    }
    let failed = bash.env_clear().envs(&*env).exec();
    out.die(&format!("cannot start bash: {failed}"))
}

fn tool_env(ctx: &Ctx, data: &str, home: &str) -> Env {
    let mut env = ctx.env.clone();
    let path = env.get("PATH").cloned().unwrap_or_default();
    env.insert("PATH".into(), format!("{data}/bin:{home}/.local/bin:{path}"));
    env.entry("TERM".into()).or_insert_with(|| "xterm-256color".into());
    for (name, at) in [
        ("UV_CACHE_DIR", "uv/cache"),
        ("UV_PYTHON_INSTALL_DIR", "uv/python"),
        ("UV_TOOL_DIR", "uv/tools"),
        ("UV_TOOL_BIN_DIR", "bin"),
    ] {
        env.insert(name.into(), format!("{data}/{at}"));
    }
    env
}

pub fn python_hint(log: &[String]) -> Option<String> {
    const LISTS: &str = "only has wheels with the following Python ABI tags";
    let listed = log.iter().find_map(|l| l.split_once(LISTS).map(|(_, tags)| tags))?;
    let newest =
        listed.split('`').filter_map(|word| word.strip_prefix("cp3")?.parse::<u32>().ok()).max()?;
    Some(format!(
        "A package here has no build for the Python that uv chose. In this folder, run: uv python pin 3.{newest}. Or cap requires-python in pyproject.toml below 3.{}.",
        newest + 1
    ))
}

async fn serve(ctx: &Ctx, r: &LaunchRequest, left: &Mutex<Leftovers>) -> Step<i32> {
    let out = Out;
    let (data, base) = (plain(&r.data), plain(&r.base));
    let home = ctx.env("HOME").unwrap_or_default().to_string();
    let mut env = tool_env(ctx, &data, &home);
    let target = resolve(&plain(&r.target), &home, &base);
    if !Path::new(&target).exists() {
        let asked = plain(&r.target);
        let at_home = !asked.starts_with(['/', '~']) && Path::new(&home).join(&asked).exists();
        let hint = if at_home {
            format!("; it is in your home directory, as '~/{asked}'")
        } else {
            String::new()
        };
        return out.die(&format!("{target} does not exist on the cluster{hint}"));
    }
    let (dir, file) = if Path::new(&target).is_dir() {
        (target.clone(), None)
    } else {
        (dirname(&target), Some(basename(&target)))
    };
    if std::env::set_current_dir(&dir).is_err() {
        return out.die(&format!("cannot open {dir}"));
    }
    let python = pyenv::choose(&dir, &data);
    let wanted = fetch::wanted(&env, &r.app, &r.server, &data, &python);
    if r.needs {
        for need in &wanted {
            out.tag("NEED", need)?;
        }
        return Ok(0);
    }

    let logs = join(&[&data, "logs"]);
    if std::fs::create_dir_all(join(&[&data, "bin"])).is_err() || private_dir(&logs).is_err() {
        return out.die(&format!("cannot create {data}"));
    }
    if private_dir(&data).is_err() {
        return out.die(&format!("cannot make {data} private"));
    }
    forget_old(&logs, KEEP_LOGS_DAYS, |_| true);
    let cache =
        ctx.env_set("XDG_CACHE_HOME").map_or_else(|| join(&[&home, ".cache"]), str::to_string);
    forget_old(&join(&[&cache, "aicentre"]), 0, |name| {
        name.starts_with("session-") && name.ends_with(".sh")
    });

    if !wanted.is_empty() && !r.approved {
        return out.die(&format!(
            "this session would install {}, which nobody agreed to; start it again and answer yes, or add --yes",
            wanted.join("; ")
        ));
    }

    let gpu = gpu_line(&env).await;
    if r.app == "shell" {
        return shell(&mut env, out, &data, &dir, gpu.as_deref(), &python).await;
    }
    python.activate(&mut env);
    let job = ["LSB_JOBID", "SLURM_JOB_ID"]
        .iter()
        .find_map(|k| env.get(*k).filter(|v| !v.is_empty()))
        .cloned();

    out.tag("PID", &std::process::id().to_string())?;
    out.tag("DIR", &dir)?;
    if let Some(gpu) = &gpu {
        out.tag("GPU", gpu)?;
    }

    let host = hostname();
    let Some(mut port) = free_port() else { return out.die(&format!("no free port on {host}")) };
    let Ok(token) = random_bytes::<24>().map(hex::encode) else {
        return out.die("cannot make a token");
    };
    let tmp = ctx.env_set("TMPDIR").unwrap_or("/tmp");
    let Ok(keys) = nix::unistd::mkdtemp(format!("{tmp}/aicentre-XXXXXX").as_str()) else {
        return out.die("cannot create a private directory");
    };
    let keys = keys.to_string_lossy().into_owned();
    crate::lock(left).keys = Some(keys.clone());
    let token_file = format!("{keys}/token");
    if private_file(&token_file, &token).is_err() {
        return out.die("cannot create a private directory");
    }
    let named = job.clone().unwrap_or_else(|| format!("login-{}", std::process::id()));
    let log = format!("{logs}/{named}-{}.log", r.app);
    let gate_log = format!("{logs}/{named}-gate.log");
    let Ok(log_file) = new_log(&log) else { return out.die(&format!("cannot create {log}")) };

    let user = env.get("USER").cloned().unwrap_or_else(|| ctx.username.clone());
    let session =
        Session { out, env: &env, user: &user, bind: &r.bind, keys: &keys, gate_log: &gate_log };
    let mut gate = None;
    if job.is_some() && r.app != "sshd" {
        out.tag("STATUS", "starting the gate into the job")?;
        gate = Some(session.open_gate(Door::Server(port)).await?);
    }

    let mut server = None;
    let url_path;
    if r.app == "sshd" {
        out.tag("STATUS", "starting the gate for your editor's ssh")?;
        let opened = session.open_gate(Door::Sshd).await?;
        port = opened.port();
        gate = Some(opened);
        url_path = String::new();
    } else {
        let asked = apps::Asked {
            app: &r.app,
            server: &r.server,
            data: &data,
            dir: &dir,
            file: file.as_deref(),
            port,
            token: &token,
            token_file: &token_file,
            in_job: job.is_some(),
            python: &python,
        };
        let status = |text: &str| {
            let _ = out.tag("STATUS", text);
        };
        let how = out.or_die(apps::server(&env, &asked, &status).await)?;
        let child = match start(&env, &how, &log_file, job.is_none()) {
            Ok(child) => child,
            Err(e) => return out.die(&format!("cannot start {}: {e}", r.app)),
        };
        crate::lock(left).server = child.id().map(|id| id as i32);
        server = Some(child);
        url_path = how.path;
    }

    let pretend = for_tests(&env, "AICENTRE_LISTENER_UID").and_then(|u| u.parse().ok());
    for _ in 0..READY_SECONDS {
        if gate.as_ref().is_some_and(Gate::stopped) {
            for line in last_lines(&gate_log, 8) {
                out.tag("LOG", &line)?;
            }
            return out.die(&format!("the gate into the job stopped; its log is {gate_log}"));
        }
        if let Some(child) = &mut server
            && !matches!(child.try_wait(), Ok(None))
        {
            crate::lock(left).server = None;
            let said = last_lines(&log, 8);
            for line in &said {
                out.tag("LOG", line)?;
            }
            if let Some(hint) = python_hint(&said) {
                out.tag("LOG", &hint)?;
            }
            return out.die(&format!("the server stopped before it was ready; its log is {log}"));
        }
        match owner(port, ctx.uid, pretend) {
            Owner::Foreign => {
                return out.die(&format!(
                    "port {port} on {host} is not this session's own, so nothing was opened; start it again"
                ));
            }
            Owner::Mine => {
                fetch::mark_started(&data, &r.app);
                out.tag("HOST", &host)?;
                out.tag("PORT", &port.to_string())?;
                out.tag("PATH", &url_path)?;
                let code = match &gate {
                    Some(gate) => {
                        let what = if r.app == "sshd" { "ssh" } else { "browser" };
                        let record = format!("{logs}/{named}-activity.log");
                        watch::idle(out, &env, gate, server.as_mut(), what, &record).await?
                    }
                    None => watch::login(out, &env, server.as_mut()).await?,
                };
                if server.is_some() && code.is_some() {
                    crate::lock(left).server = None;
                }
                return Ok(code.unwrap_or(0));
            }
            Owner::Nothing => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
    out.die(&format!("the server did not answer within 10 minutes; its log is {log}"))
}

/// Runs this command again under a login shell. A job starts with an empty
/// environment, and the login shell gives it PATH and the site's modules.
fn under_login_shell(ctx: &Ctx, r: &LaunchRequest) -> i32 {
    let words = [&r.app, &r.target, &r.server, &r.data, &r.base, &r.bind];
    let flags = [(r.needs, "--needs"), (r.approved, "--yes")];
    let failed = std::process::Command::new("bash")
        .args(["-lc", r#"exec "$0" "$@""#, &ctx.exec_path, "launch"])
        .args(flags.iter().filter(|(on, _)| *on).map(|(_, flag)| flag))
        .args(words)
        .exec();
    let _ = Out.tag("ERROR", &format!("cannot start a login shell: {failed}"));
    1
}

pub async fn launch(ctx: &Ctx, r: &LaunchRequest) -> anyhow::Result<i32> {
    if r.login {
        return Ok(under_login_shell(ctx, r));
    }
    let left = Arc::new(Mutex::new(Leftovers::default()));
    let mut term = signal(SignalKind::terminate())?;
    let mut hangup = signal(SignalKind::hangup())?;
    let mut interrupt = signal(SignalKind::interrupt())?;
    let code = tokio::select! {
        done = serve(ctx, r, &left) => match done {
            Ok(code) | Err(Stop(code)) => code,
        },
        _ = term.recv() => ENDED,
        _ = hangup.recv() => ENDED,
        _ = interrupt.recv() => ENDED,
    };
    crate::lock(&left).clear();
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_sent_as_hex_arrive_whole_and_paths_resolve_like_a_shell() {
        assert_eq!(plain("work/nb.py"), "work/nb.py");
        assert_eq!(plain(&format!("hex:{}", hex::encode("my  nb;$(id).py"))), "my  nb;$(id).py");
        assert_eq!(plain("hex:zz"), "hex:zz");
        assert_eq!(resolve("/abs", "/h", "/b"), "/abs");
        assert_eq!(resolve("~", "/h", "/b"), "/h");
        assert_eq!(resolve("~/x", "/h", "/b"), "/h/x");
        assert_eq!(resolve("x/y", "/h", "/b"), "/b/x/y");
    }

    #[test]
    fn a_missing_wheel_for_the_chosen_python_says_which_python_to_pin() {
        let log = vec![
            "error: Distribution torch==2.14.1 @ registry+https://pypi.org/simple can't be installed".to_string(),
            "hint: You're using CPython 3.15 (`cp315`), but `torch` (v2.14.1) only has wheels with the following Python ABI tags: `cp313`, `cp314`, `cp314t`".to_string(),
        ];
        let hint = python_hint(&log).unwrap();
        assert!(hint.contains("uv python pin 3.14."), "{hint}");
        assert!(hint.contains("below 3.15."), "{hint}");
        assert_eq!(python_hint(&["Traceback".to_string()]), None);
    }

    #[test]
    fn the_last_lines_of_a_log_and_old_logs() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("a.log").to_string_lossy().into_owned();
        std::fs::write(&log, "1\n2\n3\n").unwrap();
        assert_eq!(last_lines(&log, 2), ["2", "3"]);
        assert_eq!(last_lines("/nonexistent", 2), [] as [&str; 0]);
        let at = dir.path().to_string_lossy().into_owned();
        forget_old(&at, 14, |_| true);
        assert!(Path::new(&log).exists());
        forget_old(&at, 0, |name| name == "other");
        assert!(Path::new(&log).exists());
    }
}
