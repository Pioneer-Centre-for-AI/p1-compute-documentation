use std::net::TcpListener;
use std::process::{Command, Stdio};

use crate::context::Ctx;
use crate::exec::which;
use crate::platform::Os;

pub fn has_display(ctx: &Ctx) -> bool {
    ctx.platform.os == Os::Darwin
        || ctx.env_set("DISPLAY").is_some()
        || ctx.env_set("WAYLAND_DISPLAY").is_some()
}

pub async fn open_url(ctx: &Ctx, url: &str) {
    let cmd = if ctx.platform.os == Os::Darwin {
        "open"
    } else if has_display(ctx) {
        "xdg-open"
    } else {
        return;
    };
    let Some(path) = which(cmd, &ctx.env) else { return };
    let _ = spawn_detached(&path, &[url], ctx);
}

pub fn spawn_detached(cmd: &str, args: &[&str], ctx: &Ctx) -> std::io::Result<std::process::Child> {
    use std::os::unix::process::CommandExt;
    let mut c = Command::new(cmd);
    c.args(args)
        .env_clear()
        .envs(&ctx.env)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // SAFETY: setsid is async-signal-safe and runs in the child before exec.
    unsafe {
        c.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    c.spawn()
}

pub async fn free_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

pub fn link(url: &str, tty: bool) -> String {
    if tty { format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\") } else { url.to_string() }
}
