use std::path::Path;

use crate::check::clusters::{ClusterId, DTU};
use crate::context::Ctx;
use crate::exec::which;
use crate::paths::join;

pub fn provider_file(data_dir: &str) -> String {
    join(&[data_dir, "provider"])
}

fn reachable(s: &str) -> Option<ClusterId> {
    match s {
        "dtu" => Some(ClusterId::Dtu),
        "gefion" => Some(ClusterId::Gefion),
        _ => None,
    }
}

pub fn here(ctx: &Ctx) -> Option<ClusterId> {
    let said = ctx.env("AICENTRE_HERE");
    if let Some(id) = said.and_then(reachable) {
        return Some(id);
    }
    if said == Some("none") {
        return None;
    }
    if let Ok(marked) = std::fs::read_to_string(provider_file(&ctx.paths.data))
        && let Some(id) = reachable(marked.trim())
    {
        return Some(id);
    }
    if which("bsub", &ctx.env).is_some() && Path::new(DTU.project_dir).exists() {
        return Some(ClusterId::Dtu);
    }
    let host =
        nix::unistd::gethostname().map(|h| h.to_string_lossy().to_lowercase()).unwrap_or_default();
    if which("sbatch", &ctx.env).is_some()
        && (host.contains("dcai") || host.starts_with("slurm-login"))
    {
        return Some(ClusterId::Gefion);
    }
    None
}
