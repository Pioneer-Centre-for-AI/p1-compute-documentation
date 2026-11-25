use super::fetch::{NOTEBOOKS, editor, first, install_extensions, mark_started, need_uv};
use super::pyenv::{self, Python};
use crate::context::Env;

#[derive(Debug, PartialEq, Eq)]
pub struct Server {
    pub argv: Vec<String>,
    pub env: Vec<(String, String)>,
    pub path: String,
}

pub struct Asked<'a> {
    pub app: &'a str,
    pub server: &'a str,
    pub data: &'a str,
    pub dir: &'a str,
    pub file: Option<&'a str>,
    pub port: u16,
    pub token: &'a str,
    pub token_file: &'a str,
    pub in_job: bool,
    pub python: &'a Python,
}

fn program(a: &Asked, name: &str, package: &str) -> Vec<String> {
    match a.python.venv() {
        Some(venv) => vec![format!("{venv}/bin/{name}")],
        None => words(&["uv", "run", "--with", package, name]),
    }
}

fn words(text: &[&str]) -> Vec<String> {
    text.iter().map(|w| w.to_string()).collect()
}

pub fn urlencode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"._~/-".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

pub fn marimo(a: &Asked) -> Server {
    let mut argv = program(a, "marimo", "marimo");
    argv.push("edit".into());
    argv.extend(a.file.map(str::to_string));
    argv.extend(words(&["--headless", "--skip-update-check", "--host", "127.0.0.1", "--port"]));
    argv.extend([a.port.to_string(), "--token-password-file".into(), a.token_file.into()]);
    Server { argv, env: vec![], path: format!("/?access_token={}", a.token) }
}

pub fn jupyter(a: &Asked) -> Server {
    let mut argv = program(a, "jupyter-lab", "jupyterlab");
    argv.extend(words(&["--no-browser", "--ip", "127.0.0.1", "--port"]));
    argv.extend([a.port.to_string(), "--ServerApp.port_retries=0".into()]);
    if !a.in_job {
        argv.extend(words(&[
            "--MappingKernelManager.cull_idle_timeout=3600",
            "--ServerApp.shutdown_no_activity_timeout=3600",
        ]));
    }
    argv.push("--LabApp.check_for_updates_class=jupyterlab.NeverCheckForUpdate".into());
    let tree = a.file.map(|f| format!("/tree/{}", urlencode(f))).unwrap_or_default();
    Server {
        argv,
        // The token is in the environment, where other users cannot read it.
        env: vec![("JUPYTER_TOKEN".into(), a.token.into())],
        path: format!("/lab{tree}?token={}", a.token),
    }
}

pub fn code_data(a: &Asked) -> String {
    let name = if a.server == "openvscode" { "openvscode/data" } else { "vscode/server" };
    format!("{}/{name}", a.data)
}

fn point_python(a: &Asked) {
    let Some(venv) = a.python.venv() else { return };
    let dir = format!("{}/data/Machine", code_data(a));
    let file = format!("{dir}/settings.json");
    if std::path::Path::new(&file).exists() || std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let settings =
        serde_json::json!({ "python.defaultInterpreterPath": format!("{venv}/bin/python") });
    let _ = std::fs::write(file, format!("{settings:#}\n"));
}

async fn with_package(
    env: &Env,
    a: &Asked<'_>,
    package: &str,
    there: bool,
    status: &dyn Fn(&str),
) -> Result<(), String> {
    if a.python.venv().is_none() {
        return Ok(());
    }
    if !a.python.exists() {
        status(&format!("making the default Python {} environment", pyenv::PYTHON));
    }
    pyenv::ensure(env, a.python).await?;
    if !there {
        status(&format!("installing {package} (allow a minute)"));
        pyenv::install(env, a.python, &[package]).await?;
    }
    Ok(())
}

/// The editor asks before it trusts a folder, and the answer is kept by the
/// browser for one address, which is new in every session. A folder that
/// only this user can write to needs no such question. One that others can
/// write to keeps it: their settings there could run a program as this user.
fn own_alone(dir: &str) -> bool {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(dir)
        .is_ok_and(|m| m.uid() == nix::unistd::getuid().as_raw() && m.mode() & 0o022 == 0)
}

fn code(a: &Asked, program: &str) -> Server {
    let mut argv = vec![program.to_string()];
    if a.server != "openvscode" {
        argv.push("--accept-server-license-terms".into());
    }
    argv.extend(words(&["--host", "127.0.0.1", "--port"]));
    argv.extend([a.port.to_string(), "--connection-token-file".into(), a.token_file.into()]);
    argv.extend(["--server-data-dir".into(), code_data(a)]);
    argv.extend(words(&["--telemetry-level", "off", "--default-folder", a.dir]));
    if own_alone(a.dir) {
        argv.push("--disable-workspace-trust".into());
    }
    Server { argv, env: vec![], path: format!("/?tkn={}", a.token) }
}

