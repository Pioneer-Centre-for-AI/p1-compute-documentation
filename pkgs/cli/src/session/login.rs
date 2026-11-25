use std::future::Future;
use std::path::Path;

use super::here::here;
use super::sync::{
    Launcher, PATH_PROBE, Probe, VERSION_PROBE, ensure_launcher, launcher_probe, sync,
};
use crate::check::aliases::{Resolver, resolver};
use crate::check::clusters::{ClusterId, DTU, GEFION};
use crate::check::probes::connect::{classify_ssh_failure, parse_tags, provider_context};
use crate::check::select::in_use;
use crate::config::{Config, load_config_or_say};
use crate::context::Ctx;
use crate::exec::login_shell;
use crate::install::release::is_version;
use crate::remote::master::{KEEP, Master, is_open, shared_socket};
use crate::remote::shell::{LocalShell, Shell};
use crate::ui::say::fail;
use crate::ui::style::make_style;

#[derive(Clone)]
pub struct Login {
    pub provider: ClusterId,
    pub master: Shell,
    pub local: bool,
    pub alias: String,
    pub user: String,
    pub home: String,
    pub base: String,
    pub data: String,
    pub launcher: String,
    launches: String,
    there: String,
}

impl Login {
    pub fn path(&self, path: &str) -> String {
        remote_path(&self.home, &self.base, path)
    }

    pub async fn ready_to_launch(
        &mut self,
        ctx: &Ctx,
        config: &Config,
        step: &dyn Fn(&str),
    ) -> Result<(), (String, Vec<String>)> {
        if self.local {
            return Ok(());
        }
        let launcher = Launcher { runs: self.launches.clone(), there: self.there.clone() };
        let moved =
            ensure_launcher(ctx, config, &self.master, self.provider, &self.alias, &launcher, step)
                .await?;
        if let Some(path) = moved {
            self.launcher = path;
            self.launches = ctx.version.clone();
        }
        Ok(())
    }
}

const NO_PROJECT_DIR: &str = "Ask the compute coordinator to set up your P1 project directory.";
const NOT_SET_UP: &str = "no P1 provider is set up on this machine yet.";
const CHECK_ALL: &str = "aicentre check --all shows what each one needs.";

fn login_host(id: ClusterId) -> &'static str {
    if id == ClusterId::Gefion { GEFION.login } else { DTU.login }
}

