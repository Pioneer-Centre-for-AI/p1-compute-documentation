use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::check::clusters::ClusterId;
use crate::config::Config;
use crate::context::Ctx;
use crate::exec::{login_shell, shq};
use crate::install::layout::iso_now;
use base64::Engine;

use crate::check::probes::connect::parse_tags;
use crate::install::release::{is_version, sha256_hex, version_order};
use crate::install::self_install::{DEFAULT_BIN, path_line};
use crate::paths::{join, state_dir};
use crate::remote::shell::Shell;
use crate::session::state::write_private;
use crate::ui::say::printable;
use crate::ui::style::Style;

pub const INSTALLER: &str = include_str!("../../install.sh");

pub const VERSION_PROBE: &str =
    r#"echo "@@AICENTRE $("$HOME/.local/bin/aicentre" --version 2>/dev/null)""#;

pub fn launcher_probe(version: &str) -> String {
    format!(
        r#"l="${{XDG_DATA_HOME:-$HOME/.local/share}}/aicentre/versions/{version}/aicentre"; echo "@@LAUNCHER $l"; echo "@@LAUNCHES $("$l" --version 2>/dev/null)""#
    )
}

pub const PATH_PROBE: &str = r#"case ":$PATH:" in *":$HOME/.local/bin:"*) echo "@@BINPATH yes";; esac; echo "@@SHELL ${SHELL##*/}""#;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Probe {
    pub there: String,
    pub bin_on_path: bool,
    pub shell: String,
}

const LOCK_SECONDS: u32 = 60;

