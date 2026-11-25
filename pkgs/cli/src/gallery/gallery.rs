use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::check::clusters::{ClusterId, SITE};
use crate::check::types::Status;
use crate::context::{Ctx, for_tests};
use crate::exec::shq;
use crate::session::login::{Choice, with_provider};
use crate::ui::say::{fail, printable};
use crate::ui::style::make_style;
use crate::ui::table::{TableOptions, table};
use crate::unpack::{Rules, unpack};

#[derive(Clone, Debug, Deserialize)]
pub struct Example {
    pub code: String,
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub providers: Vec<ClusterId>,
    pub page: String,
    pub files: Option<String>,
    pub commands: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum GalleryRequest {
    List,
    Get {
        code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        dir: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider: Option<ClusterId>,
    },
}

#[derive(Deserialize)]
struct Index {
    examples: Vec<Example>,
}

fn site(ctx: &Ctx) -> String {
    ctx.env_set("AICENTRE_SITE").unwrap_or(SITE).trim_end_matches('/').to_string()
}

fn file_path(url: &str) -> String {
    let rest = url.strip_prefix("file://").unwrap_or(url);
    if rest.starts_with('/') {
        return rest.to_string();
    }
    rest.find('/').map_or_else(|| "/".to_string(), |i| rest[i..].to_string())
}

pub async fn examples(ctx: &Ctx) -> Result<Vec<Example>, String> {
    let url = format!("{}/gallery/index.json", site(ctx));
    let body = if url.starts_with("file://") {
        let path = file_path(&url);
        std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?
    } else {
        let res = crate::http::get(&url, Duration::from_secs(20)).await?;
        if !res.ok() {
            return Err(format!("{url}: HTTP {}", res.status));
        }
        res.text()
    };
    let mut all =
        serde_json::from_str::<Index>(&body).map(|i| i.examples).map_err(|e| e.to_string())?;
    for e in &mut all {
        for text in [&mut e.code, &mut e.slug, &mut e.title, &mut e.page] {
            *text = printable(text);
        }
        for command in &mut e.commands {
            *command = printable(command);
        }
    }
    Ok(all)
}

const TAKEN: i32 = 17;

fn slug_ok(slug: &str) -> bool {
    !slug.is_empty()
        && slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub async fn unpack_example(ctx: &Ctx, url: &str, slug: &str, parent: &str) -> anyhow::Result<i32> {
    let local = for_tests(&ctx.env, "AICENTRE_SITE").and_then(|_| url.strip_prefix("file://"));
    let named = slug_ok(slug) && url.ends_with(&format!("/gallery/{slug}.tar.gz"));
    if !named || !(url.starts_with("https://") || local.is_some()) || !parent.starts_with('/') {
        return Ok(fail(ctx, &format!("{url} is not the archive of the example {slug}"), &[]));
    }
    let folder = format!("{parent}/{slug}");
    if std::fs::symlink_metadata(&folder).is_ok() {
        return Ok(TAKEN);
    }
    let archive = match local {
        Some(path) => std::fs::read(file_path(path)).map_err(|e| e.to_string()),
        None => match crate::http::get(url, Duration::from_secs(120)).await {
            Ok(got) if got.ok() => Ok(got.body),
            Ok(got) => Err(format!("HTTP {}", got.status)),
            Err(e) => Err(e),
        },
    };
    let archive = match archive {
        Ok(bytes) => bytes,
        Err(why) => return Ok(fail(ctx, &format!("cannot download {url}: {why}"), &[])),
    };
    if let Err(e) = crate::install::layout::mkdirp(parent) {
        return Ok(fail(ctx, &format!("cannot create {parent}: {e}"), &[]));
    }
    match unpack(&archive, parent, Rules { under: Some(slug), ..Default::default() }) {
        Ok(0) => Ok(fail(ctx, &format!("the archive at {url} has no {slug} folder"), &[])),
        Ok(_) => Ok(0),
        Err(why) => {
            let _ = std::fs::remove_dir_all(&folder);
            Ok(fail(ctx, &format!("cannot unpack {url}: {why}"), &[]))
        }
    }
}

fn archive(ctx: &Ctx, example: &Example, files: &str) -> bool {
    let tail = format!("/gallery/{}.tar.gz", example.slug);
    slug_ok(&example.slug)
        && (files == format!("{SITE}{tail}") || files == format!("{}{tail}", site(ctx)))
}

pub async fn gallery(ctx: &Ctx, req: &GalleryRequest) -> anyhow::Result<i32> {
    let all = match examples(ctx).await {
        Ok(all) => all,
        Err(e) => return Ok(fail(ctx, &format!("cannot read the gallery: {e}"), &[])),
    };
    match req {
        GalleryRequest::List => Ok(list(ctx, &all)),
        GalleryRequest::Get { code, dir, provider } => {
            get(ctx, &all, code, dir.as_deref(), *provider).await
        }
    }
}

fn short(id: ClusterId) -> String {
    let name = id.name();
    let name = name.strip_prefix("P1 ").unwrap_or(name);
    name.strip_suffix(" HPC").unwrap_or(name).to_string()
}

fn list(ctx: &Ctx, all: &[Example]) -> i32 {
    let s = make_style(&ctx.env, &ctx.out);
    let rows: Vec<Vec<String>> = all
        .iter()
        .map(|e| {
            let at: Vec<String> = e.providers.iter().map(|p| short(*p)).collect();
            vec![e.code.clone(), format!("{} {}", e.title, s.dim(&format!("· {}", at.join(", "))))]
        })
        .collect();
    let lines = table(
        &rows,
        TableOptions {
            gap: "   ",
            style: Box::new(|t, _, c| if c == 0 { s.cyan(t) } else { t.to_string() }),
            ..Default::default()
        },
    );
    ctx.out.write(&format!(
        "{}  {}\n\n{}\n",
        s.bold("Gallery"),
        s.dim(&format!("{}/gallery/", site(ctx))),
        lines.join("\n")
    ));
    ctx.out.write(&format!(
        "\n{}\n",
        s.dim("aicentre gallery get <code> puts an example on the cluster, with what to run next.")
    ));
    0
}

async fn get(
    ctx: &Ctx,
    all: &[Example],
    code: &str,
    dir: Option<&str>,
    provider: Option<ClusterId>,
) -> anyhow::Result<i32> {
    let s = &make_style(&ctx.env, &ctx.out);
    let Some(example) = all.iter().find(|e| e.slug == code.to_lowercase()) else {
        let slugs: Vec<&str> = all.iter().map(|e| e.slug.as_str()).collect();
        return Ok(fail(
            ctx,
            &format!("no example {code} in the gallery."),
            &[format!("There are {}.", slugs.join(", "))],
        ));
    };

    let next = || {
        if example.commands.is_empty() {
            return;
        }
        let commands: String =
            example.commands.iter().map(|c| format!("  {} {c}\n", s.dim("$"))).collect();
        ctx.out.write(&format!("\nNext, from this machine:\n{commands}"));
        if let Some(dir) = dir {
            ctx.out.write(&s.dim(&format!("  These assume examples/; you put it in {dir}.\n")));
        }
    };
    let Some(files) = example.files.as_deref().filter(|f| !f.is_empty()) else {
        ctx.out.write(&format!(
            "{} has no files to fetch; it is commands only. {}\n",
            example.code,
            s.dim(&example.page)
        ));
        next();
        return Ok(0);
    };

    if !archive(ctx, example, files) {
        let error =
            format!("the gallery names files for {} that are not its own archive", example.code);
        return Ok(fail(
            ctx,
            &error,
            &[format!("They are at {files}, not under {SITE}/gallery/.")],
        ));
    }

    let names: Vec<&str> = example.providers.iter().map(|p| p.name()).collect();
    let runs_on = format!("{} runs on {}", example.code, names.join(" and "));
    let choice = Choice {
        asked: provider,
        among: Some(example.providers.clone()),
        outside: Some(format!("{runs_on}.")),
        none: Some(vec![
            format!("{runs_on}, and this machine is not set up for it."),
            "aicentre check --all shows what each provider needs.".into(),
        ]),
    };
    with_provider(ctx, &choice, |mut login, config| async move {
        let installing = |text: &str| ctx.out.write(&format!("{text}...\n"));
        if let Err((error, hints)) = login.ready_to_launch(ctx, &config, &installing).await {
            return Ok(fail(ctx, &error, &hints));
        }
        let parent = login.path(dir.unwrap_or("examples"));
        let words = [login.launcher.as_str(), "unpack-example", files, &example.slug, &parent];
        let command = words.iter().map(|w| shq(w)).collect::<Vec<_>>().join(" ");
        let r = login.master.run(&command, 180_000).await;
        if r.code == TAKEN {
            return Ok(fail(
                ctx,
                &format!("{parent}/{} is already on {}.", example.slug, login.alias),
                &["Move it away, or give another folder: aicentre gallery get <code> <dir>."
                    .into()],
            ));
        }
        if r.code != 0 {
            let said = printable(r.stderr.trim().split('\n').next_back().unwrap_or(""));
            return Ok(fail(
                ctx,
                &format!("cannot fetch {} on {}: {said}", example.code, login.alias),
                &[],
            ));
        }
        ctx.out.write(&format!(
            "{} {} is in {parent}/{} on {}\n",
            s.symbol(Status::Ok),
            example.code,
            example.slug,
            login.provider.name()
        ));
        next();
        Ok(0)
    })
    .await
}