pub fn not_reachable(id: ClusterId) -> Option<String> {
    (id == ClusterId::Ngc).then(|| {
        format!(
            "{} is air-gapped: you work on it through its remote desktop.",
            ClusterId::Ngc.name()
        )
    })
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice {
    pub asked: Option<ClusterId>,
    pub among: Option<Vec<ClusterId>>,
    pub outside: Option<String>,
    pub none: Option<Vec<String>>,
}

impl Choice {
    pub fn asked(asked: Option<ClusterId>) -> Choice {
        Choice { asked, ..Default::default() }
    }
}

pub async fn pick_provider(
    ctx: &Ctx,
    config: &Config,
    hosts: &Resolver,
    choice: &Choice,
) -> Result<ClusterId, (String, Vec<String>)> {
    let among = choice.among.clone().unwrap_or_else(|| vec![ClusterId::Dtu, ClusterId::Gefion]);
    if let Some(asked) = choice.asked {
        if let Some(why) = not_reachable(asked) {
            return Err((why, vec![]));
        }
        if !among.contains(&asked) {
            return Err((choice.outside.clone().unwrap_or_else(|| NOT_SET_UP.into()), vec![]));
        }
        return Ok(asked);
    }
    for id in among {
        if id != ClusterId::Ngc && in_use(id, ctx, config, hosts).await {
            return Ok(id);
        }
    }
    let mut none = choice.none.clone().unwrap_or_else(|| vec![NOT_SET_UP.into(), CHECK_ALL.into()]);
    let error = if none.is_empty() { String::new() } else { none.remove(0) };
    Err((error, none))
}

pub async fn with_login<F, Fut>(
    ctx: &Ctx,
    config: &Config,
    hosts: &Resolver,
    provider: ClusterId,
    body: F,
) -> anyhow::Result<i32>
where
    F: FnOnce(Login) -> Fut,
    Fut: Future<Output = anyhow::Result<i32>>,
{
    let Some(login) = open_login(ctx, config, provider, hosts).await? else { return Ok(1) };
    let shell = login.master.clone();
    let result = body(login).await;
    shell.close().await;
    result
}

pub async fn with_provider<F, Fut>(ctx: &Ctx, choice: &Choice, body: F) -> anyhow::Result<i32>
where
    F: FnOnce(Login, Config) -> Fut,
    Fut: Future<Output = anyhow::Result<i32>>,
{
    let Some(config) = load_config_or_say(ctx) else { return Ok(2) };
    let hosts = resolver(ctx, &config);
    let provider = match pick_provider(ctx, &config, &hosts, choice).await {
        Ok(p) => p,
        Err((error, hints)) => return Ok(fail(ctx, &error, &hints)),
    };
    let settings = config.clone();
    with_login(ctx, &config, &hosts, provider, move |login| body(login, settings)).await
}

fn local_login(ctx: &Ctx, provider: ClusterId) -> anyhow::Result<Option<Login>> {
    let user = ctx.env("USER").unwrap_or(&ctx.username).to_string();
    let home = ctx.paths.home.clone();
    let base = if provider == ClusterId::Dtu {
        format!("{}/{user}", DTU.project_dir)
    } else {
        home.clone()
    };
    if provider == ClusterId::Dtu && !Path::new(&base).exists() {
        fail(ctx, &format!("{base} does not exist."), &[NO_PROJECT_DIR.into()]);
        return Ok(None);
    }
    Ok(Some(Login {
        provider,
        master: Shell::Local(LocalShell::new(ctx)),
        local: true,
        alias: "localhost".into(),
        user,
        home,
        data: format!("{base}/.aicentre"),
        base,
        launcher: ctx.exec_path.clone(),
        launches: ctx.version.clone(),
        there: ctx.version.clone(),
    }))
}

fn setup_command(version: &str) -> String {
    let launcher = if is_version(version) { launcher_probe(version) } else { "true".into() };
    login_shell(
        &[
            r#"echo "@@USER $USER" && echo "@@HOME $HOME""#.to_string(),
            format!("{VERSION_PROBE} && {{ {PATH_PROBE}; }}"),
            format!(r#"{{ [ -d "{}/$USER" ] && echo "@@P1 yes"; true; }}"#, DTU.project_dir),
            format!("{{ {launcher}; }}"),
        ]
        .join(" && "),
    )
}

pub async fn open_login(
    ctx: &Ctx,
    config: &Config,
    provider: ClusterId,
    hosts: &Resolver,
) -> anyhow::Result<Option<Login>> {
    if here(ctx) == Some(provider) {
        return local_login(ctx, provider);
    }

    let Some(alias) = hosts.alias(provider).await else {
        fail(
            ctx,
            &format!(
                "no Host in ~/.ssh/config points at {} ({}).",
                login_host(provider),
                provider.name()
            ),
            &["aicentre check --all shows what to add.".into()],
        );
        return Ok(None);
    };

    let socket = shared_socket(ctx, &alias)?;
    if !is_open(ctx, &alias, &socket).await {
        ctx.out.write(&format!(
            "Logging in to {alias}. The login stays open for {}, so the next commands do not ask again.\n",
            KEEP.replacen('m', " min", 1)
        ));
    }
    let master = Master::new(&alias, &socket, ctx);
    if let Err(stderr) = master.start().await {
        let user = config.gefion.as_ref().and_then(|g| g.user.as_deref());
        let f = classify_ssh_failure(&stderr, &provider_context(provider, &alias, user, None));
        fail(ctx, &format!("cannot connect to {alias}: {}", f.message), &f.hint);
        master.close().await;
        return Ok(None);
    }

    let r = master.run(&setup_command(&ctx.version), 30_000).await;
    let tags = parse_tags(&r.stdout);
    let tag = |k: &str| tags.get(k).filter(|v| !v.is_empty()).cloned();
    let (user, home) = match (tag("USER"), tag("HOME")) {
        (Some(user), Some(home)) if r.code == 0 => (user, home),
        _ => {
            let why = match r.stderr.trim() {
                "" => format!("exit {}", r.code),
                said => said.to_string(),
            };
            fail(ctx, &format!("the login node did not answer as expected: {why}"), &[]);
            master.close().await;
            return Ok(None);
        }
    };
    if provider == ClusterId::Dtu && tag("P1").is_none() {
        fail(ctx, &format!("{}/{user} does not exist.", DTU.project_dir), &[NO_PROJECT_DIR.into()]);
        master.close().await;
        return Ok(None);
    }

    let probe = Probe {
        there: tags.get("AICENTRE").cloned().unwrap_or_default(),
        bin_on_path: tags.get("BINPATH").map(String::as_str) == Some("yes"),
        shell: tags.get("SHELL").cloned().unwrap_or_default(),
    };
    let shell = Shell::Master(master);
    sync(ctx, config, &shell, provider, &alias, &probe, &make_style(&ctx.env, &ctx.out)).await?;
    let base = if provider == ClusterId::Dtu {
        format!("{}/{user}", DTU.project_dir)
    } else {
        home.clone()
    };
    let own = ctx.env_set("AICENTRE_REMOTE_BIN").filter(|p| p.starts_with('/'));
    let (launcher, launches) = match own {
        Some(path) => (path.to_string(), ctx.version.clone()),
        None => (
            tag("LAUNCHER").filter(|p| p.starts_with('/')).unwrap_or_default(),
            tag("LAUNCHES").unwrap_or_default(),
        ),
    };
    Ok(Some(Login {
        provider,
        master: shell,
        local: false,
        alias,
        user,
        home,
        data: format!("{base}/.aicentre"),
        base,
        launcher,
        launches,
        there: probe.there.clone(),
    }))
}

pub fn remote_path(home: &str, base: &str, path: &str) -> String {
    if path.is_empty() {
        return base.to_string();
    }
    if path == "~" {
        return home.to_string();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return format!("{home}/{rest}");
    }
    if path.starts_with('/') {
        return path.to_string();
    }
    format!("{base}/{path}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_paths() {
        assert_eq!(remote_path("/h", "/dtu/p1/s1", ""), "/dtu/p1/s1");
        assert_eq!(remote_path("/h", "/dtu/p1/s1", "~"), "/h");
        assert_eq!(remote_path("/h", "/dtu/p1/s1", "~/x"), "/h/x");
        assert_eq!(remote_path("/h", "/dtu/p1/s1", "/abs"), "/abs");
        assert_eq!(remote_path("/h", "/dtu/p1/s1", "rel/a"), "/dtu/p1/s1/rel/a");
        assert_eq!(not_reachable(ClusterId::Dtu), None);
        assert!(
            setup_command("0.1.0")
                .starts_with(r#"bash -lc 'echo "@@USER $USER" && echo "@@HOME $HOME" && echo"#)
        );
    }
}