pub fn install_command(
    version: &str,
    provider: ClusterId,
    keep_current: bool,
    own: bool,
) -> String {
    let keep = if keep_current { "AICENTRE_KEEP_CURRENT=1 " } else { "" };
    let itself = match (own, keep_current) {
        (false, _) => String::new(),
        (true, false) => {
            format!(r#""$HOME/.local/bin/aicentre" upgrade --version {version} || "#)
        }
        (true, true) => {
            format!(r#""$HOME/.local/bin/aicentre" upgrade --version {version} --keep-current || "#)
        }
    };
    let note = if keep_current {
        String::new()
    } else {
        format!(
            r#" && printf '%s\n' {provider} > "${{XDG_DATA_HOME:-$HOME/.local/share}}/aicentre/provider""#
        )
    };
    [
        "umask 077".to_string(),
        r#"s="${XDG_STATE_HOME:-$HOME/.local/state}/aicentre""#.to_string(),
        r#"mkdir -p "$s""#.to_string(),
        r#"l="${XDG_DATA_HOME:-$HOME/.local/share}/aicentre/versions/"#.to_string()
            + version
            + r#"/aicentre""#,
        "n=0".to_string(),
        format!(
            r#"until mkdir "$s/sync.lock" 2>/dev/null; do find "$s" -maxdepth 1 -name sync.lock -mmin +30 -exec rmdir {{}} \; 2>/dev/null; n=$((n+1)); if [ "$n" -gt {LOCK_SECONDS} ]; then cat >/dev/null; echo "@@LOG another install holds $s/sync.lock"; exit 0; fi; sleep 1; done"#
        ),
        format!(
            r#"if [ "$("$l" --version 2>/dev/null)" = {version} ]; then cat >/dev/null; else cat >"$s/install.sh" && {{ {{ {itself}{keep}AICENTRE_VERSION={version} sh "$s/install.sh"; }} >"$s/sync.log" 2>&1{note}; tail -n 4 "$s/sync.log" | sed 's/^/@@LOG /'; }}; rm -f "$s/install.sh"; fi"#
        ),
        r#"rmdir "$s/sync.lock""#.to_string(),
        r#"echo "@@LAUNCHES $("$l" --version 2>/dev/null)""#.to_string(),
    ]
    .join("; ")
}

pub struct Launcher {
    pub runs: String,
    pub there: String,
}

const INSTALL: &str = "curl -fsSL https://hpc.aicentre.dk/install";

/// The first version whose `upgrade` checks the release signature, takes
/// `--keep-current`, and survives a network home, where the old version's
/// folder cannot be removed. An older one is replaced by the installer.
const UPGRADES_ITSELF_FROM: &str = "0.2.0-rc.2";

pub fn upgrades_itself(there: &str) -> bool {
    is_version(there) && version_order(there, UPGRADES_ITSELF_FROM) != std::cmp::Ordering::Less
}

fn is_dev_build(ctx: &Ctx) -> bool {
    !ctx.compiled || !is_version(&ctx.version) || ctx.version.contains("-dev")
}

fn dev_dir(version: &str) -> String {
    let base = version.strip_suffix("-dev").unwrap_or(version);
    let name = if is_version(base) { format!("{base}-dev") } else { "dev".to_string() };
    format!(r#""${{XDG_DATA_HOME:-$HOME/.local/share}}/aicentre/versions/{name}""#)
}

const CLUSTER_TARGET: &str = "x86_64-unknown-linux-musl";

fn own_cluster_build(ctx: &Ctx) -> Option<(String, bool)> {
    let running = std::path::Path::new(&ctx.exec_path);
    let target = running.parent()?.parent()?;
    let built = target.join(CLUSTER_TARGET).join("release/aicentre");
    let at = std::fs::metadata(&built).and_then(|m| m.modified()).ok()?;
    let older = newest_in(&target.parent()?.join("src")).is_some_and(|source| at < source);
    Some((built.to_string_lossy().into_owned(), older))
}

fn newest_in(dir: &std::path::Path) -> Option<std::time::SystemTime> {
    let mut newest = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let at = if path.is_dir() {
            newest_in(&path)
        } else {
            entry.metadata().and_then(|m| m.modified()).ok()
        };
        newest = newest.max(at);
    }
    newest
}

async fn push_dev_build(
    ctx: &Ctx,
    shell: &Shell,
    alias: &str,
    local: &str,
    step: &dyn Fn(&str),
) -> Result<String, (String, Vec<String>)> {
    let name = "AICENTRE_PUSH_BIN";
    let bytes =
        std::fs::read(local).map_err(|e| (format!("cannot read {name}={local}: {e}"), vec![]))?;
    if !bytes.starts_with(b"\x7fELF") {
        let hint = "It must be a static Linux build for the cluster's processor.";
        return Err((format!("{name}={local} is not a Linux program."), vec![hint.into()]));
    }
    let dir = dev_dir(&ctx.version);
    let sum = sha256_hex(&bytes);
    let report = r#"echo "@@LAUNCHER $d/aicentre"; echo "@@LAUNCHES $("$d/aicentre" --version 2>/dev/null)""#;
    let have = format!(
        r#"d={dir}; echo "@@SUM $(sha256sum "$d/aicentre" 2>/dev/null | cut -d" " -f1)"; {report}"#
    );
    let mut r = shell.run(&login_shell(&have), 30_000).await;
    if parse_tags(&r.stdout).get("SUM").map(String::as_str) != Some(sum.as_str()) {
        step(&format!("Copying this development build to {alias}"));
        let put = format!(
            r#"umask 077; d={dir}; mkdir -p "$d" && base64 -d > "$d/aicentre.new" && chmod 700 "$d/aicentre.new" && mv "$d/aicentre.new" "$d/aicentre"; {report}"#
        );
        let text = base64::engine::general_purpose::STANDARD.encode(&bytes);
        r = shell.run_input(&login_shell(&put), &text, 600_000).await;
    }
    let tags = parse_tags(&r.stdout);
    let runs = tags.get("LAUNCHES").cloned().unwrap_or_default();
    let path = tags.get("LAUNCHER").filter(|p| p.starts_with('/')).cloned();
    match path {
        Some(path) if runs == ctx.version => Ok(path),
        _ if runs.is_empty() => Err((
            format!("the build in {name} does not run on {alias}."),
            vec!["It must be a static Linux build for the cluster's processor.".into()],
        )),
        _ => Err((
            format!("the build in {name} is aicentre {runs}, and this one is {}.", ctx.version),
            vec!["A session needs the same version at both ends.".into()],
        )),
    }
}

pub async fn ensure_launcher(
    ctx: &Ctx,
    config: &Config,
    shell: &Shell,
    provider: ClusterId,
    alias: &str,
    launcher: &Launcher,
    step: &dyn Fn(&str),
) -> Result<Option<String>, (String, Vec<String>)> {
    let version = &ctx.version;
    // A build from a checkout is not a release, whatever its version says.
    if is_dev_build(ctx) {
        if let Some(local) = ctx.env_set("AICENTRE_PUSH_BIN") {
            return push_dev_build(ctx, shell, alias, local, step).await.map(Some);
        }
        if ctx.env_set("AICENTRE_REMOTE_BIN").is_none()
            && let Some((local, older)) = own_cluster_build(ctx)
        {
            if older {
                step(
                    "The source changed after the cluster's build of this checkout was made; nix run .#static-x64 makes a new one",
                );
            }
            return push_dev_build(ctx, shell, alias, &local, step).await.map(Some);
        }
    }
    if launcher.runs == *version {
        if !ctx.compiled && ctx.env_set("AICENTRE_REMOTE_BIN").is_none() {
            // Same version, but not this checkout's code.
            step(&format!(
                "Using the released aicentre {version} on {alias}; nix run .#static-x64 builds this checkout's for it"
            ));
        }
        return Ok(None);
    }
    let action = decide(ctx, config, provider, &launcher.there);
    if is_dev_build(ctx) {
        return Err((
            format!("this is a development build ({version}), which {alias} cannot download."),
            vec![
                "A session runs this same version on the cluster. Build it for the".into(),
                "cluster, and the next session copies it there:".into(),
                "  nix run .#static-x64".into(),
                "AICENTRE_PUSH_BIN names another build on this machine, and".into(),
                "AICENTRE_REMOTE_BIN one that is on the cluster already.".into(),
            ],
        ));
    }
    if action == SyncAction::Off {
        return Err((
            format!("a session needs aicentre {version} on {alias}, and sync is off."),
            vec![format!("On {alias}, run: {INSTALL} | AICENTRE_VERSION={version} sh")],
        ));
    }
    step(&format!("Installing aicentre {version} on {alias}"));
    let keep = action == SyncAction::NewerThere;
    let own = upgrades_itself(&launcher.there);
    let command = login_shell(&install_command(version, provider, keep, own));
    let r = shell.run_input(&command, INSTALLER, (u64::from(LOCK_SECONDS) + 240) * 1000).await;
    let runs = r.stdout.lines().find_map(|l| l.trim().strip_prefix("@@LAUNCHES ")).unwrap_or("");
    if printable(runs) == *version {
        let _ = remember(ctx, provider, alias, Patch::Installed);
        return Ok(None);
    }
    let mut hints: Vec<String> =
        r.stdout.lines().filter_map(|l| l.trim().strip_prefix("@@LOG ")).map(printable).collect();
    if hints.is_empty() {
        let last = r.stderr.trim().lines().next_back().unwrap_or("the login node did not answer");
        hints.push(printable(last));
    }
    if hints.iter().any(|h| h.contains("error: 404") || h.contains("HTTP 404")) {
        hints.push(format!(
            "aicentre {version} has no release yet, so there is nothing to download."
        ));
    }
    hints.push(format!("The installer's log on {alias} is ~/.local/state/aicentre/sync.log."));
    Err((format!("cannot install aicentre {version} on {alias}, which a session needs."), hints))
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
        r#"umask 077; mkdir -p "$s""#.to_string(),
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
        assert!(cmd.starts_with(
            r#"s="${XDG_STATE_HOME:-$HOME/.local/state}/aicentre"; umask 077; mkdir -p "$s"; "#
        ));
        assert!(cmd.contains(r#"then if cat >"$s/install.sh"; then nohup sh -c 'AICENTRE_VERSION=0.2.0 sh "$0/install.sh" >"$0/sync.log" 2>&1 && printf '\''%s\n'\'' gefion > "${XDG_DATA_HOME"#));
        assert!(cmd.ends_with(
            r#"rmdir "$0/sync.lock"' "$s" >/dev/null 2>&1 </dev/null & else rmdir "$s/sync.lock"; fi; else cat >/dev/null; fi; true"#
        ));
        assert!(!cmd.contains("curl"));
        assert!(INSTALLER.starts_with("#!/bin/sh"));
    }

    #[test]
    fn a_session_waits_for_the_install_and_hears_what_went_wrong() {
        use std::io::Write;
        let home = tempfile::tempdir().unwrap();
        let run = |keep: bool, script: &str| {
            let mut child = std::process::Command::new("bash")
                .args(["-c", &install_command("0.2.0", ClusterId::Dtu, keep, false)])
                .env_clear()
                .env("PATH", std::env::var("PATH").unwrap_or_default())
                .env("HOME", home.path())
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(script.as_bytes()).unwrap();
            let out = child.wait_with_output().unwrap();
            assert!(out.status.success());
            String::from_utf8_lossy(&out.stdout).into_owned()
        };
        let state = home.path().join(".local/state/aicentre");
        let data = home.path().join(".local/share/aicentre");

        let said = run(false, "echo 'aicentre install: no release'; exit 1\n");
        assert_eq!(said, "@@LOG aicentre install: no release\n@@LAUNCHES \n");
        assert!(!state.join("sync.lock").exists() && !state.join("install.sh").exists());
        assert!(!data.join("provider").exists());

        let install = r#"d="$HOME/.local/share/aicentre/versions/$AICENTRE_VERSION"; mkdir -p "$d"
printf '#!/bin/sh\necho %s\n' "$AICENTRE_VERSION" > "$d/aicentre"; chmod +x "$d/aicentre"
echo "kept=${AICENTRE_KEEP_CURRENT:-no}"
"#;
        let said = run(true, install);
        assert_eq!(said, "@@LOG kept=1\n@@LAUNCHES 0.2.0\n");
        assert!(!data.join("provider").exists());

        std::fs::remove_dir_all(data.join("versions")).unwrap();
        assert_eq!(run(false, install), "@@LOG kept=no\n@@LAUNCHES 0.2.0\n");
        assert_eq!(std::fs::read_to_string(data.join("provider")).unwrap(), "dtu\n");
        assert_eq!(run(false, "exit 1\n"), "@@LAUNCHES 0.2.0\n");
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&state).unwrap().permissions().mode() & 0o077, 0);

        assert!(!upgrades_itself("") && !upgrades_itself("0.1.2"));
        assert!(!upgrades_itself("0.2.0-rc.1"));
        assert!(upgrades_itself("0.2.0-rc.2") && upgrades_itself("0.3.0"));
        let own = install_command("0.2.0", ClusterId::Dtu, true, true);
        assert!(own.contains(
            r#"{ { "$HOME/.local/bin/aicentre" upgrade --version 0.2.0 --keep-current || AICENTRE_KEEP_CURRENT=1 AICENTRE_VERSION=0.2.0 sh "$s/install.sh"; } >"$s/sync.log" 2>&1;"#
        ), "{own}");
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
