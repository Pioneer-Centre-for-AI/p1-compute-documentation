use std::time::Duration;

use super::pyenv::Python;
use crate::context::{Env, for_tests};
use crate::exec::{RunOptions, run, which};
use crate::install::release::sha256_hex;
use crate::paths::join;
use crate::unpack::{Rules, unpack};

pub const UV: &str = "0.13.9";
const UV_SHA_X86_64: &str = "58ccf5f1792b8a5a88d01bc910ea131a8b3db4a319a17ad5fad75e0845a3e271";
const UV_SHA_AARCH64: &str = "37fbb7c5bc66cf99c12aa881949d216181e201dec449772c4329e56487d61179";
pub const OPENVSCODE: &str = "openvscode-server-v1.109.5";
const OPENVSCODE_SHA: &str = "b433bf4f0227321a7014d8460d10a8f958adc0f45aa79bd889e84e65e8f88363";
pub const VSCODE: &str = "69047191fc427f2f92828279aae1d90d8eeda7ab";
const VSCODE_SHA: &str = "7c4536e63c54c90fae02b74dfa98342aa2d5f02b342505c5b3d37033ab8d4f2f";

fn folder(url: &str) -> &str {
    url.rsplit_once('/').map_or(url, |(dir, _)| dir)
}

async fn body(url: &str, local: bool) -> Result<Vec<u8>, String> {
    if let Some(path) = url.strip_prefix("file://") {
        return if local {
            std::fs::read(path).map_err(|e| e.to_string())
        } else {
            Err("not https".into())
        };
    }
    if !url.starts_with("https://") {
        return Err("not https".into());
    }
    // The VS Code server is about 300 MB.
    let got = crate::http::get_up_to(url, Duration::from_secs(900), 1 << 30).await?;
    if got.ok() { Ok(got.body) } else { Err(format!("HTTP {}", got.status)) }
}

pub async fn fetch(env: &Env, what: &str, url: &str, want: &str) -> Result<Vec<u8>, String> {
    let mirror = for_tests(env, "AICENTRE_FETCH_FROM");
    let url = match mirror {
        Some(m) => format!("{m}/{}", url.rsplit('/').next().unwrap_or(url)),
        None => url.to_string(),
    };
    let from = folder(&url);
    let bytes = body(&url, mirror.is_some())
        .await
        .map_err(|why| format!("cannot download {what} from {from}: {why}"))?;
    let got = sha256_hex(&bytes);
    if got != want {
        return Err(format!(
            "{what} from {from} is not the file this aicentre was built for (sha256 {got}), so it was deleted and nothing ran"
        ));
    }
    Ok(bytes)
}

pub async fn machine(env: &Env) -> String {
    run("uname", &["-m"], RunOptions::new(env).timeout_ms(5_000)).await.stdout.trim().to_string()
}

async fn x64_only(env: &Env, what: &str) -> Result<(), String> {
    let arch = machine(env).await;
    if arch == "x86_64" {
        return Ok(());
    }
    Err(format!("{what} is only set up for x86_64 nodes, and this one is {arch}"))
}

fn started_marker(data: &str, app: &str) -> String {
    format!("{data}/started/{app}")
}

pub fn first(data: &str, app: &str) -> bool {
    !std::path::Path::new(&started_marker(data, app)).exists()
}

pub fn mark_started(data: &str, app: &str) {
    let marker = started_marker(data, app);
    let _ = std::fs::create_dir_all(join(&[data, "started"]));
    let _ = std::fs::write(marker, "");
}

pub const EXTENSIONS: [&str; 3] =
    ["ms-python.python", "ms-toolsai.jupyter", "marimo-team.vscode-marimo"];

pub const NOTEBOOKS: &str = "code-notebooks";

pub fn wanted(env: &Env, app: &str, server: &str, data: &str, python: &Python) -> Vec<String> {
    let mut out = Vec::new();
    if matches!(app, "shell" | "marimo" | "jupyter" | "code") && which("uv", env).is_none() {
        out.push(format!("uv {UV}, from github.com/astral-sh/uv"));
    }
    if app != "sshd"
        && let Python::Default(venv) = python
        && !python.exists()
    {
        out.push(format!(
            "Python {}, as a default environment in {venv}, from github.com/astral-sh/python-build-standalone",
            super::pyenv::PYTHON
        ));
    }
    let first = |app: &str| first(data, app);
    let missing = |program: &str, app: &str| match python.venv() {
        Some(_) => !python.has(program),
        None => first(app),
    };
    let into =
        python.venv().map_or_else(|| "through uv".to_string(), |venv| format!("into {venv}"));
    match app {
        "marimo" if missing("marimo", app) => out.push(format!("marimo, from pypi.org, {into}")),
        "jupyter" if missing("jupyter-lab", app) => {
            out.push(format!("JupyterLab, from pypi.org, {into}"));
        }
        "code" => {
            for (package, there) in
                [("ipykernel", python.imports("ipykernel")), ("marimo", python.has("marimo"))]
            {
                if python.venv().is_some() && !there {
                    out.push(format!(
                        "{package}, for notebooks in the editor, from pypi.org, {into}"
                    ));
                }
            }
            let open = server == "openvscode";
            if code_server(data, server).is_none() {
                out.push(if open {
                    format!("{OPENVSCODE}, from github.com/gitpod-io/openvscode-server")
                } else {
                    "the VS Code server, from update.code.visualstudio.com (Microsoft)".into()
                });
            }
            if first(NOTEBOOKS) {
                let shop = if open { "open-vsx.org" } else { "marketplace.visualstudio.com" };
                out.push(format!(
                    "the Python, Jupyter and marimo extensions for the editor, from {shop}"
                ));
            }
        }
        _ => {}
    }
    out
}

