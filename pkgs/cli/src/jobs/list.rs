use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use serde::Serialize;

use super::gpus::{
    Gpu, Queue, advice, bhosts_command, headline, memory, parse_bhosts, parse_sinfo, sinfo_command,
};
use super::lsf::{bjobs_command, no_jobs, parse_bjobs};
use super::slurm::{parse_squeue, squeue_command};
use super::types::{Job, JobState};
use crate::check::aliases::{Resolver, resolver};
use crate::check::clusters::{ClusterId, DTU, GEFION, tsh_login};
use crate::check::probes::connect::{FailureContext, classify_ssh_failure, provider_context};
use crate::check::probes::ssh::{Tokens, expand_path};
use crate::check::probes::tsh::{cert_validity, duration};
use crate::check::select::in_use;
use crate::check::types::Status;
use crate::config::{Config, load_config_or_say};
use crate::context::Ctx;
use crate::exec::{RunOptions, RunResult, login_shell, run};
use crate::remote::master::{KEEP, is_open, log_in, shared_socket};
use crate::remote::shell::LocalShell;
use crate::session::here::here;
use crate::session::state::live_notes;
use crate::ui::live::{Animation, animates};
use crate::ui::say::printable_lines;
use crate::ui::style::{Style, make_style};
use crate::ui::table::{TableOptions, table};

#[derive(Clone, Debug, Serialize)]
pub struct ClusterJobs {
    pub id: ClusterId,
    pub name: String,
    #[serde(skip)]
    pub jobs: Vec<Job>,
    #[serde(skip)]
    pub gpus: Vec<Gpu>,
    #[serde(skip)]
    pub queues: Vec<Queue>,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
}

impl ClusterJobs {
    fn new(id: ClusterId) -> ClusterJobs {
        ClusterJobs {
            id,
            name: id.name().into(),
            jobs: vec![],
            gpus: vec![],
            queues: vec![],
            ok: false,
            error: None,
            hint: None,
            login: None,
        }
    }
}

const NGC_GPUS: &str = "air-gapped, so what is free there is only visible on its remote desktop";
const NGC_NOTE: &str = "air-gapped, so its queue is only visible on its remote desktop (qstat)";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum What {
    Jobs,
    Gpus,
}

async fn query(
    ctx: &Ctx,
    id: ClusterId,
    hosts: &Resolver,
    what: What,
) -> anyhow::Result<ClusterJobs> {
    let base = ClusterJobs::new(id);
    let command = login_shell(&match (what, id == ClusterId::Dtu) {
        (What::Jobs, true) => bjobs_command(),
        (What::Jobs, false) => squeue_command(),
        (What::Gpus, true) => bhosts_command(),
        (What::Gpus, false) => sinfo_command(),
    });
    if here(ctx) == Some(id) {
        return Ok(parse(base, &LocalShell::new(ctx).run(&command, 30_000).await, what));
    }

    let Some(alias) = hosts.alias(id).await else {
        let login = if id == ClusterId::Dtu { DTU.login } else { GEFION.login };
        return Ok(ClusterJobs {
            error: Some(format!("no Host in ~/.ssh/config points at {login}")),
            hint: Some(vec!["aicentre check shows the block to add.".into()]),
            ..base
        });
    };

    let tsh = if id == ClusterId::Gefion {
        teleport(ctx, hosts, &alias).await
    } else {
        Teleport { user: String::new(), left: None }
    };
    if let Some(left) = tsh.left
        && left <= 0
    {
        let user = Some(tsh.user.as_str()).filter(|u| !u.is_empty());
        return Ok(ClusterJobs {
            error: Some(format!("your Teleport login expired {} ago", duration(left))),
            hint: Some(vec![tsh_login(user)]),
            ..base
        });
    }

    let r = over_ssh(ctx, &alias, &command).await?;
    if (r.code == 255 || r.timed_out) && !(id == ClusterId::Dtu && no_jobs(&r.stderr)) {
        let context = FailureContext {
            timed_out: r.timed_out,
            ..provider_context(id, &alias, Some(&tsh.user), None)
        };
        let f = classify_ssh_failure(&r.stderr, &context);
        let typed = f.kind == "password" || f.kind == "host-key-unknown";
        let mut hint = f.hint;
        if !typed {
            hint.push("aicentre check --connect looks closer.".into());
        }
        return Ok(ClusterJobs {
            error: Some(format!("cannot reach {alias}: {}", f.message)),
            hint: Some(hint),
            login: typed.then_some(alias),
            ..base
        });
    }
    Ok(parse(base, &r, what))
}

