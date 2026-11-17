use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::BoxFuture;
use crate::browser::{has_display, link};
use crate::check::clusters::{ClusterId, GPU_HOST};
use crate::check::probes::connect::parse_tags;
use crate::check::probes::ssh::ssh_g;
use crate::check::types::Status;
use crate::config::load_config_or_say;
use crate::context::Ctx;
use crate::exec::{shq, signal};
use crate::install::layout::iso_now;
use crate::paths::Paths;
use crate::session::editor::{Editor, editor_installed, find_editor, remote_args, split_words};
use crate::session::login::{Choice, Login, not_reachable, with_provider};
use crate::session::lsf::{
    App, Event, QUEUE, Request, Server, View, bsub_command, launcher_args, login_shell, parse_line,
};
use crate::session::proxy::{Relay, gpu_host_ready, include_hint, secret_proxy, write_ssh_config};
use crate::session::state::{
    HOST_KEY, NODE, SECRET, SessionNote, URL_PATH, drop_note, live_notes, save_note,
    write_known_hosts,
};
use crate::ui::live::{LiveArea, Ticker};
use crate::ui::say::{fail, printable};
use crate::ui::steps::Steps;
use crate::ui::style::{Style, make_style};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveRequest {
    pub app: App,
    pub path: String,
    pub gpu: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ClusterId>,
    pub server: Server,
    pub minutes: u32,
    pub cpus: u32,
    pub mem_per_cpu: String,
    pub browser: bool,
    pub view: View,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub another: bool,
}

pub type OpenUrl = Arc<dyn Fn(String) -> BoxFuture<'static, ()> + Send + Sync>;
pub type OpenFolder = Arc<dyn Fn(Vec<String>) -> BoxFuture<'static, Option<String>> + Send + Sync>;
pub type FreePort = Arc<dyn Fn() -> BoxFuture<'static, std::io::Result<u16>> + Send + Sync>;

#[derive(Clone)]
pub struct SessionDeps {
    pub cancel: CancellationToken,
    pub open_url: OpenUrl,
    pub open_folder: OpenFolder,
    pub free_port: FreePort,
}

