use std::fs;
use std::path::Path;

use crate::check::types::Status;
use crate::context::Ctx;
use crate::install::layout::{
    is_our_link, read_manifest, remove_if_empty, remove_link, safe_remove,
};
use crate::paths::tilde;
use crate::platform::is_nix_store;
use crate::session::sync::read_remotes;
use crate::ui::prompt::confirm;
use crate::ui::style::make_style;

pub async fn uninstall(ctx: &Ctx, purge: bool) -> anyhow::Result<i32> {
    let s = make_style(&ctx.env, &ctx.out);
    if is_nix_store(&ctx.exec_path) {
        ctx.err.write(
            "aicentre: this aicentre comes from nix; remove it from your Nix configuration instead.\n",
        );
        return Ok(1);
    }
    let paths = &ctx.paths;
    let home = &paths.home;
    let mut removed: Vec<String> = Vec::new();
    let mut kept: Vec<String> = Vec::new();
    let created_dirs = read_manifest(paths).and_then(|m| m.created_dirs).unwrap_or_default();
    let remotes = read_remotes(ctx);

    if fs::symlink_metadata(&paths.bin_link).is_ok() {
        if is_our_link(&paths.bin_link, paths) {
            remove_link(&paths.bin_link)?;
            removed.push(paths.bin_link.clone());
        } else {
            kept.push(format!("{} (not ours)", tilde(&paths.bin_link, home)));
        }
    }

    for dir in [&paths.data, &paths.state, &paths.cache, &paths.runtime] {
        if !Path::new(dir).exists() {
            continue;
        }
        safe_remove(dir, paths)?;
        removed.push(dir.clone());
    }

    let config = tilde(&paths.config, home);
    if Path::new(&paths.config).exists() {
        let question = format!("Also remove your settings in {config}?");
        let remove = purge || (ctx.interactive && confirm(ctx, &question).await);
        if remove {
            safe_remove(&paths.config, paths)?;
            removed.push(paths.config.clone());
        } else {
            kept.push(config.clone());
        }
    }

    remove_if_empty(&created_dirs);

    let ok = s.symbol(Status::Ok);
    let info = s.symbol(Status::Info);
    if removed.is_empty() {
        ctx.out.write(&format!("{ok} aicentre is not installed here; nothing to remove.\n"));
    } else {
        ctx.out.write(&format!("{ok} Removed aicentre:\n"));
        for path in &removed {
            ctx.out.write(&format!("    {}\n", tilde(path, home)));
        }
    }
    for path in &kept {
        ctx.out.write(&format!("{info} Kept {path}\n"));
    }
    if kept.contains(&config) {
        ctx.out.write(&s.dim(&format!("  Remove it later with: rm -r {config}\n")));
    }
    for (provider, remote) in &remotes {
        ctx.out.write(&format!(
            "{info} aicentre is also installed on {}. To remove it there:\n",
            provider.name()
        ));
        let command = format!("ssh -t {} '~/.local/bin/aicentre uninstall'", remote.alias);
        ctx.out.write(&format!("    {}\n", s.cyan(&command)));
        ctx.out.write(&s.dim(
            "    What sessions installed there stays in .aicentre in your project directory.\n",
        ));
    }
    Ok(0)
}
