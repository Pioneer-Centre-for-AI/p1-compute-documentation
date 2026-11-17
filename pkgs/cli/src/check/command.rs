use std::sync::Arc;

use tokio::signal::unix::{SignalKind, signal};

use super::clusters::ClusterId;
use super::run::run_checks;
use super::types::Sink;
use crate::config::load_config_or_say;
use crate::context::Ctx;
use crate::exec::kill_all;
use crate::ui::report::{JsonReporter, LiveReporter, PlainReporter, Reporter, summarize};
use crate::ui::style::make_style;

pub async fn check(
    ctx: &Ctx,
    clusters: &[ClusterId],
    connect: bool,
    json: bool,
) -> anyhow::Result<i32> {
    let Some(config) = load_config_or_say(ctx) else { return Ok(2) };

    let style = make_style(&ctx.env, &ctx.out);
    let live = !json && ctx.out.is_tty && ctx.env("TERM") != Some("dumb");
    let reporter: Arc<dyn Reporter> = if json {
        Arc::new(JsonReporter::new(&ctx.out, &ctx.version, connect))
    } else if live {
        Arc::new(LiveReporter::new(style.clone(), &ctx.out))
    } else {
        Arc::new(PlainReporter::new(style.clone(), &ctx.out))
    };

    let sink: Arc<dyn Sink> = reporter.clone();
    let checks = run_checks(ctx, &config, clusters, connect, sink);
    let reports = match signal(SignalKind::interrupt()) {
        Ok(mut interrupt) => tokio::select! {
            reports = checks => reports,
            _ = interrupt.recv() => {
                reporter.abort();
                kill_all();
                return Ok(130);
            }
        },
        Err(_) => checks.await,
    };
    let summary = summarize(&reports, &style, connect);
    reporter.finish(&reports, &summary);
    Ok(if summary.ok { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use crate::context::Ctx;
    use crate::session::login::{Choice, with_provider};

    #[allow(dead_code)]
    fn runs_in_a_task(ctx: &'static Ctx) -> impl Send {
        async move {
            super::check(ctx, &[], true, false).await.ok();
            crate::session::proxy::proxy(ctx, "aicentre-gpu").await.ok();
            with_provider(ctx, &Choice::default(), |login, _| async move {
                Ok(login.master.run("true", 1_000).await.code)
            })
            .await
        }
    }
}
