use std::sync::LazyLock;

use regex::Regex;

use crate::check::clusters::ClusterId;
use crate::check::types::Status;
use crate::cli::args::Direction;
use crate::context::Ctx;
use crate::exec::{RunOptions, attach, login_shell, run, shq, which};
use crate::remote::master::shared_socket;
use crate::session::login::{Choice, not_reachable, with_provider};
use crate::ui::say::fail;
use crate::ui::style::make_style;

static VERSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"rsync\s+version\s+([0-9]+)\.([0-9]+)").unwrap());

pub fn rsync_flags(version_text: &str) -> Vec<String> {
    let modern = VERSION.captures(version_text).is_some_and(|m| {
        let major: u64 = m[1].parse().unwrap_or(u64::MAX);
        let minor: u64 = m[2].parse().unwrap_or(u64::MAX);
        major > 3 || (major == 3 && minor >= 1)
    });
    let flags: &[&str] = if modern {
        &["-a", "--partial", "--protect-args", "--info=progress2"]
    } else {
        &["-a", "--partial", "--progress"]
    };
    flags.iter().map(|f| f.to_string()).collect()
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
        let version =
            run("rsync", &["--version"], RunOptions::new(&ctx.env).timeout_ms(5_000)).await;
        let rsh = format!("ssh -S {} -o ControlMaster=no", shq(&shared_socket(ctx, &login.alias)?));

        let (src, dst) = match direction {
            Direction::Push => {
                let into = format!("{}/", login.path(to.unwrap_or("")).trim_end_matches('/'));
                let made = login
                    .master
                    .run(&login_shell(&format!("mkdir -p {}", shq(&into))), 15_000)
                    .await;
                if made.code != 0 {
                    return Ok(fail(
                        ctx,
                        &format!("cannot create {into} on {}: {}", login.alias, made.stderr.trim()),
                        &[],
                    ));
                }
                (from.trim_end_matches('/').to_string(), format!("{}:{into}", login.alias))
            }
            Direction::Pull => {
                (format!("{}:{}", login.alias, login.path(from)), to.unwrap_or(".").to_string())
            }
        };

        let s = make_style(&ctx.env, &ctx.out);
        ctx.out.write(&format!("{}\n", s.dim(&format!("{src} → {dst}"))));
        let mut args = rsync_flags(&version.stdout);
        args.extend(["-e".into(), rsh, src, dst.clone()]);
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
    })
    .await
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
    }
}