fn app_name(app: App) -> &'static str {
    match app {
        App::Shell => "Shell",
        App::Marimo => "marimo",
        App::Jupyter => "JupyterLab",
        App::Code => "VS Code",
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn no_gpu_on(asked: Option<ClusterId>) -> Option<String> {
    match asked {
        None | Some(ClusterId::Dtu) => None,
        Some(ClusterId::Gefion) => Some(format!(
            "{} runs GPU work as batch jobs; in aicentre shell, sbatch submits one.",
            ClusterId::Gefion.name()
        )),
        Some(id) => not_reachable(id),
    }
}

fn session_command(login: &Login, r: &InteractiveRequest, bsub_mode: &str) -> String {
    if r.gpu {
        let request = Request {
            app: r.app,
            path: r.path.clone(),
            server: r.server,
            view: Some(r.view),
            minutes: r.minutes,
            cpus: r.cpus,
            mem_per_cpu: r.mem_per_cpu.clone(),
        };
        return login_shell(&bsub_command(
            &request,
            &login.launcher,
            &login.data,
            &login.base,
            bsub_mode,
        ));
    }
    let args = launcher_args(
        r.app,
        &r.path,
        r.server,
        Some(r.view),
        &login.launcher,
        &login.data,
        &login.base,
        "127.0.0.1",
    );
    login_shell(&args.iter().map(|a| shq(a)).collect::<Vec<_>>().join(" "))
}

const OPEN_HINT: &str = "set [desktop] open in ~/.config/aicentre/config.toml.";

pub fn why_not_desktop(
    ctx: &Ctx,
    gpu: bool,
    editor: Option<&Editor>,
    gpu_host: bool,
    configured: Option<&str>,
) -> Option<Vec<String>> {
    let display = has_display(ctx);
    let Some(editor) = editor else {
        if let Some(open) = configured.filter(|c| !c.is_empty())
            && display
        {
            let program = split_words(open).into_iter().next().unwrap_or_default();
            return Some(vec![format!(
                "the [desktop] open command {program} is not installed here."
            )]);
        }
        if display {
            return Some(vec![
                "found no editor that opens folders over ssh here;".into(),
                OPEN_HINT.into(),
            ]);
        }
        return Some(vec!["this terminal has no display for a desktop app.".into()]);
    };
    if !editor.ready {
        return Some(vec![
            format!("{} has no Remote-SSH extension yet;", editor.name),
            format!("install one, or {OPEN_HINT}"),
        ]);
    }
    if gpu && !gpu_host {
        return Some(vec![
            format!("~/.ssh/config does not include aicentre's file for {GPU_HOST} yet;"),
            "aicentre check shows the line to add.".into(),
        ]);
    }
    None
}

fn folder_of(path: &str) -> String {
    let dir = path.rfind('/').map_or(path, |i| &path[..i]);
    if dir.is_empty() { "/".into() } else { dir.into() }
}

async fn open_desktop(
    ctx: &Ctx,
    login: &Login,
    r: &InteractiveRequest,
    editor: &Editor,
    deps: &SessionDeps,
) -> anyhow::Result<i32> {
    let path = shq(&r.path);
    let probe = login
        .master
        .run(
            &login_shell(&format!(
                "if [ -d {path} ]; then echo @@KIND dir; elif [ -e {path} ]; then echo @@KIND file; fi"
            )),
            15_000,
        )
        .await;
    let Some(kind) = parse_tags(&probe.stdout).remove("KIND").filter(|k| !k.is_empty()) else {
        return Ok(fail(ctx, &format!("{} does not exist on the cluster.", r.path), &[]));
    };
    let folder = if kind == "dir" { r.path.clone() } else { folder_of(&r.path) };
    if let Some(problem) =
        (deps.open_folder)(remote_args(&editor.argv, &login.alias, &folder)).await
    {
        return Ok(fail(ctx, &format!("{} did not open: {problem}", editor.name), &[]));
    }
    let s = make_style(&ctx.env, &ctx.out);
    ctx.out.write(&format!(
        "{} {} on {}, the {} login node\n{}\n",
        s.symbol(Status::Ok),
        s.bold(&format!("{} opens {folder}", editor.name)),
        login.alias,
        login.provider.name(),
        s.dim(&format!(
            "  Any ssh tool reaches it as {}:{folder}. It runs until you close it.",
            login.alias
        ))
    ));
    Ok(0)
}

const DESKTOP_GPU: &str = "Your desktop editor on a GPU node needs --desktop: it runs an ssh server of your own inside the job, and DTU has not said it allows that.";

fn browser_instead(ctx: &Ctx, missing: &[String]) {
    let style = make_style(&ctx.env, &ctx.out);
    ctx.out.write(&style.dim(&format!("{} Opening it in the browser.\n", missing.join(" "))));
}

pub async fn interactive(
    ctx: &Ctx,
    r: &InteractiveRequest,
    deps: &SessionDeps,
) -> anyhow::Result<i32> {
    if r.gpu
        && let Some(why) = no_gpu_on(r.provider)
    {
        return Ok(fail(ctx, &why, &[]));
    }

    if r.gpu
        && !r.another
        && let Some(note) = live_notes(&ctx.paths).into_values().next()
    {
        let error = format!(
            "job {} from this laptop already holds a GPU on {QUEUE}, for aicentre {}",
            note.jobid, note.app
        );
        let hint =
            format!("{QUEUE} has two GPUs for everyone. End that session first, or add --another.");
        return Ok(fail(ctx, &error, &[hint]));
    }

    let mut editor = None;
    if r.app == App::Code && r.gpu && r.view == View::Auto {
        let Some(config) = load_config_or_say(ctx) else { return Ok(2) };
        if editor_installed(ctx, &config) {
            browser_instead(ctx, &[DESKTOP_GPU.into()]);
        }
    } else if r.app == App::Code && r.view != View::Browser {
        let Some(config) = load_config_or_say(ctx) else { return Ok(2) };
        let open = config.desktop.as_ref().and_then(|d| d.open.clone()).filter(|o| !o.is_empty());
        let found = find_editor(ctx, &config).await;
        match why_not_desktop(ctx, r.gpu, found.as_ref(), true, open.as_deref()) {
            None => editor = found,
            Some(missing) if r.view == View::Desktop => {
                let error = format!("cannot open the desktop app: {}", missing[0]);
                return Ok(fail(ctx, &error, &missing[1..]));
            }
            Some(missing) => {
                if found.is_some() || open.is_some() {
                    browser_instead(ctx, &missing);
                }
            }
        }
    }

    let asked = if r.gpu { Some(ClusterId::Dtu) } else { r.provider };
    with_provider(ctx, &Choice::asked(asked), |login, _| logged_in(ctx, r, deps, login, editor))
        .await
}

async fn logged_in(
    ctx: &Ctx,
    r: &InteractiveRequest,
    deps: &SessionDeps,
    login: Login,
    mut editor: Option<Editor>,
) -> anyhow::Result<i32> {
    if login.local && !(r.app == App::Shell && r.gpu) {
        let why = if r.app == App::Shell {
            format!(
                "you are already on the {} login node. aicentre shell --gpu opens one on a GPU node.",
                login.provider.name()
            )
        } else {
            format!("{} opens on your laptop, so run this from there.", app_name(r.app))
        };
        return Ok(fail(ctx, &why, &[]));
    }
    if r.gpu
        && let Some(found) = editor.clone()
    {
        let dtu = ssh_g(&login.alias, &ctx.env).await;
        write_ssh_config(ctx, &login.user, dtu.as_ref().ok())?;
        let ready = gpu_host_ready(ctx).await;
        if let Some(missing) = why_not_desktop(ctx, r.gpu, Some(&found), ready, None) {
            if r.view == View::Desktop {
                let mut hints = missing[1..].to_vec();
                hints.extend(include_hint(&ctx.paths));
                let error = format!("cannot open the desktop app: {}", missing[0]);
                return Ok(fail(ctx, &error, &hints));
            }
            browser_instead(ctx, &missing);
            editor = None;
        }
    }
    let view = if editor.is_some() { View::Desktop } else { View::Browser };
    let path = login.path(if r.path.is_empty() { "~" } else { &r.path });
    let req = InteractiveRequest { path, view, ..r.clone() };
    if let Some(editor) = &editor
        && !r.gpu
    {
        return open_desktop(ctx, &login, &req, editor, deps).await;
    }
    if r.app != App::Shell {
        return serve(ctx, &login, &req, deps, editor).await;
    }
    if r.gpu {
        ctx.out.write(&format!(
            "Asking {QUEUE} for a GPU for {} min. The prompt appears on the GPU node; exit ends the job.\n",
            r.minutes
        ));
    }
    Ok(login.master.attach(&session_command(&login, &req, "-Is")).await)
}

#[derive(Default)]
struct Heard {
    jobid: Option<String>,
    pid: Option<String>,
    error: Option<String>,
    logs: Vec<String>,
    recent: Vec<String>,
    got: HashMap<String, String>,
}

impl Heard {
    fn got(&self, key: &str) -> Option<String> {
        self.got.get(key).cloned()
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref().filter(|e| !e.is_empty())
    }
}

struct Listener {
    paths: Paths,
    app: App,
    steps: Steps,
    heard: Arc<Mutex<Heard>>,
    ready: CancellationToken,
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

impl Listener {
    fn hear(&self, line: &str) {
        let mut heard = lock(&self.heard);
        match parse_line(&printable(line)) {
            Event::Submitted { jobid } => {
                heard.jobid = Some(jobid.clone());
                let note = SessionNote {
                    jobid: jobid.clone(),
                    app: self.app.as_str().into(),
                    pid: std::process::id() as i32,
                    started_at: iso_now(),
                    ..Default::default()
                };
                let _ = save_note(&self.paths, &note);
                self.steps.step(&format!("Waiting for a GPU on {QUEUE} (job {jobid})"));
            }
            Event::Started { host } => self.steps.step(&format!("Running on {host}")),
            Event::Text { line } => {
                let line = line.trim();
                if !line.is_empty() {
                    heard.recent.push(line.to_string());
                }
                if heard.recent.len() > 8 {
                    heard.recent.remove(0);
                }
            }
            Event::Tag { name, value } if !usable(&name, &value) => {
                heard.got.insert("BAD".into(), name);
                self.ready.cancel();
            }
            Event::Tag { name, value } => match name.as_str() {
                "PID" => heard.pid = Some(value),
                "GPU" => self.steps.note(&format!("GPU: {value}")),
                "STATUS" => self.steps.step(&capitalize(&value)),
                "ERROR" => heard.error = Some(value),
                "LOG" => heard.logs.push(value),
                "HOSTKEY" | "SECRET" | "GATE" | "DIR" => {
                    heard.got.insert(name, value);
                }
                "HOST" | "PORT" | "PATH" => {
                    heard.got.insert(name, value);
                    let set = |k: &str| heard.got.get(k).is_some_and(|v| !v.is_empty());
                    if set("HOST") && set("PORT") && heard.got.contains_key("PATH") {
                        self.ready.cancel();
                    }
                }
                _ => {}
            },
            Event::Queued | Event::Busy => {}
        }
    }
}

fn usable(name: &str, value: &str) -> bool {
    match name {
        "PID" => !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()),
        "HOST" => NODE.is_match(value),
        "PATH" => URL_PATH.is_match(value),
        "DIR" => value.starts_with('/'),
        "PORT" | "GATE" => value.trim().parse::<u16>().is_ok(),
        _ => true,
    }
}

struct Session<'a> {
    ctx: &'a Ctx,
    login: &'a Login,
    heard: Arc<Mutex<Heard>>,
    child: Option<i32>,
    desktop: bool,
    relay: Option<Relay>,
    name: String,
    gpu: bool,
}