struct Teleport {
    user: String,
    left: Option<i64>,
}

async fn teleport(ctx: &Ctx, hosts: &Resolver, alias: &str) -> Teleport {
    let Ok(cfg) = hosts.ssh_g(alias).await else {
        return Teleport { user: String::new(), left: None };
    };
    let user = cfg.get("user").unwrap_or("").to_string();
    let tokens = Tokens {
        home: ctx.paths.home.clone(),
        local_user: ctx.username.clone(),
        remote_user: Some(user.clone()),
        ..Default::default()
    };
    let cert = cfg.all("certificatefile").first().map(|p| expand_path(p, &tokens));
    let Some(cert) = cert.filter(|c| !c.is_empty() && Path::new(c).exists()) else {
        return Teleport { user, left: None };
    };
    let left = match cert_validity(&cert, &ctx.env).await.and_then(|v| v.to) {
        Some(to) => to.as_millisecond() - jiff::Timestamp::now().as_millisecond(),
        None => i64::MAX,
    };
    Teleport { user, left: Some(left) }
}

async fn over_ssh(ctx: &Ctx, alias: &str, command: &str) -> anyhow::Result<RunResult> {
    let socket = shared_socket(ctx, alias)?;
    let mut args: Vec<String> =
        ["-o", "BatchMode=yes", "-o", "ConnectTimeout=10"].map(String::from).to_vec();
    if is_open(ctx, alias, &socket).await {
        args.extend(["-S".into(), socket, "-o".into(), "ControlMaster=no".into()]);
    }
    args.extend(["-T".into(), alias.into(), command.into()]);
    Ok(run("ssh", &args, RunOptions::new(&ctx.env).timeout_ms(30_000).detached()).await)
}

fn last_line(text: &str) -> Option<&str> {
    text.trim().split('\n').rfind(|l| !l.is_empty())
}

fn parse(base: ClusterJobs, r: &RunResult, what: What) -> ClusterJobs {
    let r = &RunResult {
        stdout: printable_lines(&r.stdout),
        stderr: printable_lines(&r.stderr),
        ..r.clone()
    };
    let dtu = base.id == ClusterId::Dtu;
    if what == What::Gpus {
        let (queues, gpus) =
            if dtu { parse_bhosts(&r.stdout) } else { (parse_sinfo(&r.stdout), vec![]) };
        if queues.iter().all(|q| q.gpus == Some(0)) {
            let tool = if dtu { "bhosts" } else { "sinfo" };
            let why = last_line(&r.stderr).unwrap_or("it listed no GPUs");
            return ClusterJobs { error: Some(format!("{tool} failed: {why}")), ..base };
        }
        return ClusterJobs { ok: true, queues, gpus, ..base };
    }
    if dtu && no_jobs(&format!("{}{}", r.stdout, r.stderr)) {
        return ClusterJobs { ok: true, ..base };
    }
    if r.code != 0 {
        let last = last_line(&r.stderr).map_or_else(|| format!("exit {}", r.code), str::to_string);
        let tool = if dtu { "bjobs" } else { "squeue" };
        return ClusterJobs { error: Some(format!("{tool} failed: {last}")), ..base };
    }
    let mut jobs = if dtu { parse_bjobs(&r.stdout) } else { parse_squeue(&r.stdout) };
    jobs.sort_by(|a, b| a.order().cmp(&b.order()));
    ClusterJobs { ok: true, jobs, ..base }
}

#[derive(Clone, Debug, Serialize)]
pub struct Row {
    #[serde(flatten)]
    pub job: Job,
    pub provider: ClusterId,
}