fn code_bin(data: &str, server: &str) -> String {
    if server == "openvscode" {
        format!("{data}/openvscode/{OPENVSCODE}-linux-x64/bin/openvscode-server")
    } else {
        format!("{data}/vscode/{VSCODE}/bin/code-server")
    }
}

pub fn code_server(data: &str, server: &str) -> Option<String> {
    let bin = code_bin(data, server);
    std::path::Path::new(&bin).exists().then_some(bin)
}

pub async fn install_extensions(env: &Env, program: &str, server_data: &str) -> bool {
    let mut args = vec!["--server-data-dir", server_data];
    for id in EXTENSIONS {
        args.extend(["--install-extension", id]);
    }
    run(program, &args, RunOptions::new(env).timeout_ms(600_000)).await.code == 0
}

pub async fn need_uv(env: &Env, data: &str, say: impl FnOnce(&str)) -> Result<(), String> {
    if which("uv", env).is_some() {
        return Ok(());
    }
    let bin = join(&[data, "bin"]);
    say(&bin);
    let (arch, sha) = match machine(env).await.as_str() {
        "x86_64" => ("x86_64", UV_SHA_X86_64),
        "aarch64" | "arm64" => ("aarch64", UV_SHA_AARCH64),
        other => return Err(format!("no uv is set up for {other} nodes")),
    };
    let url = format!(
        "https://github.com/astral-sh/uv/releases/download/{UV}/uv-{arch}-unknown-linux-musl.tar.gz"
    );
    let archive = fetch(env, &format!("uv {UV}"), &url, sha).await?;
    let rules = Rules { strip: 1, ..Default::default() };
    unpack(&archive, &bin, rules).map(|_| ()).map_err(|why| format!("cannot unpack uv: {why}"))
}

pub async fn editor(
    env: &Env,
    data: &str,
    server: &str,
    say: impl FnOnce(),
) -> Result<String, String> {
    let bin = code_bin(data, server);
    if which(&bin, env).is_some() {
        return Ok(bin);
    }
    say();
    let (what, url, sha, into, strip) = if server == "openvscode" {
        let url = format!(
            "https://github.com/gitpod-io/openvscode-server/releases/download/{OPENVSCODE}/{OPENVSCODE}-linux-x64.tar.gz"
        );
        (OPENVSCODE, url, OPENVSCODE_SHA, join(&[data, "openvscode"]), 0)
    } else {
        let url = format!(
            "https://update.code.visualstudio.com/commit:{VSCODE}/server-linux-x64-web/stable"
        );
        ("the VS Code server", url, VSCODE_SHA, join(&[data, "vscode", VSCODE]), 1)
    };
    x64_only(env, what).await?;
    std::fs::create_dir_all(&into).map_err(|e| format!("cannot create {into}: {e}"))?;
    let archive = fetch(env, what, &url, sha).await?;
    let rules = Rules { strip, links: true, ..Default::default() };
    unpack(&archive, &into, rules).map_err(|why| format!("cannot unpack {what}: {why}"))?;
    // Earlier versions ran the editor through its CLI, which kept servers of its own.
    let _ = std::fs::remove_dir_all(join(&[data, "vscode", "cli"]));
    Ok(bin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn only_the_pinned_file_reaches_the_disk() {
        let dir = tempfile::tempdir().unwrap();
        let at = |name: &str| dir.path().join(name).to_string_lossy().into_owned();
        std::fs::write(at("tool"), "the tool").unwrap();
        let env: Env = [("AICENTRE_FETCH_FROM".to_string(), format!("file://{}", at("")))].into();
        let url = "https://example.org/releases/tool";

        let wrong = fetch(&env, "tool", url, &"0".repeat(64)).await.unwrap_err();
        assert!(wrong.contains("is not the file this aicentre was built for (sha256 "), "{wrong}");

        let sum = sha256_hex(b"the tool");
        assert_eq!(fetch(&env, "tool", url, &sum).await.unwrap(), b"the tool");

        let none = Env::new();
        let plain = fetch(&none, "tool", "http://example.org/tool", &sum).await;
        assert_eq!(plain.unwrap_err(), "cannot download tool from http://example.org: not https");
        let local = format!("file://{}", at("tool"));
        assert!(fetch(&none, "tool", &local, &sum).await.is_err());
    }
}
