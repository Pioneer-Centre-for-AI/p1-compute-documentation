use std::sync::Arc;

use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::BoxFuture;
use crate::browser::has_display;
use crate::check::aliases::resolver;
use crate::check::clusters::{ClusterId, GPU_HOST};
use crate::check::probes::connect::parse_tags;
use crate::check::probes::ssh::ssh_g;
use crate::check::select::in_use;
use crate::check::types::Status;
use crate::cli::args::{Direction, SIZES};
use crate::config::{Config, load_config, load_config_or_say};
use crate::context::Ctx;
use crate::exec::shq;
use crate::session::editor::{Editor, editor_installed, find_editor, remote_args, split_words};
use crate::session::here::here;
use crate::session::login::{Choice, Login, not_reachable, with_provider};
use crate::session::lsf::{
    App, QUEUE, Request, Server, View, bsub_command, gigabytes, launcher_args, login_shell,
    parse_minutes,
};
use crate::session::proxy::{gpu_host_ready, include_hint, write_ssh_config};
use crate::session::serve::serve;
use crate::session::sizes;
use crate::session::state::live_notes;
use crate::transfer::transfer::copy;
use crate::ui::prompt::confirm_yes;
use crate::ui::say::{fail, printable};
use crate::ui::style::make_style;

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
    #[serde(skip_serializing_if = "one")]
    pub gpus: u32,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub ask_size: bool,
    pub browser: bool,
    pub view: View,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub another: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub yes: bool,
}

fn one(n: &u32) -> bool {
    *n == 1
}

pub fn asks_for(r: &InteractiveRequest) -> String {
    let gpus = if r.gpus == 1 { "1 GPU".to_string() } else { format!("{} GPUs", r.gpus) };
    let memory = gigabytes(&r.mem_per_cpu).unwrap_or(0.0) * f64::from(r.cpus);
    let time = if r.minutes.is_multiple_of(60) {
        format!("{} h", r.minutes / 60)
    } else {
        format!("{} min", r.minutes)
    };
    format!("{gpus}, {} cores, {memory:.0} GB, {time}", r.cpus)
}

pub type OpenUrl = Arc<dyn Fn(String) -> BoxFuture<'static, ()> + Send + Sync>;
pub type OpenFolder = Arc<dyn Fn(Vec<String>) -> BoxFuture<'static, Option<String>> + Send + Sync>;
pub type Choose =
    Arc<dyn Fn(String, Vec<String>, usize) -> BoxFuture<'static, Option<usize>> + Send + Sync>;
pub type FreePort = Arc<dyn Fn() -> BoxFuture<'static, std::io::Result<u16>> + Send + Sync>;

#[derive(Clone)]
pub struct SessionDeps {
    pub cancel: CancellationToken,
    pub open_url: OpenUrl,
    pub open_folder: OpenFolder,
    pub free_port: FreePort,
    pub choose: Choose,
}

async fn offers_gpu(ctx: &Ctx, r: &InteractiveRequest, config: &Config) -> bool {
    match r.provider {
        Some(id) => id == ClusterId::Dtu,
        None => {
            here(ctx).is_none() && in_use(ClusterId::Dtu, ctx, config, &resolver(ctx, config)).await
        }
    }
}

async fn sized(
    ctx: &Ctx,
    r: &InteractiveRequest,
    deps: &SessionDeps,
) -> Option<InteractiveRequest> {
    let config = load_config(&ctx.paths.config_file).unwrap_or_default();
    let asks = r.ask_size
        && ctx.interactive
        && config.session.as_ref().and_then(|s| s.ask) != Some(false)
        && (r.gpu || offers_gpu(ctx, r, &config).await);
    if !asks {
        return Some(r.clone());
    }
    let with_login = !r.gpu;
    let title = if with_login { "Where to run" } else { "Size of the GPU session" };
    let title = format!("{title}  (↑↓ to move, Enter to start, q to leave)");
    let picked = (deps.choose)(title, sizes::choices(with_login), 0).await?;
    let style = make_style(&ctx.env, &ctx.out);
    let next = |flags: &str| {
        ctx.out.write(&style.dim(&format!("Next time: aicentre {} {flags}\n", r.app.as_str())));
    };
    if with_login && picked == 0 {
        next("--login");
        return Some(InteractiveRequest { ask_size: false, ..r.clone() });
    }
    let size = SIZES.get(picked - usize::from(with_login))?;
    next(&format!("--gpu --size {}", size.name));
    Some(InteractiveRequest {
        gpu: true,
        gpus: size.gpus,
        minutes: parse_minutes(size.time)?,
        mem_per_cpu: size.mem_per_cpu.to_string(),
        ask_size: false,
        ..r.clone()
    })
}

