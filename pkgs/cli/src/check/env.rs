use super::aliases::Resolver;
use super::clusters::ClusterId;
use super::probes::connect::{FailureContext, Tags, ssh_probe};
use super::probes::net::NetResult;
use super::probes::ssh::{SshGResult, SshVersion, Tokens, expand_path};
use super::types::{Outcome, Status};
use crate::config::Config;
use crate::context::Ctx;
use crate::remote::master::{is_open, shared_socket};

#[derive(Clone, Debug, Default)]
pub struct Machine {
    pub ssh: Option<SshVersion>,
    pub ssh_usable: bool,
}

pub struct CheckEnv<'a> {
    pub ctx: &'a Ctx,
    pub config: &'a Config,
    pub connect: bool,
    pub explicit: bool,
    pub machine: Machine,
    pub resolver: Resolver,
}

impl CheckEnv<'_> {
    pub async fn ssh_g(&self, alias: &str) -> SshGResult {
        self.resolver.ssh_g(alias).await
    }

    pub async fn alias(&self, id: ClusterId) -> Option<String> {
        self.resolver.alias(id).await
    }

    pub async fn login(
        &self,
        script: &str,
        context: &FailureContext,
    ) -> anyhow::Result<Result<Tags, Outcome>> {
        let socket = shared_socket(self.ctx, &context.alias)?;
        let socket = is_open(self.ctx, &context.alias, &socket).await.then_some(socket);
        Ok(ssh_probe(self.machine.ssh.as_ref(), script, context, &self.ctx.env, socket.as_deref())
            .await
            .map_err(|f| Outcome::new(f.status).detail(f.message).hint(f.hint)))
    }

    pub fn reach(&self, r: NetResult, port: u16, hint: &str) -> Outcome {
        match r {
            NetResult::Ok => Outcome::ok(format!("port {port} answers")),
            NetResult::Failed { message, .. } => {
                Outcome::new(if self.connect { Status::Fail } else { Status::Warn })
                    .detail(message)
                    .hint(vec![hint.into()])
            }
        }
    }
}

pub fn expander(
    c: &CheckEnv,
    remote_user: Option<&str>,
    host: Option<&str>,
) -> impl Fn(&str) -> String + use<> {
    let t = Tokens {
        home: c.ctx.paths.home.clone(),
        local_user: c.ctx.username.clone(),
        remote_user: remote_user.map(str::to_string),
        host: host.map(str::to_string),
        uid: Some(c.ctx.uid),
    };
    move |path| expand_path(path, &t)
}

pub fn block_hint(intro: &str, block: &[String]) -> Vec<String> {
    let mut lines = vec![intro.to_string(), String::new()];
    lines.extend(block.iter().map(|line| format!("  {line}")));
    lines
}

pub fn reached(tags: &Tags) -> Outcome {
    Outcome::ok(format!("reached {}", tags.get("HOST").map_or("", String::as_str)))
}

pub fn cli_there(version: &str, there: Option<&str>) -> Outcome {
    match there.filter(|t| !t.is_empty()) {
        None => Outcome::info("not installed there yet; the next session or job installs it"),
        Some(t) if t == version => Outcome::ok(format!("{t}, the same as here")),
        Some(t) => Outcome::info(format!(
            "{t} there, {version} here; the next session or job brings it in line"
        )),
    }
}
