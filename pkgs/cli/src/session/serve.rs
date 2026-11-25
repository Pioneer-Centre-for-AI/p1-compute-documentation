use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio_util::sync::CancellationToken;

use crate::browser::link;
use crate::check::clusters::GPU_HOST;
use crate::check::types::Status;
use crate::context::Ctx;
use crate::exec::{shq, signal};
use crate::install::layout::iso_now;
use crate::lock;
use crate::paths::Paths;
use crate::session::editor::{Editor, remote_args};
use crate::session::interactive::{
    InteractiveRequest, SessionDeps, app_name, asks_for, session_command,
};
use crate::session::login::Login;
use crate::session::lsf::{App, Event, QUEUE, View, login_shell, parse_line};
use crate::session::proxy::{Relay, secret_proxy};
use crate::session::state::{
    HOST_KEY, NODE, SECRET, SessionNote, URL_PATH, drop_note, live_notes, save_note,
    write_known_hosts,
};
use crate::ui::live::{Animation, animates};
use crate::ui::say::printable;
use crate::ui::steps::Steps;
use crate::ui::style::Style;

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
                "HOSTKEY" | "SECRET" | "GATE" | "GATECERT" | "DIR" => {
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
        "GATECERT" => SECRET.is_match(value),
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

pub(super) async fn serve(
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
        format!("Submitting to {QUEUE}: {}", asks_for(r))
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
            let Some(pin) = got("GATECERT").filter(|p| SECRET.is_match(p)) else {
                return session.failed(&steps, "the job sent no certificate for its gate").await;
            };
            local = (deps.free_port)().await?;
            session.relay = Some(secret_proxy(local, upstream, &secret, &pin).await?);
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

    let clock = running_clock(ctx, &steps.style, r.gpu.then_some(until), u64::from(r.minutes));
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

const WARN_AT: [u64; 2] = [10, 2];

pub fn time_warning(left: u64, total: u64, said: &mut Vec<u64>) -> Option<String> {
    let due: Vec<u64> =
        WARN_AT.iter().copied().filter(|w| left <= *w && total > *w && !said.contains(w)).collect();
    if due.is_empty() {
        return None;
    }
    said.extend(due);
    let unit = if left == 1 { "minute" } else { "minutes" };
    Some(format!(
        "{left} {unit} left. Save your work: the job ends then, and its time cannot be made longer."
    ))
}

fn running_clock(ctx: &Ctx, s: &Style, until: Option<Instant>, total: u64) -> Option<Animation> {
    if !animates(&ctx.env, &ctx.out) {
        return None;
    }
    let s = s.clone();
    let mut said = Vec::new();
    Some(Animation::start(&ctx.out, Duration::from_secs(1), move |area| {
        let left =
            until.map(|u| u.saturating_duration_since(Instant::now()).as_secs().div_ceil(60));
        if let Some(text) = left.and_then(|left| time_warning(left, total, &mut said)) {
            area.print(&[format!("\x07{} {}", s.symbol(Status::Warn), s.bold(&text))]);
        }
        let left = left.map_or("Ctrl-C ends it".to_string(), |m| format!("{m} min left"));
        area.draw(vec![s.dim(&format!("  {} running · {left}", s.green("●")))]);
    }))
}
