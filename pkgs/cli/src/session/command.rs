use std::sync::Arc;

use tokio::signal::unix::{SignalKind, signal};
use tokio_util::sync::CancellationToken;

use crate::browser::{free_port, open_url};
use crate::context::Ctx;
use crate::session::editor::open_folder;
use crate::session::interactive::{InteractiveRequest, SessionDeps, interactive};
use crate::session::lsf::App;

pub async fn session(ctx: &Ctx, request: &InteractiveRequest) -> anyhow::Result<i32> {
    let cancel = CancellationToken::new();
    let mut int = signal(SignalKind::interrupt())?;
    let mut term = signal(SignalKind::terminate())?;
    let mut hup = signal(SignalKind::hangup())?;
    let shell = request.app == App::Shell;
    let token = cancel.clone();
    let watch = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = int.recv() => {}
                _ = term.recv() => {}
                _ = hup.recv() => {}
            }
            if shell {
                continue;
            }
            if token.is_cancelled() {
                std::process::exit(130);
            }
            token.cancel();
        }
    });
    let (url_ctx, folder_ctx) = (ctx.clone(), ctx.clone());
    let deps = SessionDeps {
        cancel,
        open_url: Arc::new(move |url| {
            let ctx = url_ctx.clone();
            Box::pin(async move { open_url(&ctx, &url).await })
        }),
        open_folder: Arc::new(move |argv| {
            let ctx = folder_ctx.clone();
            Box::pin(async move { open_folder(&ctx, &argv).await })
        }),
        free_port: Arc::new(|| Box::pin(free_port())),
    };
    let code = interactive(ctx, request, &deps).await;
    watch.abort();
    code
}
