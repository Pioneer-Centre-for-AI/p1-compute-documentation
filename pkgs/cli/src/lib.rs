pub mod browser;
pub mod check;
pub mod cli;
pub mod config;
pub mod context;
pub mod exec;
pub mod gallery;
pub mod http;
pub mod install;
pub mod jobs;
pub mod launch;
pub mod mcp;
pub mod paths;
pub mod platform;
pub mod remote;
pub mod seal;
pub mod session;
pub mod transfer;
pub mod ui;
pub mod unpack;
pub mod version;

use std::future::Future;
use std::pin::Pin;

pub use context::Ctx;

use cli::args::{Command, parse_command};
use cli::help::render_help;
use jobs::list::What;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub fn lock<T>(m: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

pub async fn run(argv: &[String], ctx: &Ctx) -> i32 {
    let loud = |a: &String| a == "--verbose";
    let own =
        argv.first().is_some_and(|a| ["proxy", "launch", "unpack-example"].contains(&a.as_str()));
    let argv: Vec<String> = if !own && argv.iter().any(loud) {
        exec::set_verbose(true);
        argv.iter().filter(|a| !loud(a)).cloned().collect()
    } else {
        argv.to_vec()
    };
    let command = match parse_command(&argv) {
        Ok(c) => c,
        Err(e) => {
            let said = e.to_string();
            let own = if said.starts_with("aicentre ") { "" } else { "aicentre: " };
            ctx.err.write(&format!("{own}{said}\nRun aicentre help for usage.\n"));
            return 2;
        }
    };
    match dispatch(command, ctx).await {
        Ok(code) => code,
        Err(e) => {
            ctx.err.write(&format!("aicentre: {e}\n"));
            1
        }
    }
}

async fn dispatch(command: Command, ctx: &Ctx) -> anyhow::Result<i32> {
    Ok(match command {
        Command::Help { topic } => {
            let style = ui::style::make_style(&ctx.env, &ctx.out);
            ctx.out.write(&render_help(topic.as_deref(), &style, &ctx.version));
            0
        }
        Command::Version => {
            ctx.out.write(&format!("{}\n", ctx.version));
            0
        }
        Command::Check { clusters, connect, json } => {
            check::command::check(ctx, &clusters, connect, json).await?
        }
        Command::Upgrade { version, keep_current } => {
            install::upgrade::upgrade(ctx, version.as_deref(), keep_current).await?
        }
        Command::Uninstall { purge } => install::uninstall::uninstall(ctx, purge).await?,
        Command::SelfInstall { keep_current } => {
            install::self_install::self_install(ctx, keep_current).await?
        }
        Command::Launch { launch } => launch::launch(ctx, &launch).await?,
        Command::UnpackExample { url, slug, parent } => {
            gallery::gallery::unpack_example(ctx, &url, &slug, &parent).await?
        }
        Command::Session { session } => session::command::session(ctx, &session).await?,
        Command::Mcp => mcp::server::mcp(ctx).await?,
        Command::Proxy { host } => session::proxy::proxy(ctx, &host).await?,
        Command::Transfer { direction, from, to, provider } => {
            transfer::transfer::transfer(ctx, direction, &from, to.as_deref(), provider).await?
        }
        Command::Gallery { gallery } => gallery::gallery::gallery(ctx, &gallery).await?,
        Command::List { what, json, .. } if what == "sizes" => session::sizes::list(ctx, json),
        Command::List { what, clusters, json } => {
            let what = if what == "gpus" { What::Gpus } else { What::Jobs };
            jobs::list::list(ctx, &clusters, json, what).await?
        }
    })
}
