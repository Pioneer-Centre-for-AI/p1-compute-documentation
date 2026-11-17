use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::check::clusters::ClusterId;
use crate::config::Config;
use crate::context::Ctx;
use crate::exec::{login_shell, shq};
use crate::install::layout::iso_now;
use crate::install::release::{is_version, version_order};
use crate::install::self_install::{DEFAULT_BIN, path_line};
use crate::paths::{join, state_dir};
use crate::remote::shell::Shell;
use crate::session::state::write_private;
use crate::ui::style::Style;

pub const INSTALLER: &str = include_str!("../../install.sh");

pub const VERSION_PROBE: &str =
    r#"echo "@@AICENTRE $("$HOME/.local/bin/aicentre" --version 2>/dev/null)""#;

pub const PATH_PROBE: &str = r#"case ":$PATH:" in *":$HOME/.local/bin:"*) echo "@@BINPATH yes";; esac; echo "@@SHELL ${SHELL##*/}""#;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Probe {
    pub there: String,
    pub bin_on_path: bool,
    pub shell: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncAction {
    None,
    Install,
    NewerThere,
    Off,
}

pub fn decide(ctx: &Ctx, config: &Config, provider: ClusterId, there: &str) -> SyncAction {
    if ctx.env("AICENTRE_SYNC") == Some("0")
        || config.cluster(provider).and_then(|c| c.sync) == Some(false)
    {
        return SyncAction::Off;
    }
    if !is_version(&ctx.version) || ctx.version.contains("-dev") {
        return SyncAction::Off;
    }
    if !is_version(there) {
        return SyncAction::Install;
    }
    match version_order(there, &ctx.version) {
        std::cmp::Ordering::Less => SyncAction::Install,
        std::cmp::Ordering::Greater => SyncAction::NewerThere,
        std::cmp::Ordering::Equal => SyncAction::None,
    }
}