pub fn merge(results: &[ClusterJobs]) -> Vec<Row> {
    let mut rows: Vec<Row> = results
        .iter()
        .flat_map(|r| r.jobs.iter().map(|j| Row { job: j.clone(), provider: r.id }))
        .collect();
    rows.sort_by(|a, b| {
        (a.job.order(), a.provider.as_str()).cmp(&(b.job.order(), b.provider.as_str()))
    });
    rows
}

const HEAD: [&str; 9] =
    ["PROVIDER", "JOB", "STATE", "QUEUE", "NAME", "NODE", "ELAPSED", "LEFT", ""];

fn cut(name: &str) -> String {
    let units: Vec<u16> = name.encode_utf16().collect();
    if units.len() > 28 {
        format!("{}…", String::from_utf16_lossy(&units[..27]))
    } else {
        name.to_string()
    }
}

fn or_dash(text: &str) -> String {
    if text.is_empty() { "-".into() } else { text.to_string() }
}

fn job_table(rows: &[Row], s: &Style, held: &BTreeSet<String>) -> Vec<String> {
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|j| {
            let state = if j.job.state == JobState::Other {
                j.job.raw_state.to_lowercase()
            } else {
                j.job.state.as_str().into()
            };
            let held = j.provider == ClusterId::Dtu && held.contains(&j.job.id);
            vec![
                j.provider.short_name().into(),
                j.job.id.clone(),
                state,
                j.job.queue.clone(),
                cut(&j.job.name),
                or_dash(&j.job.node),
                or_dash(&j.job.elapsed),
                or_dash(&j.job.left),
                if held { "this laptop".into() } else { j.job.reason.clone() },
            ]
        })
        .collect();
    table(
        &cells,
        TableOptions {
            head: Some(HEAD.map(String::from).to_vec()),
            head_style: Box::new(|t| s.dim(t)),
            style: Box::new(|text, r, c| {
                if c == 2 {
                    match rows[r].job.state {
                        JobState::Running => s.green(text),
                        JobState::Pending => s.yellow(text),
                        _ => s.dim(text),
                    }
                } else if c == HEAD.len() - 1 {
                    s.dim(text)
                } else {
                    text.to_string()
                }
            }),
            ..Default::default()
        },
    )
}

pub fn render(
    results: &[ClusterJobs],
    s: &Style,
    held: &BTreeSet<String>,
    ngc: bool,
) -> Vec<String> {
    let rows = merge(results);
    let answered: Vec<&ClusterJobs> = results.iter().filter(|r| r.ok).collect();
    let mut lines = Vec::new();
    if !rows.is_empty() {
        let count =
            if rows.len() == 1 { "1 job".to_string() } else { format!("{} jobs", rows.len()) };
        lines.push(format!("{}  {}", s.bold("Your jobs on P1 compute"), s.dim(&count)));
        lines.extend(job_table(&rows, s, held));
    } else if !answered.is_empty() {
        let asked: Vec<&str> = answered.iter().map(|r| r.id.short_name()).collect();
        lines.push(format!(
            "{}  {}",
            s.bold("No jobs on P1 compute"),
            s.dim(&format!("asked {}", asked.join(" and ")))
        ));
    }
    trouble(results, s, ngc, NGC_NOTE, &mut lines);
    lines
}

fn trouble(results: &[ClusterJobs], s: &Style, ngc: bool, note: &str, lines: &mut Vec<String>) {
    let failed: Vec<&ClusterJobs> = results.iter().filter(|r| !r.ok).collect();
    if !failed.is_empty() {
        lines.push(String::new());
    }
    for c in failed {
        lines.push(format!(
            "{} {}: {}",
            s.symbol(Status::Fail),
            c.id.short_name(),
            c.error.as_deref().unwrap_or_default()
        ));
        lines.extend(c.hint.iter().flatten().map(|h| format!("    {}", s.dim(h))));
    }
    if ngc {
        lines.push(String::new());
        lines.push(format!(
            "{} {}",
            s.symbol(Status::Info),
            s.dim(&format!("{} is {}.", ClusterId::Ngc.name(), note))
        ));
    }
}

