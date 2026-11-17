use crate::check::types::Status;
use crate::context::Ctx;
use crate::install::layout::{
    Linked, activate, link_bin, place_binary, prune_versions, remove_if_empty, safe_remove,
    smoke_test, version_dir, write_manifest,
};
use crate::paths::{basename, tilde};
use crate::ui::say::fail;
use crate::ui::style::make_style;

pub fn commit(ctx: &Ctx, version: &str, created: &[String]) -> anyhow::Result<Linked> {
    activate(&ctx.paths, version)?;
    let link = link_bin(&ctx.paths)?;
    let all: Vec<String> = created.iter().chain(&link.created).cloned().collect();
    write_manifest(&ctx.paths, version, ctx.platform, &all)?;
    prune_versions(&ctx.paths, version)?;
    Ok(link.result)
}

pub struct PathHint {
    pub add: String,
    pub now: Option<String>,
}

pub fn path_hint(ctx: &Ctx) -> Option<PathHint> {
    let path = ctx.env("PATH").unwrap_or("");
    if path.split(':').any(|d| d == ctx.paths.bin_dir) {
        return None;
    }
    let shell = basename(ctx.env("SHELL").unwrap_or(""));
    let short = tilde(&ctx.paths.bin_dir, &ctx.paths.home);
    let dir = match short.strip_prefix('~') {
        Some(rest) => format!("$HOME{rest}"),
        None => short,
    };
    Some(PathHint { add: path_line(&shell, &dir), now: session_line(&shell, &dir) })
}

pub fn path_line(shell: &str, dir: &str) -> String {
    if shell == "fish" {
        return format!("fish_add_path {}", dir.replacen("$HOME", "~", 1));
    }
    if shell == "tcsh" || shell == "csh" {
        return format!("echo 'set path = ({dir} $path)' >> ~/.cshrc");
    }
    let rc = match shell {
        "zsh" => "~/.zshrc",
        "bash" => "~/.bashrc",
        _ => "~/.profile",
    };
    format!("echo 'export PATH=\"{dir}:$PATH\"' >> {rc}")
}

pub fn session_line(shell: &str, dir: &str) -> Option<String> {
    if shell == "fish" {
        return None;
    }
    if shell == "tcsh" || shell == "csh" {
        return Some(format!("set path = ({dir} $path)"));
    }
    Some(format!("export PATH=\"{dir}:$PATH\""))
}

pub const DEFAULT_BIN: &str = "$HOME/.local/bin";

pub async fn self_install(ctx: &Ctx) -> anyhow::Result<i32> {
    let s = make_style(&ctx.env, &ctx.out);
    if !ctx.compiled {
        ctx.err.write("aicentre: self-install runs from a release binary; install.sh calls it.\n");
        return Ok(2);
    }
    let home = &ctx.paths.home;
    let placed = place_binary(&ctx.paths, &ctx.version, &ctx.exec_path)?;
    if let Some(problem) = smoke_test(&placed.path, &ctx.version, &ctx.env).await {
        safe_remove(&version_dir(&ctx.paths, &ctx.version), &ctx.paths)?;
        remove_if_empty(&placed.created);
        return Ok(fail(ctx, &format!("the new binary does not run: {problem}"), &[]));
    }
    let link = commit(ctx, &ctx.version, &placed.created)?;

    let ok = s.symbol(Status::Ok);
    let warn = s.symbol(Status::Warn);
    ctx.out.write(&format!(
        "{ok} Installed aicentre {} in {}\n",
        ctx.version,
        tilde(&ctx.paths.data, home)
    ));
    if link == Linked::Foreign {
        ctx.out.write(&format!(
            "{warn} {} already exists and is not ours, so it was left alone.\n",
            tilde(&ctx.paths.bin_link, home)
        ));
        ctx.out
            .write(&format!("  Run aicentre as {}.\n", tilde(&current_path(&placed.path), home)));
    } else {
        ctx.out.write(&format!("  The command is {}.\n", tilde(&ctx.paths.bin_link, home)));
        if let Some(hint) = path_hint(ctx) {
            let at = if hint.now.is_some() { " for new terminals" } else { "" };
            ctx.out.write(&format!(
                "{warn} {} is not on your PATH. Add it{at}:\n",
                tilde(&ctx.paths.bin_dir, home)
            ));
            ctx.out.write(&format!("    {}\n", s.cyan(&hint.add)));
            if let Some(now) = hint.now {
                ctx.out.write(&format!("  and for this one:\n    {}\n", s.cyan(&now)));
            }
        }
    }
    ctx.out.write(&format!(
        "\nNext, see whether this machine is ready for the clusters:\n    {}\n",
        s.cyan("aicentre check")
    ));
    Ok(0)
}

fn current_path(binary: &str) -> String {
    static VERSION_PART: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"versions/[^/]+").unwrap());
    VERSION_PART.replace(binary, "current").into_owned()
}