impl Session<'_> {
    fn forget(&self, jobid: &str) -> anyhow::Result<()> {
        drop_note(&self.ctx.paths, jobid);
        if self.desktop {
            write_known_hosts(&self.ctx.paths)?;
        }
        Ok(())
    }

    fn close_relay(&self) {
        if let Some(relay) = &self.relay {
            relay.close();
        }
    }

    async fn release(&self, jobid: &str) -> Result<(), String> {
        let r = self.login.master.run(&login_shell(&format!("bkill {jobid}")), 15_000).await;
        let said = format!("{}{}", r.stdout, r.stderr);
        if r.code == 0 || said.contains("already finished") || said.contains("No matching job") {
            return Ok(());
        }
        Err(if r.timed_out {
            "the login node did not answer".into()
        } else {
            said.trim().lines().next_back().unwrap_or("the login node could not be reached").into()
        })
    }

    async fn end(&self, code: i32, message: Option<String>) -> anyhow::Result<i32> {
        self.close_relay();
        let (jobid, pid) = {
            let heard = lock(&self.heard);
            (heard.jobid.clone(), heard.pid.clone().filter(|p| !p.is_empty()))
        };
        let mut message = message;
        if let Some(jobid) = jobid {
            if let Err(why) = self.release(&jobid).await {
                message = Some(unconfirmed(&jobid, &why));
            }
            self.forget(&jobid)?;
        } else if let Some(pid) = pid {
            self.login.master.run(&format!("kill {}", shq(&pid)), 15_000).await;
        }
        if let Some(child) = self.child {
            signal(child, libc::SIGTERM);
        }
        if let Some(message) = message {
            self.ctx.out.write(&format!("{message}\n"));
        }
        Ok(code)
    }

    fn stopped(&self) -> String {
        if let Some(jobid) = &lock(&self.heard).jobid {
            return format!("Ended job {jobid}.");
        }
        if self.gpu {
            "Stopped before the job was submitted.".into()
        } else {
            format!("Stopped {} on {}.", self.name, self.login.alias)
        }
    }

    async fn stop(&self) -> anyhow::Result<i32> {
        let message = self.stopped();
        self.end(130, Some(message)).await
    }

    async fn failed(&self, steps: &Steps, text: &str) -> anyhow::Result<i32> {
        steps.fail(text, &[]);
        steps.done();
        self.end(1, None).await
    }
}

