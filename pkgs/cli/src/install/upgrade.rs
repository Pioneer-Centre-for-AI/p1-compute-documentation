use std::cmp::Ordering;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::Path;

use anyhow::bail;

use crate::check::types::Status;
use crate::context::Ctx;
use crate::exec::{RunOptions, run};
use crate::install::layout::{
    Linked, mkdirp, read_manifest, remove_if_empty, safe_remove, smoke_test, version_dir,
};
use crate::install::release::{
    DownloadError, download, download_base, fetch_text, is_version, parse_sums, sha256,
    version_order,
};
use crate::install::self_install::commit;
use crate::install::signature;
use crate::paths::{ensure_runtime_dir, join, tilde};
use crate::platform::{asset_name, is_nix_store};
use crate::ui::say::fail;
use crate::ui::style::make_style;

const INSTALL: &str = "curl -fsSL https://hpc.aicentre.dk/install | sh";

struct Lock(String);

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn lock(ctx: &Ctx) -> anyhow::Result<Lock> {
    let path = join(&[&ensure_runtime_dir(&ctx.paths, ctx.uid)?, "upgrade.lock"]);
    for _ in 0..2 {
        match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(std::process::id().to_string().as_bytes())?;
                return Ok(Lock(path));
            }
            Err(e) if e.kind() != ErrorKind::AlreadyExists => return Err(e.into()),
            Err(_) => {}
        }
        let pid: i32 = fs::read_to_string(&path)?.trim().parse().unwrap_or(0);
        if pid > 0 {
            match nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None) {
                Ok(()) => bail!("another upgrade is running (pid {pid})"),
                Err(nix::errno::Errno::ESRCH) => {}
                Err(e) => return Err(std::io::Error::from(e).into()),
            }
        }
        let _ = fs::remove_file(&path);
    }
    bail!("cannot take the upgrade lock")
}

pub async fn upgrade(ctx: &Ctx, version: Option<&str>) -> anyhow::Result<i32> {
    let pinned = version.filter(|v| !v.is_empty());
    if !ctx.compiled {
        return Ok(fail(
            ctx,
            "upgrade works on an installed release; this is a development run.",
            &["Update the checkout with git pull instead.".into()],
        ));
    }
    if is_nix_store(&ctx.exec_path) {
        return Ok(fail(ctx, "this aicentre comes from nix; upgrade it with nix.", &[]));
    }
    if read_manifest(&ctx.paths).is_none() {
        return Ok(fail(
            ctx,
            &format!(
                "{} is missing, so this copy was not set up by the installer.",
                tilde(&ctx.paths.manifest, &ctx.paths.home)
            ),
            &[format!("Reinstall with: {INSTALL}")],
        ));
    }
    if let Some(v) = pinned
        && !is_version(v)
    {
        return Ok(fail(ctx, &format!("\"{v}\" is not a version like 0.2.0."), &[]));
    }

    let base = download_base(&ctx.env, pinned);
    let held = match lock(ctx) {
        Ok(l) => l,
        Err(e) => return Ok(fail(ctx, &e.to_string(), &[])),
    };

    let tarball = join(&[&ctx.paths.cache, &asset_name(ctx.platform)]);
    let mut work = Work { stage: None, created: vec![] };
    let result = fetch_and_switch(ctx, version, &base, &tarball, &mut work).await;
    if let Some(stage) = &work.stage {
        safe_remove(stage, &ctx.paths)?;
    }
    let _ = fs::remove_file(&tarball);
    remove_if_empty(&work.created);
    drop(held);
    match result {
        Err(e) => match e.downcast_ref::<DownloadError>() {
            Some(d) => Ok(fail(ctx, &d.0, &[])),
            None => Err(e),
        },
        ok => ok,
    }
}

struct Work {
    stage: Option<String>,
    created: Vec<String>,
}

