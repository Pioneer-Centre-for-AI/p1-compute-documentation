use std::sync::Arc;

use super::aliases::resolver;
use super::clusters::{CLUSTER_IDS, ClusterId};
use super::dtu::check_dtu;
use super::env::CheckEnv;
use super::gefion::check_gefion;
use super::local::check_local;
use super::ngc::check_ngc;
use super::select::in_use;
use super::types::{Group, GroupId, GroupReport, Outcome, Sink};
use crate::config::Config;
use crate::context::Ctx;
use crate::session::here::here;

fn group_id(id: ClusterId) -> GroupId {
    match id {
        ClusterId::Dtu => GroupId::Dtu,
        ClusterId::Gefion => GroupId::Gefion,
        ClusterId::Ngc => GroupId::Ngc,
    }
}

fn not_set_up(id: ClusterId) -> &'static str {
    if id == ClusterId::Ngc {
        "not in config.toml; aicentre check --all checks it too"
    } else {
        "no Host in ~/.ssh/config points at it; aicentre check --all shows what to add"
    }
}

async fn selected(id: ClusterId, base: &CheckEnv<'_>, clusters: &[ClusterId]) -> bool {
    if base.explicit {
        return clusters.contains(&id);
    }
    base.machine.ssh_usable && in_use(id, base.ctx, base.config, &base.resolver).await
}

async fn check_cluster(
    id: ClusterId,
    group: Option<Group>,
    chosen: bool,
    base: &CheckEnv<'_>,
) -> Option<GroupReport> {
    let mut g = group?;
    if !chosen {
        g.skipped = true;
        g.note = Some(not_set_up(id).to_string());
        return Some(g.end());
    }
    if here(base.ctx) == Some(id) {
        let detail =
            format!("is the {} login node; aicentre runs its scheduler directly", id.name());
        g.step("here", "This machine", async { Ok(Outcome::info(detail)) }).await;
        return Some(g.end());
    }
    match id {
        ClusterId::Dtu => check_dtu(&mut g, base).await,
        ClusterId::Gefion => check_gefion(&mut g, base).await,
        ClusterId::Ngc => check_ngc(&mut g, base).await,
    }
    Some(g.end())
}

pub async fn run_checks(
    ctx: &Ctx,
    config: &Config,
    clusters: &[ClusterId],
    connect: bool,
    sink: Arc<dyn Sink>,
) -> Vec<GroupReport> {
    let mut local = Group::new(GroupId::Local, "This machine", sink.clone());
    let machine = check_local(&mut local, ctx).await;
    let mut reports = vec![local.end()];

    let base = CheckEnv {
        ctx,
        config,
        connect,
        explicit: !clusters.is_empty(),
        machine,
        resolver: resolver(ctx, config),
    };
    let [dtu, gefion, ngc] = CLUSTER_IDS;
    let chosen = tokio::join!(
        selected(dtu, &base, clusters),
        selected(gefion, &base, clusters),
        selected(ngc, &base, clusters),
    );
    let chosen = [chosen.0, chosen.1, chosen.2];
    let [g0, g1, g2]: [Option<Group>; 3] = std::array::from_fn(|i| {
        let id = CLUSTER_IDS[i];
        (chosen[i] || !base.explicit).then(|| Group::new(group_id(id), id.name(), sink.clone()))
    });
    let done = tokio::join!(
        check_cluster(dtu, g0, chosen[0], &base),
        check_cluster(gefion, g1, chosen[1], &base),
        check_cluster(ngc, g2, chosen[2], &base),
    );
    reports.extend([done.0, done.1, done.2].into_iter().flatten());
    reports
}
