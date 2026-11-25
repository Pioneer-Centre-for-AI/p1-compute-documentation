use std::sync::LazyLock;

use regex::Regex;

use crate::check::clusters::ClusterId;
use crate::check::types::Status;
use crate::cli::args::Direction;
use crate::context::Ctx;
use crate::exec::{RunOptions, attach, login_shell, run, shq, which};
use crate::remote::master::shared_socket;
use crate::session::login::{Choice, Login, not_reachable, with_provider};
use crate::ui::say::fail;
use crate::ui::style::make_style;

static VERSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"rsync\s+version\s+([0-9]+)\.([0-9]+)").unwrap());

pub fn rsync_version(version_text: &str) -> Option<(u64, u64)> {
    let m = VERSION.captures(version_text)?;
    Some((m[1].parse().ok()?, m[2].parse().ok()?))
}

fn protects_args(version: Option<(u64, u64)>) -> bool {
    version.is_some_and(|v| v >= (3, 1))
}

pub fn rsync_flags(version_text: &str) -> Vec<String> {
    let flags: &[&str] = if protects_args(rsync_version(version_text)) {
        &["-a", "--partial", "--protect-args", "--info=progress2"]
    } else {
        &["-a", "--partial", "--progress"]
    };
    flags.iter().map(|f| f.to_string()).collect()
}

/// rsync before 3.4.0 lets the sending side write outside the folder it
/// copies into (CVE-2024-12087) and read files it was not sent
/// (CVE-2024-12086). On a pull the sending side is the cluster.
fn trusts_the_sender(version: Option<(u64, u64)>) -> bool {
    version.is_some_and(|v| v < (3, 4))
}

fn local(path: &str) -> String {
    let first = path.split('/').next().unwrap_or("");
    if first.contains(':') { format!("./{path}") } else { path.to_string() }
}

pub async fn transfer(
    ctx: &Ctx,
    direction: Direction,
    from: &str,
    to: Option<&str>,
    provider: Option<ClusterId>,
) -> anyhow::Result<i32> {
    if which("rsync", &ctx.env).is_none() {
        return Ok(fail(
            ctx,
            "rsync is not installed on this machine.",
            &["Install it with your package manager; macOS ships one.".into()],
        ));
    }
    if let Some(why) = provider.and_then(not_reachable) {
        return Ok(fail(
            ctx,
            &format!("{why} Its staging SFTP endpoint is in the gallery's XFR-DAT example."),
            &[],
        ));
    }

    with_provider(ctx, &Choice::asked(provider), |login, _config| async move {
        if login.local {
            return Ok(fail(
                ctx,
                &format!(
                    "you are on {} already; copy with cp or rsync here.",
                    login.provider.name()
                ),
                &[],
            ));
        }
        copy(ctx, &login, direction, from, to).await
    })
    .await
}

pub async fn copy(
    ctx: &Ctx,
    login: &Login,
    direction: Direction,
    from: &str,
    to: Option<&str>,
) -> anyhow::Result<i32> {
    let version = run("rsync", &["--version"], RunOptions::new(&ctx.env).timeout_ms(5_000)).await;
    let found = rsync_version(&version.stdout);
    // An rsync without --protect-args hands the remote path to the
    // cluster's shell, so the path is quoted for it.
    let remote = |path: &str| {
        if protects_args(found) { path.to_string() } else { shq(path) }
    };
    let rsh = format!("ssh -S {} -o ControlMaster=no", shq(&shared_socket(ctx, &login.alias)?));

    let (src, dst) = match direction {
        Direction::Push => {
            let into = format!("{}/", login.path(to.unwrap_or("")).trim_end_matches('/'));
            let made =
                login.master.run(&login_shell(&format!("mkdir -p {}", shq(&into))), 15_000).await;
            if made.code != 0 {
                return Ok(fail(
                    ctx,
                    &format!("cannot create {into} on {}: {}", login.alias, made.stderr.trim()),
                    &[],
                ));
            }
            (local(from.trim_end_matches('/')), format!("{}:{}", login.alias, remote(&into)))
        }
        Direction::Pull => {
            (format!("{}:{}", login.alias, remote(&login.path(from))), local(to.unwrap_or(".")))
        }
    };

    let s = make_style(&ctx.env, &ctx.out);
    ctx.out.write(&format!("{}\n", s.dim(&format!("{src} → {dst}"))));
    let mut args = rsync_flags(&version.stdout);
    if direction == Direction::Pull && found.is_some() {
        args.push("--safe-links".into());
    }
    if direction == Direction::Pull && trusts_the_sender(found) {
        ctx.out.write(&format!(
            "{} This rsync is older than 3.4.0, which trusts the cluster more than it should (CVE-2024-12087). Update rsync when you can.\n",
            s.symbol(Status::Warn)
        ));
    }
    args.extend(["-e".into(), rsh, "--".into(), src, dst.clone()]);
    let code = attach("rsync", &args, &ctx.env).await;
    ctx.out.write(&if code == 0 {
        format!("{} Copied to {dst}\n", s.symbol(Status::Ok))
    } else {
        format!(
            "{} rsync stopped (exit {code}); run it again to pick up where it left off.\n",
            s.symbol(Status::Fail)
        )
    });
    Ok(if code == 0 { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsync_flags_fit_the_rsync_on_this_machine() {
        assert_eq!(
            rsync_flags("rsync  version 3.4.1  protocol version 32"),
            ["-a", "--partial", "--protect-args", "--info=progress2"]
        );
        let old = ["-a", "--partial", "--progress"];
        assert_eq!(rsync_flags("rsync  version 2.6.9  protocol version 29"), old);
        assert_eq!(rsync_flags("openrsync: protocol version 29"), old);
        assert_eq!(rsync_version("rsync  version 3.4.1  protocol version 32"), Some((3, 4)));
        assert_eq!(rsync_version("openrsync: protocol version 29"), None);
        assert!(trusts_the_sender(Some((3, 2))) && trusts_the_sender(Some((2, 6))));
        assert!(!trusts_the_sender(Some((3, 4))) && !trusts_the_sender(None));
        assert_eq!(local("a:b/c"), "./a:b/c");
        assert_eq!(local("dir/a:b"), "dir/a:b");
    }
}