async fn fetch_and_switch(
    ctx: &Ctx,
    version: Option<&str>,
    base: &str,
    tarball: &str,
    work: &mut Work,
) -> anyhow::Result<i32> {
    let s = make_style(&ctx.env, &ctx.out);
    let pinned = version.is_some_and(|v| !v.is_empty());
    let asset = asset_name(ctx.platform);
    let target = match version {
        Some(v) => v.to_string(),
        None => fetch_text(base, "VERSION").await?.trim().to_string(),
    };
    if !is_version(&target) {
        let message =
            format!("the release names its version \"{target}\", which is not a version.");
        return Ok(fail(ctx, &message, &[]));
    }
    let ok = s.symbol(Status::Ok);
    match version_order(&target, &ctx.version) {
        Ordering::Equal => {
            let what = if pinned { "already installed" } else { "the latest" };
            ctx.out.write(&format!("{ok} aicentre {} is {what}.\n", ctx.version));
            return Ok(0);
        }
        Ordering::Less if !pinned => {
            ctx.out.write(&format!(
                "{ok} aicentre {} is newer than the latest release, {target}.\n",
                ctx.version
            ));
            return Ok(0);
        }
        _ => {}
    }

    ctx.out.write(&format!(
        "Downloading aicentre {target} for {}-{}...\n",
        ctx.platform.os.as_str(),
        ctx.platform.arch.as_str()
    ));
    let listed = fetch_text(base, "SHA256SUMS").await?;
    let signed = match fetch_text(base, signature::FILE).await {
        Ok(armored) => {
            signature::verify(listed.as_bytes(), &armored, &signature::signers(&ctx.env))
        }
        Err(_) => Err("it has no signature".into()),
    };
    if let Err(why) = signed {
        return Ok(fail(
            ctx,
            &format!("the {target} release cannot be trusted: {why}; nothing was changed."),
            &[format!("aicentre checks {} against its release key.", signature::FILE)],
        ));
    }
    let sums = parse_sums(&listed);
    let Some(expected) = sums.get(&asset) else {
        return Ok(fail(ctx, &format!("the {target} release has no {asset}."), &[]));
    };
    work.created = mkdirp(&ctx.paths.cache)?;
    download(base, &asset, tarball).await?;
    let actual = sha256(tarball)?;
    if actual != *expected {
        return Ok(fail(
            ctx,
            &format!("{asset} does not match its checksum; nothing was changed."),
            &[format!("expected {expected}, got {actual}")],
        ));
    }

    let stage = format!("{}.partial", version_dir(&ctx.paths, &target));
    work.stage = Some(stage.clone());
    safe_remove(&stage, &ctx.paths)?;
    fs::create_dir_all(&stage)?;
    let untar =
        run("tar", &["-xzf", tarball, "-C", &stage], RunOptions::new(&ctx.env).timeout_ms(120_000))
            .await;
    if untar.code != 0 || !Path::new(&join(&[&stage, "aicentre"])).exists() {
        let why = match untar.stderr.trim() {
            "" => "no aicentre inside",
            text => text,
        };
        return Ok(fail(ctx, &format!("cannot unpack {asset}: {why}"), &[]));
    }
    let dir = version_dir(&ctx.paths, &target);
    safe_remove(&dir, &ctx.paths)?;
    fs::rename(&stage, &dir)?;
    work.stage = Some(dir.clone());

    if let Some(problem) = smoke_test(&join(&[&dir, "aicentre"]), &target, &ctx.env).await {
        let message = format!("the new binary does not run, so nothing was changed: {problem}");
        return Ok(fail(ctx, &message, &[]));
    }

    let link = commit(ctx, &target, &[])?;
    work.stage = None;
    ctx.out.write(&format!("{ok} Upgraded aicentre {} → {target}\n", ctx.version));
    if link == Linked::Foreign {
        ctx.out.write(&format!(
            "{} {} is not ours, so it was left alone.\n",
            s.symbol(Status::Warn),
            tilde(&ctx.paths.bin_link, &ctx.paths.home)
        ));
    }
    Ok(0)
}
