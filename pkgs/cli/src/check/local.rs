use std::os::unix::fs::MetadataExt;

use super::env::Machine;
use super::probes::files::{config_problem, ssh_dir_problem, stat_or_null};
use super::probes::ssh::{at_least, ssh_g, ssh_version};
use super::types::{Group, Outcome};
use crate::context::Ctx;
use crate::paths::join;
use crate::platform::Os;

pub async fn check_local(g: &mut Group, ctx: &Ctx) -> Machine {
    let mut machine = Machine::default();
    let ssh_dir = join(&[&ctx.paths.home, ".ssh"]);
    let ssh_config = join(&[&ssh_dir, "config"]);

    g.step("ssh", "OpenSSH client", async {
        let Some(v) = ssh_version(&ctx.env).await else {
            let fix = if ctx.platform.os == Os::Darwin {
                "macOS ships OpenSSH; check that /usr/bin is on your PATH."
            } else {
                "Install OpenSSH with your package manager (openssh-client or openssh)."
            };
            return Ok(
                Outcome::fail("ssh is not installed, or is not OpenSSH").hint(vec![fix.into()])
            );
        };
        machine.ssh = Some(v.clone());
        if !at_least(&v, 6, 8) {
            return Ok(Outcome::fail(format!(
                "OpenSSH {} is too old; aicentre needs 6.8 or newer",
                v.text
            ))
            .hint(vec!["Update OpenSSH with your package manager.".into()]));
        }
        machine.ssh_usable = true;
        Ok(Outcome::ok(format!("OpenSSH {}", v.text)))
    })
    .await;

    g.step("ssh-dir", "~/.ssh", async {
        let Some(st) = stat_or_null(&ssh_dir) else {
            return Ok(Outcome::warn("does not exist yet").hint(vec!["mkdir -m 700 ~/.ssh".into()]));
        };
        if let Some(problem) = ssh_dir_problem(&st, ctx.uid) {
            return Ok(Outcome::warn(problem).hint(vec!["chmod 700 ~/.ssh".into()]));
        }
        Ok(Outcome::ok("private to you"))
    })
    .await;

    let usable = machine.ssh_usable;
    g.step("ssh-config", "~/.ssh/config", async {
        let Some(st) = stat_or_null(&ssh_config) else {
            return Ok(Outcome::warn(
                "does not exist yet; each provider below shows what to put in it",
            ));
        };
        if let Some(problem) = config_problem(&st, ctx.uid) {
            let fix = if st.uid() == ctx.uid || st.uid() == 0 {
                "chmod 600 ~/.ssh/config".to_string()
            } else {
                let who = if ctx.username.is_empty() { "$USER" } else { &ctx.username };
                format!("chown {who} ~/.ssh/config")
            };
            let detail = format!("{problem}, so ssh refuses to read it");
            return Ok(Outcome::fail(detail).hint(vec![fix]));
        }
        if !usable {
            return Ok(Outcome::skip("needs a working ssh to read"));
        }
        match ssh_g("aicentre-check.invalid", &ctx.env).await {
            Err(error) => Ok(Outcome::fail("ssh cannot read it")
                .hint(error.split('\n').map(str::to_string).collect())),
            Ok(_) => Ok(Outcome::ok("ssh reads it")),
        }
    })
    .await;

    machine
}