pub(super) fn app_name(app: App) -> &'static str {
    match app {
        App::Shell => "Shell",
        App::Marimo => "marimo",
        App::Jupyter => "JupyterLab",
        App::Code => "VS Code",
    }
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

pub(super) fn session_command(login: &Login, r: &InteractiveRequest, bsub_mode: &str) -> String {
    if r.gpu {
        let request = Request {
            app: r.app,
            path: r.path.clone(),
            server: r.server,
            view: Some(r.view),
            minutes: r.minutes,
            cpus: r.cpus,
            mem_per_cpu: r.mem_per_cpu.clone(),
            gpus: r.gpus,
            approved: r.yes,
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
        r.yes,
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

    let Some(r) = sized(ctx, r, deps).await else {
        ctx.out.write("Nothing was started.\n");
        return Ok(130);
    };
    let r = &r;

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
    with_provider(ctx, &Choice::asked(asked), |login, config| {
        logged_in(ctx, r, deps, login, config, editor)
    })
    .await
}

async fn logged_in(
    ctx: &Ctx,
    r: &InteractiveRequest,
    deps: &SessionDeps,
    mut login: Login,
    config: Config,
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
    if !bring_local(ctx, &login, &req, &r.path).await? {
        return Ok(1);
    }
    let installing = |text: &str| ctx.out.write(&format!("{text}...\n"));
    if let Err((error, hints)) = login.ready_to_launch(ctx, &config, &installing).await {
        return Ok(fail(ctx, &error, &hints));
    }
    let req = match agreed_to_installs(ctx, &login, req).await {
        Ok(req) => req,
        Err(code) => return Ok(code),
    };
    if r.app != App::Shell {
        return serve(ctx, &login, &req, deps, editor).await;
    }
    if r.gpu {
        ctx.out.write(&format!(
            "Asking {QUEUE} for {}. The prompt appears on the GPU node; exit ends the job.\n",
            asks_for(r)
        ));
    }
    Ok(login.master.attach(&session_command(&login, &req, "-Is")).await)
}

/// A relative path that is a file here and not on the cluster is most
/// often a notebook the user means to open there.
async fn bring_local(
    ctx: &Ctx,
    login: &Login,
    req: &InteractiveRequest,
    given: &str,
) -> anyhow::Result<bool> {
    let relative = !given.is_empty() && !given.starts_with(['/', '~']);
    if login.local || !relative || !ctx.interactive || !std::path::Path::new(given).exists() {
        return Ok(true);
    }
    let there = login.master.run(&format!("test -e {}", shq(&req.path)), 15_000).await;
    if there.code != 1 {
        return Ok(true);
    }
    let what = format!("{given} is here and not on {}", login.alias);
    if !confirm_yes(ctx, &format!("{what}. Copy it to {}?", req.path)).await {
        return Ok(true);
    }
    let (folder, _) = given.trim_end_matches('/').rsplit_once('/').unwrap_or(("", given));
    if copy(ctx, login, Direction::Push, given, Some(folder)).await? != 0 {
        return Ok(false);
    }
    let style = make_style(&ctx.env, &ctx.out);
    let back = if folder.is_empty() { "." } else { folder };
    ctx.out.write(&style.dim(&format!(
        "Changes stay on the cluster. To bring them back: aicentre pull {given} {back}\n"
    )));
    Ok(true)
}

async fn agreed_to_installs(
    ctx: &Ctx,
    login: &Login,
    req: InteractiveRequest,
) -> Result<InteractiveRequest, i32> {
    if req.yes {
        return Ok(req);
    }
    let app = if req.view == View::Desktop { "sshd" } else { req.app.as_str() };
    let words = [
        login.launcher.as_str(),
        "launch",
        "--login",
        "--needs",
        app,
        &req.path,
        req.server.as_str(),
        &login.data,
        &login.base,
        "127.0.0.1",
    ];
    let command = words.iter().map(|w| shq(w)).collect::<Vec<_>>().join(" ");
    let asked = login.master.run(&command, 30_000).await;
    let needs: Vec<String> = asked
        .stdout
        .lines()
        .filter_map(|l| l.trim().strip_prefix("@@NEED "))
        .map(printable)
        .collect();
    if needs.is_empty() {
        return Ok(req);
    }
    let list: String = needs.iter().map(|n| format!("  {n}\n")).collect();
    let what = format!("This session needs software on {} that is not aicentre's own", login.alias);
    if !ctx.interactive {
        let mut hints = needs.clone();
        hints.push("Add --yes to install it.".into());
        return Err(fail(ctx, &format!("{what}, and nobody is here to ask."), &hints));
    }
    ctx.out.write(&format!("{what}.\nIt goes into {}:\n{list}", login.data));
    if !confirm_yes(ctx, "Install it?").await {
        ctx.out.write("Nothing was installed.\n");
        return Err(1);
    }
    Ok(InteractiveRequest { yes: true, ..req })
}
