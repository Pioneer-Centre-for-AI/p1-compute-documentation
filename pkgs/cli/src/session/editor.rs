use std::path::Path;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;

use crate::browser::has_display;
use crate::config::Config;
use crate::context::{Ctx, Env};
use crate::exec::{RunOptions, errno_name, run, which};
use crate::paths::{basename, join};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Editor {
    pub name: String,
    pub argv: Vec<String>,
    pub ready: bool,
}

struct Known {
    commands: &'static [&'static str],
    args: &'static [&'static str],
    remote: Option<&'static LazyLock<Regex>>,
}

static REMOTE_SSH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)remote-(open)?ssh").unwrap());

static KNOWN: [Known; 2] = [
    Known {
        commands: &["code", "codium", "cursor", "windsurf"],
        args: &["--folder-uri", "vscode-remote://ssh-remote+{host}{urlpath}"],
        remote: Some(&REMOTE_SSH),
    },
    Known { commands: &["zed", "zeditor"], args: &["ssh://{host}{urlpath}"], remote: None },
];

pub fn split_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut quote: Option<char> = None;
    for ch in text.chars() {
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else {
                word.get_or_insert_with(String::new).push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
            word.get_or_insert_with(String::new);
        } else if ch.is_whitespace() {
            if let Some(w) = word.take() {
                words.push(w);
            }
        } else {
            word.get_or_insert_with(String::new).push(ch);
        }
    }
    if let Some(w) = word {
        words.push(w);
    }
    words
}

fn configured(ctx: &Ctx, template: &str) -> Option<Editor> {
    let words = split_words(template);
    let (program, args) = words.split_first()?;
    if program.is_empty() {
        return None;
    }
    let path = match program.strip_prefix("~/") {
        Some(rest) => join(&[&ctx.paths.home, rest]),
        None => program.clone(),
    };
    let cli = if path.contains('/') {
        Path::new(&path).exists().then_some(path)
    } else {
        which(&path, &ctx.env)
    }?;
    let mut argv = vec![cli];
    argv.extend(args.iter().cloned());
    Some(Editor { name: basename(program), argv, ready: true })
}

fn preferred(env: &Env) -> Option<String> {
    ["VISUAL", "EDITOR"].iter().find_map(|var| {
        let name = basename(split_words(env.get(*var)?).first()?);
        KNOWN.iter().any(|k| k.commands.contains(&name.as_str())).then_some(name)
    })
}

pub fn editor_installed(ctx: &Ctx, config: &Config) -> bool {
    if !has_display(ctx) {
        return false;
    }
    if let Some(open) = config.desktop.as_ref().and_then(|d| d.open.as_deref()) {
        return configured(ctx, open).is_some();
    }
    KNOWN.iter().any(|k| k.commands.iter().any(|c| which(c, &ctx.env).is_some()))
}

pub async fn find_editor(ctx: &Ctx, config: &Config) -> Option<Editor> {
    if !has_display(ctx) {
        return None;
    }
    if let Some(open) = config.desktop.as_ref().and_then(|d| d.open.as_deref()) {
        return configured(ctx, open);
    }
    let first = preferred(&ctx.env);
    let mut candidates: Vec<(&Known, &str)> =
        KNOWN.iter().flat_map(|k| k.commands.iter().map(move |c| (k, *c))).collect();
    candidates.sort_by_key(|(_, command)| Some(*command) != first.as_deref());
    let mut checks = Vec::new();
    for (k, command) in candidates {
        let Some(cli) = which(command, &ctx.env) else { continue };
        let env = ctx.env.clone();
        let remote = k.remote;
        let mut argv = vec![cli.clone()];
        argv.extend(k.args.iter().map(|a| a.to_string()));
        checks.push(tokio::spawn(async move {
            let mut ready = true;
            if let Some(re) = remote {
                let opts = RunOptions::new(&env).timeout_ms(30_000);
                let r = run(&cli, &["--list-extensions"], opts).await;
                ready = r.code == 0 && re.is_match(&r.stdout);
            }
            Editor { name: command.to_string(), argv, ready }
        }));
    }
    let mut editors = Vec::new();
    for check in checks {
        if let Ok(editor) = check.await {
            editors.push(editor);
        }
    }
    let ready = editors.iter().position(|e| e.ready).unwrap_or(0);
    (!editors.is_empty()).then(|| editors.swap_remove(ready))
}

fn encode_uri_component(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

pub fn remote_args(argv: &[String], host: &str, path: &str) -> Vec<String> {
    let urlpath = path.split('/').map(encode_uri_component).collect::<Vec<_>>().join("/");
    argv.iter()
        .map(|word| {
            word.replace("{host}", host).replace("{urlpath}", &urlpath).replace("{path}", path)
        })
        .collect()
}

pub async fn open_folder(ctx: &Ctx, argv: &[String]) -> Option<String> {
    let (program, args) = argv.split_first()?;
    let mut c = tokio::process::Command::new(program);
    c.args(args)
        .env_clear()
        .envs(&ctx.env)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // SAFETY: setsid is async-signal-safe and runs in the child before exec.
    unsafe {
        c.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    let mut child = match c.spawn() {
        Ok(child) => child,
        Err(e) => return Some(format!("spawn {program} {}", errno_name(&e))),
    };
    match tokio::time::timeout(Duration::from_millis(15_000), child.wait()).await {
        Err(_) => None,
        Ok(Ok(status)) if status.success() => None,
        Ok(Ok(status)) => Some(format!(
            "{program} exited {}",
            status.code().map_or("null".to_string(), |c| c.to_string())
        )),
        Ok(Err(e)) => Some(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn an_editor_template_is_words_with_host_path_and_urlpath_filled_in() {
        let vscode = words(&["code", "--folder-uri", "vscode-remote://ssh-remote+{host}{urlpath}"]);
        assert_eq!(
            remote_args(&vscode, "dtu", "/dtu/p1/s123456/my project"),
            ["code", "--folder-uri", "vscode-remote://ssh-remote+dtu/dtu/p1/s123456/my%20project"]
        );
        assert_eq!(
            remote_args(&words(&["emacsclient", "-c", "/ssh:{host}:{path}"]), "dtu", "/a b"),
            ["emacsclient", "-c", "/ssh:dtu:/a b"]
        );
        assert_eq!(
            split_words(r#""/Applications/Zed.app/Contents/MacOS/cli" 'ssh://{host}{urlpath}'"#),
            ["/Applications/Zed.app/Contents/MacOS/cli", "ssh://{host}{urlpath}"]
        );
        assert_eq!(split_words(r#"  a ""  b"c d"e "#), ["a", "", "bc de"]);
        assert_eq!(encode_uri_component("ä#?&=/(x)"), "%C3%A4%23%3F%26%3D%2F(x)");
    }
}