const GPU_HEAD: [&str; 5] = ["NODE", "GPU", "MODEL", "MEMORY USED", "JOBS"];

pub fn render_gpus(results: &[ClusterJobs], s: &Style, ngc: bool) -> Vec<String> {
    let answered: Vec<&ClusterJobs> = results.iter().filter(|r| r.ok).collect();
    let mut lines = Vec::new();
    if !answered.is_empty() {
        let asked: Vec<&str> = answered.iter().map(|r| r.id.short_name()).collect();
        lines.push(format!(
            "{}  {}",
            s.bold("GPUs on P1 compute"),
            s.dim(&format!("asked {}", asked.join(" and ")))
        ));
    }
    for r in &answered {
        for q in &r.queues {
            lines.push(String::new());
            lines.push(format!(
                "{}  {}",
                s.bold(&format!("{} {}", r.id.short_name(), q.queue)),
                s.dim(&headline(r.id, q))
            ));
            let cells: Vec<Vec<String>> = r
                .gpus
                .iter()
                .filter(|g| g.queue == q.queue)
                .map(|g| {
                    let jobs =
                        if g.jobs == 0 && !q.shared { "free".into() } else { g.jobs.to_string() };
                    vec![g.node.clone(), g.gpu.clone(), g.model.clone(), memory(&g.used), jobs]
                })
                .collect();
            if !cells.is_empty() {
                lines.extend(table(
                    &cells,
                    TableOptions {
                        right: &[1, 3, 4],
                        head: Some(GPU_HEAD.map(String::from).to_vec()),
                        head_style: Box::new(|t| s.dim(t)),
                        style: Box::new(|text, _, c| {
                            if c == GPU_HEAD.len() - 1 && (text == "free" || text == "0") {
                                s.green(text)
                            } else {
                                text.to_string()
                            }
                        }),
                        ..Default::default()
                    },
                ));
            }
            if let Some(advice) = advice(r.id, q) {
                lines.push(format!("  {}", s.dim(advice)));
            }
        }
    }
    trouble(results, s, ngc, NGC_GPUS, &mut lines);
    lines
}

async fn wanted(
    id: ClusterId,
    clusters: &[ClusterId],
    ctx: &Ctx,
    config: &Config,
    hosts: &Resolver,
) -> bool {
    if clusters.is_empty() { in_use(id, ctx, config, hosts).await } else { clusters.contains(&id) }
}

async fn query_if(
    id: ClusterId,
    chosen: &[ClusterId],
    ctx: &Ctx,
    hosts: &Resolver,
    what: What,
) -> Option<anyhow::Result<ClusterJobs>> {
    if chosen.contains(&id) { Some(query(ctx, id, hosts, what).await) } else { None }
}

fn asking(ctx: &Ctx, s: &Style, chosen: &[ClusterId], what: What) -> Animation {
    let names: Vec<&str> = chosen.iter().map(|id| id.short_name()).collect();
    let about = if what == What::Jobs { "for your jobs" } else { "what is free" };
    let text = format!("Asking {} {about}", names.join(" and "));
    let style = s.clone();
    Animation::start(&ctx.out, Duration::from_millis(80), move |area| {
        let frame = area.next_frame();
        area.draw(vec![format!("{} {text}", style.spinner(frame))]);
    })
}