pub fn sync_command(version: &str, provider: ClusterId) -> String {
    let install = format!(
        r#"AICENTRE_VERSION={version} sh "$0/install.sh" >"$0/sync.log" 2>&1 && printf '%s\n' {provider} > "${{XDG_DATA_HOME:-$HOME/.local/share}}/aicentre/provider"; rm -f "$0/install.sh"; rmdir "$0/sync.lock""#
    );
    [
        r#"s="${XDG_STATE_HOME:-$HOME/.local/state}/aicentre""#.to_string(),
        r#"mkdir -p "$s""#.to_string(),
        r#"find "$s" -maxdepth 1 -name sync.lock -mmin +30 -exec rmdir {} \; 2>/dev/null"#
            .to_string(),
        format!(
            r#"if mkdir "$s/sync.lock" 2>/dev/null; then if cat >"$s/install.sh"; then nohup sh -c {} "$s" >/dev/null 2>&1 </dev/null & else rmdir "$s/sync.lock"; fi; else cat >/dev/null; fi"#,
            shq(&install)
        ),
        "true".to_string(),
    ]
    .join("; ")
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Remote {
    pub version: String,
    pub at: String,
    pub alias: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_hinted: Option<bool>,
}

pub type Remotes = BTreeMap<ClusterId, Remote>;

fn remotes_file(ctx: &Ctx) -> String {
    join(&[&ctx.paths.state, "remotes.json"])
}

pub fn read_remotes(ctx: &Ctx) -> Remotes {
    std::fs::read_to_string(remotes_file(ctx))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

const RETRY_HOURS: i64 = 24;

fn tried_lately(ctx: &Ctx, provider: ClusterId) -> bool {
    let remotes = read_remotes(ctx);
    let Some(remote) = remotes.get(&provider).filter(|r| r.version == ctx.version) else {
        return false;
    };
    remote.at.parse::<jiff::Timestamp>().is_ok_and(|at| {
        let hours = jiff::Timestamp::now().duration_since(at).as_hours();
        (0..RETRY_HOURS).contains(&hours)
    })
}

enum Patch {
    Installed,
    PathHinted,
}

fn remember(ctx: &Ctx, provider: ClusterId, alias: &str, patch: Patch) -> anyhow::Result<()> {
    let mut remotes = read_remotes(ctx);
    let remote = remotes.entry(provider).or_insert_with(|| Remote {
        version: ctx.version.clone(),
        at: iso_now(),
        alias: String::new(),
        path_hinted: None,
    });
    remote.alias = alias.to_string();
    match patch {
        Patch::Installed => {
            remote.version = ctx.version.clone();
            remote.at = iso_now();
        }
        Patch::PathHinted => remote.path_hinted = Some(true),
    }
    state_dir(&ctx.paths, &[])?;
    write_private(&remotes_file(ctx), &format!("{}\n", serde_json::to_string_pretty(&remotes)?))?;
    Ok(())
}

pub async fn sync(
    ctx: &Ctx,
    config: &Config,
    shell: &Shell,
    provider: ClusterId,
    alias: &str,
    probe: &Probe,
    style: &Style,
) -> anyhow::Result<()> {
    let there = probe.there.as_str();
    let action = decide(ctx, config, provider, there);
    if action == SyncAction::Off {
        return Ok(());
    }
    if action == SyncAction::NewerThere {
        ctx.out.write(&style.dim(&format!(
            "  {alias} has aicentre {there}, newer than this one ({}); aicentre upgrade updates it here.\n",
            ctx.version
        )));
    }
    if action == SyncAction::Install && tried_lately(ctx, provider) {
        return Ok(());
    }
    if action == SyncAction::Install {
        let command = login_shell(&sync_command(&ctx.version, provider));
        let r = shell.run_input(&command, INSTALLER, 15_000).await;
        if r.code != 0 {
            return Ok(());
        }
        remember(ctx, provider, alias, Patch::Installed)?;
        let what = if there.is_empty() {
            format!("Installing aicentre {}", ctx.version)
        } else {
            format!("Updating aicentre {there} → {}", ctx.version)
        };
        ctx.out.write(
            &style.dim(&format!("  {what} on {alias} in the background, so it is there too.\n")),
        );
    }

    let present = action == SyncAction::Install || !there.is_empty();
    let hinted = read_remotes(ctx).get(&provider).and_then(|r| r.path_hinted) == Some(true);
    if present && !probe.bin_on_path && !hinted {
        ctx.out.write(&style.dim(&format!(
            "  ~/.local/bin is not on your PATH on {alias}, so call it as ~/.local/bin/aicentre there, or run this once on {alias}:\n"
        )));
        ctx.out.write(&format!("      {}\n", style.cyan(&path_line(&probe.shell, DEFAULT_BIN))));
        remember(ctx, provider, alias, Patch::PathHinted)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::check::probes::net::NetResult;
    use crate::config::ClusterSettings;
    use crate::context::{Env, Output};
    use crate::paths::resolve_paths;
    use crate::platform::current_platform;

    fn ctx(version: &str, env: &[(&str, &str)]) -> Ctx {
        let mut env: Env = env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        env.insert("HOME".into(), "/h".into());
        let (out, _) = Output::capture(false, None);
        Ctx {
            paths: resolve_paths(&env, 1).unwrap(),
            env,
            uid: 1,
            username: "tester".into(),
            err: out.clone(),
            out,
            interactive: false,
            exec_path: "aicentre".into(),
            compiled: true,
            version: version.into(),
            platform: current_platform(),
            tcp: Arc::new(|_, _, _| Box::pin(async { NetResult::Ok })),
        }
    }

    #[test]
    fn installed_when_missing_or_older_never_downgraded_never_from_a_dev_build() {
        let none = Config::default();
        let dtu = ClusterId::Dtu;
        assert_eq!(decide(&ctx("0.2.0", &[]), &none, dtu, ""), SyncAction::Install);
        assert_eq!(decide(&ctx("0.2.0", &[]), &none, dtu, "0.1.0"), SyncAction::Install);
        assert_eq!(decide(&ctx("0.2.0", &[]), &none, dtu, "0.2.0"), SyncAction::None);
        assert_eq!(decide(&ctx("0.2.0", &[]), &none, dtu, "0.3.0"), SyncAction::NewerThere);
        assert_eq!(decide(&ctx("0.0.0-dev", &[]), &none, dtu, ""), SyncAction::Off);
        let off = Config {
            dtu: Some(ClusterSettings { sync: Some(false), ..Default::default() }),
            ..Default::default()
        };
        assert_eq!(decide(&ctx("0.2.0", &[]), &off, dtu, ""), SyncAction::Off);
        let env = [("AICENTRE_SYNC", "0")];
        assert_eq!(decide(&ctx("0.2.0", &env), &none, ClusterId::Gefion, ""), SyncAction::Off);
    }

    #[test]
    fn an_install_that_did_not_take_is_not_tried_again_the_same_day() {
        let home = tempfile::tempdir().unwrap();
        let at = |version: &str| {
            let mut c = ctx(version, &[]);
            let env: Env =
                [("HOME".to_string(), home.path().to_string_lossy().into_owned())].into();
            c.paths = resolve_paths(&env, 1).unwrap();
            c
        };
        let c = at("0.2.0-rc.1");
        assert!(!tried_lately(&c, ClusterId::Dtu));
        remember(&c, ClusterId::Dtu, "dtu", Patch::Installed).unwrap();
        assert!(tried_lately(&c, ClusterId::Dtu));
        assert!(!tried_lately(&c, ClusterId::Gefion));
        assert!(!tried_lately(&at("0.2.0"), ClusterId::Dtu));
        let old = std::fs::read_to_string(remotes_file(&c)).unwrap();
        let at_line = old.lines().find(|l| l.contains("\"at\"")).unwrap();
        let stale = old.replace(at_line, "    \"at\": \"2026-01-01T00:00:00.000Z\",");
        std::fs::write(remotes_file(&c), stale).unwrap();
        assert!(!tried_lately(&c, ClusterId::Dtu));
    }

    #[test]
    fn the_install_runs_in_the_background_under_a_lock() {
        let cmd = sync_command("0.2.0", ClusterId::Gefion);
        assert!(
            cmd.starts_with(
                r#"s="${XDG_STATE_HOME:-$HOME/.local/state}/aicentre"; mkdir -p "$s"; "#
            )
        );
        assert!(cmd.contains(r#"then if cat >"$s/install.sh"; then nohup sh -c 'AICENTRE_VERSION=0.2.0 sh "$0/install.sh" >"$0/sync.log" 2>&1 && printf '\''%s\n'\'' gefion > "${XDG_DATA_HOME"#));
        assert!(cmd.ends_with(
            r#"rmdir "$0/sync.lock"' "$s" >/dev/null 2>&1 </dev/null & else rmdir "$s/sync.lock"; fi; else cat >/dev/null; fi; true"#
        ));
        assert!(!cmd.contains("curl"));
        assert!(INSTALLER.starts_with("#!/bin/sh"));
    }

    #[test]
    fn the_command_runs_the_script_from_stdin_once_then_frees_the_lock() {
        use std::io::Write;
        let home = tempfile::tempdir().unwrap();
        let state = home.path().join(".local/state/aicentre");
        let run = |script: &str| {
            let mut child = std::process::Command::new("bash")
                .args(["-c", &sync_command("0.2.0", ClusterId::Dtu)])
                .env_clear()
                .env("PATH", std::env::var("PATH").unwrap_or_default())
                .env("HOME", home.path())
                .stdin(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(script.as_bytes()).unwrap();
            assert!(child.wait().unwrap().success());
        };
        run("echo \"ran $AICENTRE_VERSION\"; mkdir -p \"$HOME/.local/share/aicentre\"\n");
        let done = home.path().join(".local/share/aicentre/provider");
        for _ in 0..50 {
            if done.exists() && !state.join("sync.lock").exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert_eq!(std::fs::read_to_string(&done).unwrap(), "dtu\n");
        assert_eq!(std::fs::read_to_string(state.join("sync.log")).unwrap(), "ran 0.2.0\n");
        assert!(!state.join("install.sh").exists() && !state.join("sync.lock").exists());
    }
}