pub async fn server(env: &Env, a: &Asked<'_>, status: &dyn Fn(&str)) -> Result<Server, String> {
    let uv = |bin: &str| status(&format!("installing uv into {bin}"));
    match a.app {
        "marimo" => {
            need_uv(env, a.data, uv).await?;
            with_package(env, a, "marimo", a.python.has("marimo"), status).await?;
            status("starting marimo");
            Ok(marimo(a))
        }
        "jupyter" => {
            need_uv(env, a.data, uv).await?;
            with_package(env, a, "jupyterlab", a.python.has("jupyter-lab"), status).await?;
            status("starting JupyterLab");
            Ok(jupyter(a))
        }
        "code" => {
            need_uv(env, a.data, uv).await?;
            with_package(env, a, "ipykernel", a.python.imports("ipykernel"), status).await?;
            with_package(env, a, "marimo", a.python.has("marimo"), status).await?;
            point_python(a);
            let program = editor(env, a.data, a.server, || status("fetching the editor")).await?;
            if first(a.data, NOTEBOOKS) {
                status("adding notebook support to the editor (first time)");
                if install_extensions(env, &program, &code_data(a)).await {
                    mark_started(a.data, NOTEBOOKS);
                }
            }
            status("starting the editor");
            Ok(code(a, &program))
        }
        other => Err(format!("unknown app {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asked<'a>(python: &'a Python, file: Option<&'a str>, in_job: bool) -> Asked<'a> {
        Asked {
            app: "jupyter",
            server: "vscode",
            data: "/d",
            dir: "/w",
            file,
            port: 4000,
            token: "s3cr3t",
            token_file: "/k/token",
            in_job,
            python,
        }
    }

    #[test]
    fn the_token_is_never_an_argument() {
        let shared = Python::Default("/d/venv".into());
        for in_job in [false, true] {
            let a = asked(&shared, Some("my nb.ipynb"), in_job);
            for s in [marimo(&a), jupyter(&a), code(&a, "/bin/code")] {
                assert!(s.argv.iter().all(|w| !w.contains("s3cr3t")), "{s:?}");
                assert!(s.path.ends_with("=s3cr3t"), "{s:?}");
            }
        }
        let a = asked(&shared, Some("my nb.ipynb"), false);
        assert_eq!(jupyter(&a).path, "/lab/tree/my%20nb.ipynb?token=s3cr3t");
        assert_eq!(jupyter(&a).env, [("JUPYTER_TOKEN".to_string(), "s3cr3t".to_string())]);
        assert!(jupyter(&a).argv.join(" ").starts_with(
            "/d/venv/bin/jupyter-lab --no-browser --ip 127.0.0.1 --port 4000 --ServerApp.port_retries=0 --MappingKernelManager.cull_idle_timeout=3600"
        ));
        assert!(!jupyter(&asked(&shared, None, true)).argv.join(" ").contains("cull_idle"));
        assert_eq!(
            marimo(&a).argv.join(" "),
            "/d/venv/bin/marimo edit my nb.ipynb --headless --skip-update-check --host 127.0.0.1 --port 4000 --token-password-file /k/token"
        );
        assert_eq!(urlencode("a b;$(id)/é.py"), "a%20b%3B%24%28id%29/%C3%A9.py");
    }

    #[test]
    fn the_program_comes_from_the_folders_environment_or_from_its_project() {
        let own = Python::Own("/w/.venv".into());
        assert!(marimo(&asked(&own, None, false)).argv[0] == "/w/.venv/bin/marimo");
        let project = asked(&Python::Project, None, false);
        assert!(marimo(&project).argv.join(" ").starts_with("uv run --with marimo marimo edit "));
        assert!(
            jupyter(&project).argv.join(" ").starts_with("uv run --with jupyterlab jupyter-lab ")
        );
        assert_eq!(code_data(&project), "/d/vscode/server");

        let home = tempfile::tempdir().unwrap();
        let dir = home.path().to_string_lossy().into_owned();
        let trusts = |dir: &str| {
            let a = Asked { dir, ..asked(&own, None, false) };
            code(&a, "/bin/code-server").argv.contains(&"--disable-workspace-trust".to_string())
        };
        assert!(trusts(&dir) && !trusts("/tmp") && !trusts("/nonexistent"));
    }
}