pub async fn list(
    ctx: &Ctx,
    clusters: &[ClusterId],
    json: bool,
    what: What,
) -> anyhow::Result<i32> {
    let Some(config) = load_config_or_say(ctx) else { return Ok(2) };
    let s = make_style(&ctx.env, &ctx.out);
    let hosts = resolver(ctx, &config);

    let (dtu, gefion) = tokio::join!(
        wanted(ClusterId::Dtu, clusters, ctx, &config, &hosts),
        wanted(ClusterId::Gefion, clusters, ctx, &config, &hosts)
    );
    let chosen: Vec<ClusterId> = [(ClusterId::Dtu, dtu), (ClusterId::Gefion, gefion)]
        .into_iter()
        .filter_map(|(id, picked)| picked.then_some(id))
        .collect();
    let ngc =
        if clusters.is_empty() { config.ngc.is_some() } else { clusters.contains(&ClusterId::Ngc) };

    if chosen.is_empty() && !ngc {
        if json {
            let key = if what == What::Jobs { "jobs" } else { "gpus" };
            ctx.out.write(&format!("{{\n  \"{key}\": [],\n  \"providers\": []\n}}\n"));
        } else {
            ctx.out.write("No P1 provider is set up on this machine yet. aicentre check --all shows what each one needs.\n");
        }
        return Ok(0);
    }

    let live = (!json && !chosen.is_empty() && animates(&ctx.env, &ctx.out))
        .then(|| asking(ctx, &s, &chosen, what));
    let (a, b) = tokio::join!(
        query_if(ClusterId::Dtu, &chosen, ctx, &hosts, what),
        query_if(ClusterId::Gefion, &chosen, ctx, &hosts, what)
    );
    if let Some(live) = live {
        live.stop();
    }
    let mut results = [a, b].into_iter().flatten().collect::<anyhow::Result<Vec<_>>>()?;

    if ctx.interactive && !json {
        for r in &mut results {
            let Some(login) = r.login.clone() else { continue };
            ctx.out.write(&format!(
                "{login} asks for more than your key. Log in here; the login stays open for {}.\n",
                KEEP.replacen('m', " min", 1)
            ));
            let socket = shared_socket(ctx, &login)?;
            if log_in(ctx, &login, &socket).await.is_ok() {
                *r = query(ctx, r.id, &hosts, what).await?;
            }
        }
    }

    #[derive(Serialize)]
    struct Of<T> {
        provider: ClusterId,
        #[serde(flatten)]
        it: T,
    }
    let text = match (json, what) {
        (true, What::Gpus) => {
            let gpus: Vec<_> = results
                .iter()
                .flat_map(|r| r.gpus.iter().map(|it| Of { provider: r.id, it }))
                .collect();
            let queues: Vec<_> = results
                .iter()
                .flat_map(|r| r.queues.iter().map(|it| Of { provider: r.id, it }))
                .collect();
            let doc = serde_json::json!({ "gpus": gpus, "queues": queues, "providers": results });
            serde_json::to_string_pretty(&doc)? + "\n"
        }
        (true, What::Jobs) => {
            let doc = serde_json::json!({ "jobs": merge(&results), "providers": results });
            serde_json::to_string_pretty(&doc)? + "\n"
        }
        (false, What::Gpus) => lines(&render_gpus(&results, &s, ngc)),
        (false, What::Jobs) => {
            let held: BTreeSet<String> = live_notes(&ctx.paths).into_keys().collect();
            lines(&render(&results, &s, &held, ngc))
        }
    };
    ctx.out.write(&text);
    Ok(if results.iter().all(|r| r.ok) { 0 } else { 1 })
}

fn lines(lines: &[String]) -> String {
    lines.iter().map(|l| format!("{l}\n")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(id: &str, state: JobState) -> Job {
        Job { id: id.into(), state, ..Job::default() }
    }

    #[test]
    fn rows_run_first_then_by_id_then_provider() {
        let results = [
            ClusterJobs {
                jobs: vec![job("98765", JobState::Running), job("12", JobState::Pending)],
                ..ClusterJobs::new(ClusterId::Gefion)
            },
            ClusterJobs {
                jobs: vec![job("124", JobState::Pending), job("123", JobState::Running)],
                ..ClusterJobs::new(ClusterId::Dtu)
            },
        ];
        let rows = merge(&results);
        let order: Vec<&str> = rows.iter().map(|r| r.job.id.as_str()).collect();
        assert_eq!(order, ["123", "98765", "12", "124"]);
        assert_eq!(cut(&"x".repeat(29)), format!("{}…", "x".repeat(27)));
        assert_eq!(cut(&"x".repeat(28)), "x".repeat(28));
    }
}