fn unconfirmed(jobid: &str, why: &str) -> String {
    format!(
        "Could not confirm that job {jobid} ended: {why}\n  It may still hold a GPU. Check with aicentre list jobs; in aicentre shell, bkill {jobid} ends it."
    )
}

enum First {
    Ready,
    Exit,
    Abort,
}

async fn serve(
    ctx: &Ctx,
    login: &Login,
    r: &InteractiveRequest,
    deps: &SessionDeps,
    editor: Option<Editor>,
) -> anyhow::Result<i32> {
    let steps = Steps::new(&ctx.env, &ctx.out);
    let name = editor.as_ref().map_or_else(|| app_name(r.app).to_string(), |e| e.name.clone());
    let heard = Arc::new(Mutex::new(Heard::default()));
    let ready = CancellationToken::new();
    let exited = CancellationToken::new();

    steps.step(&if r.gpu {
        format!("Submitting to {QUEUE}")
    } else {
        format!("Starting {name} on {}", login.alias)
    });
    let job = login.master.stream(&session_command(login, r, "-I"), !r.gpu);
    let listener = Listener {
        paths: ctx.paths.clone(),
        app: r.app,
        steps: steps.clone(),
        heard: heard.clone(),
        ready: ready.clone(),
    };
    let left = Arc::new(AtomicI32::new(0));
    let (mut lines, exit, done, code) = (job.lines, job.exit, exited.clone(), left.clone());
    tokio::spawn(async move {
        while let Some(line) = lines.recv().await {
            listener.hear(&line);
        }
        code.store(exit.await.unwrap_or(1), Ordering::Relaxed);
        done.cancel();
    });

    let mut session = Session {
        ctx,
        login,
        heard: heard.clone(),
        child: job.pid,
        desktop: editor.is_some(),
        relay: None,
        name: name.clone(),
        gpu: r.gpu,
    };

    let first = tokio::select! {
        biased;
        _ = ready.cancelled() => First::Ready,
        _ = exited.cancelled() => First::Exit,
        _ = deps.cancel.cancelled() => First::Abort,
    };
    match first {
        First::Abort => {
            steps.done();
            return session.stop().await;
        }
        First::Exit => {
            let (why, hint, jobid) = {
                let heard = lock(&heard);
                let mut hint = heard.logs.clone();
                if heard.error().is_none() {
                    hint.extend(heard.recent.iter().cloned());
                }
                let hint = hint.split_off(hint.len().saturating_sub(8));
                (why_not_started(r.gpu, &heard), hint, heard.jobid.clone())
            };
            steps.fail(&format!("{name} did not start: {why}"), &hint);
            steps.done();
            if let Some(jobid) = jobid {
                if let Err(why) = session.release(&jobid).await {
                    ctx.out.write(&format!("{}\n", unconfirmed(&jobid, &why)));
                }
                session.forget(&jobid)?;
            }
            return Ok(1);
        }
        First::Ready => {}
    }

    let started_at = iso_now();
    let got = |key: &str| lock(&heard).got(key);
    if let Some(bad) = got("BAD") {
        let text = format!("the cluster sent a {bad} that cannot be used, so nothing was opened");
        return session.failed(&steps, &text).await;
    }
    let jobid = lock(&heard).jobid.clone();
    let secret = got("SECRET").unwrap_or_default();
    let gate = got("GATE").and_then(|g| g.trim().parse::<u16>().ok());
    if r.gpu && (!SECRET.is_match(&secret) || gate.is_none()) {
        return session.failed(&steps, "the job sent no usable secret or gate port").await;
    }
    let shown;
    let mut url = None;
    if let Some(editor) = &editor {
        let host_key = got("HOSTKEY").unwrap_or_default();
        if !HOST_KEY.is_match(&host_key) {
            return session.failed(&steps, "the job's sshd sent no usable host key").await;
        }
        let taken = live_notes(&ctx.paths).values().any(|note| {
            note.host.as_deref() == Some(GPU_HOST) && Some(&note.jobid) != jobid.as_ref()
        });
        let jobid = jobid.clone().unwrap_or_default();
        let host = if taken { format!("{GPU_HOST}-{jobid}") } else { GPU_HOST.to_string() };
        save_note(
            &ctx.paths,
            &SessionNote {
                jobid,
                app: r.app.as_str().into(),
                pid: std::process::id() as i32,
                started_at,
                url: None,
                alias: Some(login.alias.clone()),
                node: got("HOST"),
                port: gate,
                host_key: Some(host_key),
                secret: Some(secret),
                host: Some(host.clone()),
            },
        )?;
        write_known_hosts(&ctx.paths)?;
        let dir = got("DIR").filter(|d| !d.is_empty()).unwrap_or_else(|| r.path.clone());
        if let Some(problem) = (deps.open_folder)(remote_args(&editor.argv, &host, &dir)).await {
            return session
                .failed(&steps, &format!("{} did not open: {problem}", editor.name))
                .await;
        }
        shown = format!(
            "{} opens {dir} on {host}; any ssh tool reaches it as {host}:{dir}",
            editor.name
        );
    } else {
        let upstream = (deps.free_port)().await?;
        let (target, port) = if r.gpu {
            (got("HOST").unwrap_or_default(), gate.unwrap_or(0))
        } else {
            ("127.0.0.1".to_string(), got("PORT").and_then(|p| p.trim().parse().ok()).unwrap_or(0))
        };
        let fwd = login.master.forward(upstream, &target, port).await;
        if fwd.code != 0 {
            let text = format!("cannot forward a port to {target}:{port}: {}", fwd.stderr.trim());
            return session.failed(&steps, &text).await;
        }
        let mut local = upstream;
        if r.gpu {
            local = (deps.free_port)().await?;
            session.relay = Some(secret_proxy(local, upstream, &secret).await?);
        }
        let address = format!("http://127.0.0.1:{local}{}", got("PATH").unwrap_or_default());
        if let Some(jobid) = jobid {
            let note = SessionNote {
                jobid,
                app: r.app.as_str().into(),
                pid: std::process::id() as i32,
                started_at,
                url: Some(address.clone()),
                ..Default::default()
            };
            save_note(&ctx.paths, &note)?;
        }
        shown = link(&address, ctx.out.is_tty);
        url = Some(address);
    }
    steps.done();

    let until = Instant::now() + Duration::from_secs(u64::from(r.minutes) * 60);
    let block = ready_block(&steps.style, login, r, &lock(&heard), &name, &shown);
    ctx.out.write(&block);
    if let Some(url) = url
        && r.browser
    {
        (deps.open_url)(url).await;
    }

    let clock = running_clock(ctx, &steps.style, r.gpu.then_some(until));
    let aborted = tokio::select! {
        biased;
        _ = exited.cancelled() => false,
        _ = deps.cancel.cancelled() => true,
    };
    if let Some(clock) = clock {
        clock.stop();
    }
    if aborted {
        return session.stop().await;
    }
    session.close_relay();
    let (jobid, error) = {
        let heard = lock(&heard);
        (heard.jobid.clone(), heard.error.clone())
    };
    if let Some(jobid) = &jobid {
        let released = session.release(jobid).await;
        session.forget(jobid)?;
        if let Err(why) = released {
            ctx.out.write(&format!("{}\n", unconfirmed(jobid, &why)));
            return Ok(1);
        }
    }
    let reason = if r.gpu && Instant::now() + Duration::from_secs(60) >= until {
        "its time ran out".to_string()
    } else if let Some(error) = error {
        error
    } else if matches!(left.load(Ordering::Relaxed), 128 | 255) {
        let then = if jobid.is_some() { ", so the job was ended to free the GPU" } else { "" };
        format!("the connection to {} was lost{then}", login.alias)
    } else {
        "the server stopped".into()
    };
    let who = jobid.map_or(name, |j| format!("Job {j}"));
    ctx.out.write(&format!("{} {who} ended: {reason}.\n", steps.style.symbol(Status::Info)));
    Ok(0)
}

