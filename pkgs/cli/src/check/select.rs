use super::aliases::Resolver;
use super::clusters::{ClusterId, GEFION};
use super::probes::tsh::{has_profile, tsh_home};
use crate::config::Config;
use crate::context::Ctx;
use crate::session::here::here;

pub async fn in_use(id: ClusterId, ctx: &Ctx, config: &Config, r: &Resolver) -> bool {
    if config.cluster(id).is_some() || here(ctx) == Some(id) {
        return true;
    }
    if id == ClusterId::Gefion && has_profile(&tsh_home(&ctx.env, &ctx.paths.home), GEFION.proxy) {
        return true;
    }
    r.alias(id).await.is_some()
}
