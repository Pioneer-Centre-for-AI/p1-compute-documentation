use std::path::Path;

use super::clusters::{ClusterId, DTU, DTU_NO_ANSWER_HINT, GPU_HOST, host_block};
use super::env::{CheckEnv, block_hint, cli_there, expander, reached};
use super::probes::connect::provider_context;
use super::probes::files::{private_key_problem, stat_or_null};
use super::probes::knownhosts::known_host;
use super::probes::net::reach_timeout;
use super::probes::ssh::{SshConfig, is_configured, port_of};
use super::types::{Group, Outcome, Status};
use crate::exec::{RunOptions, run};
use crate::paths::tilde;
use crate::session::editor::editor_installed;
use crate::session::proxy::{include_hint, is_gpu_host, ssh_config_file, write_ssh_config};
use crate::session::sync::VERSION_PROBE;

fn placeholder(user: &str) -> bool {
    user.contains('<') || user.contains('>')
}

pub async fn check_dtu(g: &mut Group, c: &CheckEnv<'_>) {
    let alias = c.alias(ClusterId::Dtu).await.unwrap_or_else(|| DTU.alias.to_string());
    let want_user = c.config.dtu.as_ref().and_then(|d| d.user.clone());
    let home = &c.ctx.paths.home;

    if !c.machine.ssh_usable {
        g.skip("host", "Host in ~/.ssh/config", "needs a working ssh").await;
        return;
    }

    let mut cfg: Option<SshConfig> = None;
    let host = g
        .step("host", "Host in ~/.ssh/config", async {
            let conf = match c.ssh_g(&alias).await {
                Err(error) => return Ok(Outcome::fail(error)),
                Ok(conf) => conf,
            };
            if !is_configured(&conf, &alias) {
                return Ok(Outcome::fail(format!("none points at {}", DTU.login)).hint(
                    block_hint(
                        "Add this to ~/.ssh/config. The Host name is yours to choose:",
                        &host_block(ClusterId::Dtu, &alias, want_user.as_deref()),
                    ),
                ));
            }
            let hostname = conf.get("hostname").unwrap_or_default().to_string();
            cfg = Some(conf);
            if hostname != DTU.login {
                return Ok(Outcome::warn(format!("points at {hostname}; P1 uses {}", DTU.login)));
            }
            Ok(Outcome::ok(format!("{alias} → {hostname}")))
        })
        .await;
    let Some(conf) = cfg.filter(|_| host.status != Status::Fail) else { return };
    let hostname = conf.get("hostname").unwrap_or(DTU.login).to_string();
    let port = port_of(&conf);
    let user = conf.get("user").unwrap_or_default().to_string();
    let expand = expander(c, Some(&user), Some(&hostname));

    g.step("user", "User", async {
        if placeholder(&user) {
            return Ok(Outcome::fail(format!("still the placeholder {user}"))
                .hint(vec![format!("Set User to your DTU username in Host {alias}.")]));
        }
        if let Some(want) = want_user.as_deref()
            && user != want
        {
            return Ok(Outcome::fail(format!("{user}, but config.toml says {want}"))
                .hint(vec![format!("Set \"User {want}\" in Host {alias}, or fix config.toml.")]));
        }
        if want_user.is_none() && user == c.ctx.username {
            return Ok(Outcome::ok(format!("{user}, the same name as on this machine")));
        }
        Ok(Outcome::ok(user.clone()))
    })
    .await;

    g.step("key", "SSH key", async {
        let keys: Vec<String> = conf
            .all("identityfile")
            .iter()
            .map(|p| expand(p))
            .filter(|p| Path::new(p).exists())
            .collect();
        for key in &keys {
            if let Some(problem) =
                stat_or_null(key).and_then(|st| private_key_problem(&st, c.ctx.uid))
            {
                return Ok(Outcome::fail(format!("{}: {problem}", tilde(key, home)))
                    .hint(vec![format!("chmod 600 {}", tilde(key, home))]));
            }
        }
        let agent = run("ssh-add", &["-l"], RunOptions::new(&c.ctx.env).timeout_ms(5_000)).await;
        let agent_keys = if agent.code == 0 {
            agent.stdout.trim().split('\n').filter(|l| !l.is_empty()).count()
        } else {
            0
        };
        if keys.is_empty() && agent_keys == 0 {
            return Ok(Outcome::fail("no key found").hint(vec![
                "Make one and copy it to the login node:".into(),
                "  ssh-keygen -t ed25519".into(),
                format!("  ssh-copy-id {alias}"),
            ]));
        }
        let mut parts: Vec<String> = keys.iter().map(|k| tilde(k, home)).collect();
        if agent_keys > 0 {
            parts.push(format!("{agent_keys} in the agent"));
        }
        Ok(Outcome::ok(parts.join(", ")))
    })
    .await;

    g.step("known-host", "Host key", async {
        let mut files = conf.words("userknownhostsfile");
        files.extend(conf.words("globalknownhostsfile"));
        let files: Vec<String> = files.iter().map(|f| expand(f)).collect();
        if let Some(found) = known_host(&hostname, port, &files, &c.ctx.env).await {
            return Ok(Outcome::ok(format!("known, in {}", tilde(&found, home))));
        }
        Ok(Outcome::warn("not seen yet, so the first ssh asks you to confirm it")
            .hint(vec![format!("ssh {alias}")]))
    })
    .await;

    if editor_installed(c.ctx, c.config) && !user.is_empty() && !placeholder(&user) {
        g.step("gpu-host", "VS Code on a GPU node", async {
            if !c.ctx.env.contains_key(crate::mcp::tools::READ_ONLY) {
                write_ssh_config(c.ctx, &user, Some(&conf))?;
            }
            let res = c.ssh_g(GPU_HOST).await;
            let file = tilde(&ssh_config_file(&c.ctx.paths), home);
            if res.as_ref().is_ok_and(is_gpu_host) {
                return Ok(Outcome::ok(format!("~/.ssh/config includes {file}")));
            }
            Ok(Outcome::info("needs one Include line in ~/.ssh/config")
                .hint(include_hint(&c.ctx.paths)))
        })
        .await;
    }

    let proxied = match conf.get("proxyjump") {
        Some(jump) => !jump.is_empty(),
        None => conf.get("proxycommand").is_some_and(|p| !p.is_empty()),
    };
    if !proxied {
        let reach = g
            .step("reach", &format!("Network path to {hostname}"), async {
                let r = (c.ctx.tcp)(hostname.clone(), port, reach_timeout(c.connect)).await;
                Ok(c.reach(r, port, DTU_NO_ANSWER_HINT))
            })
            .await;
        if reach.status != Status::Ok {
            if c.connect {
                g.skip("login", "Login", "needs the network path").await;
            }
            return;
        }
    }

    if !c.connect {
        return;
    }

    let mut p1: Option<String> = None;
    let mut there: Option<String> = None;
    let login = g
        .step("login", "Login", async {
            let script = format!(
                r#"printf "@@HOST %s\n" "$(hostname)"; d={}/$USER; [ -d "$d" ] && printf "@@P1 %s\n" "$d"; {VERSION_PROBE}; true"#,
                DTU.project_dir
            );
            let context = provider_context(ClusterId::Dtu, &alias, None, Some(&hostname));
            Ok(match c.login(&script, &context).await? {
                Err(outcome) => outcome,
                Ok(tags) => {
                    p1 = tags.get("P1").cloned();
                    there = tags.get("AICENTRE").cloned();
                    reached(&tags)
                }
            })
        })
        .await;
    if login.status != Status::Ok {
        return;
    }

    g.step("project", "Project directory", async {
        Ok(match p1.filter(|p| !p.is_empty()) {
            Some(p) => Outcome::ok(p),
            None => Outcome::fail(format!("{}/{user} does not exist", DTU.project_dir)).hint(vec![
                "Ask the compute coordinator to set up your P1 project directory.".into(),
            ]),
        })
    })
    .await;
    let version = c.ctx.version.clone();
    g.step("cli", "aicentre there", async move { Ok(cli_there(&version, there.as_deref())) }).await;
}