fn why_not_started(gpu: bool, heard: &Heard) -> String {
    if let Some(error) = heard.error() {
        return error.to_string();
    }
    let why = if !gpu {
        "it stopped before it was ready"
    } else if heard.jobid.is_some() {
        "the job ended before the server was ready"
    } else {
        "bsub did not submit the job"
    };
    why.into()
}

fn ready_block(
    s: &Style,
    login: &Login,
    r: &InteractiveRequest,
    heard: &Heard,
    name: &str,
    shown: &str,
) -> String {
    let place = if r.gpu {
        heard.got("HOST").unwrap_or_default()
    } else {
        format!("{}, the {} login node", login.alias, login.provider.name())
    };
    let hint = if r.gpu {
        let jobid = heard.jobid.as_deref();
        let mut hint =
            format!("  Job {} has {} min. Ctrl-C here ends it.", jobid.unwrap_or("?"), r.minutes);
        if r.view == View::Desktop {
            hint.push_str(&format!(
                "\n  So do 10 min with no {name} window or other ssh connection to it."
            ));
        }
        hint
    } else {
        "  It runs until you press Ctrl-C, or for 12 h. For real compute, add --gpu.".into()
    };
    let mut lines = vec![
        String::new(),
        format!("{} {} on {place}", s.symbol(Status::Ok), s.bold(&format!("{name} is ready"))),
        String::new(),
        format!("    {shown}"),
    ];
    if r.view != View::Desktop
        && let Some(dir) = heard.got.get("DIR").filter(|d| !d.is_empty())
    {
        lines.push(format!("    {}", s.dim(&format!("in {dir}"))));
    }
    lines.extend([String::new(), s.dim(&hint), String::new()]);
    lines.join("\n") + "\n"
}

struct Clock {
    area: Arc<Mutex<LiveArea>>,
    ticker: Ticker,
}

impl Clock {
    fn stop(mut self) {
        self.ticker.stop();
        lock(&self.area).stop();
    }
}

fn running_clock(ctx: &Ctx, s: &Style, until: Option<Instant>) -> Option<Clock> {
    if !ctx.out.is_tty || ctx.env("TERM") == Some("dumb") {
        return None;
    }
    let s = s.clone();
    let render = move || {
        let left = match until {
            None => "Ctrl-C ends it".to_string(),
            Some(until) => {
                let ms = until.saturating_duration_since(Instant::now()).as_millis();
                format!("{} min left", ms.div_ceil(60_000))
            }
        };
        vec![s.dim(&format!("  {} running · {left}", s.green("●")))]
    };
    let mut area = LiveArea::new(ctx.out.clone());
    area.start();
    area.draw(render());
    let area = Arc::new(Mutex::new(area));
    let shared = area.clone();
    let ticker = Ticker::every(Duration::from_millis(1_000), move || {
        let lines = render();
        lock(&shared).draw(lines);
    });
    Some(Clock { area, ticker })
}
