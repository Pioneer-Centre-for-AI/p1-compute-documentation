use std::path::Path;

use crate::context::Env;
use crate::exec::{RunOptions, run};

/// The Python of the default environment. Packages with compiled parts,
/// such as torch, have builds for it. A newer one may not have them yet.
pub const PYTHON: &str = "3.13";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Python {
    Project,
    Own(String),
    Default(String),
}

/// Looks in the folder and then in the ones above it, as uv does, so a
/// notebook in `project/notebooks` runs in the project's environment. It
/// stops at the top of `roots`, the user's own directories.
pub fn choose(dir: &str, data: &str, roots: &[&str]) -> Python {
    let mut at = Path::new(dir);
    loop {
        if at.join("pyproject.toml").is_file() {
            return Python::Project;
        }
        let own = at.join(".venv");
        if own.join("bin/python").exists() {
            return Python::Own(own.to_string_lossy().into_owned());
        }
        match at.parent().filter(|up| roots.iter().any(|root| up.starts_with(root))) {
            Some(up) => at = up,
            None => return Python::Default(format!("{data}/venv")),
        }
    }
}

impl Python {
    pub fn venv(&self) -> Option<&str> {
        match self {
            Python::Project => None,
            Python::Own(venv) | Python::Default(venv) => Some(venv),
        }
    }

    pub fn exists(&self) -> bool {
        self.venv().is_none_or(|venv| Path::new(venv).join("bin/python").exists())
    }

    pub fn has(&self, program: &str) -> bool {
        self.venv().is_some_and(|venv| Path::new(venv).join("bin").join(program).exists())
    }

    pub fn imports(&self, package: &str) -> bool {
        let Some(venv) = self.venv() else { return false };
        let lib = Path::new(venv).join("lib");
        std::fs::read_dir(lib)
            .into_iter()
            .flatten()
            .flatten()
            .any(|python| python.path().join("site-packages").join(package).is_dir())
    }

    pub fn activate(&self, env: &mut Env) {
        let Some(venv) = self.venv() else { return };
        let path = env.get("PATH").cloned().unwrap_or_default();
        env.insert("PATH".into(), format!("{venv}/bin:{path}"));
        env.insert("VIRTUAL_ENV".into(), venv.to_string());
    }
}

async fn uv(env: &Env, args: &[&str]) -> Result<(), String> {
    let r = run("uv", args, RunOptions::new(env).timeout_ms(900_000)).await;
    if r.code == 0 {
        return Ok(());
    }
    let said = r.stderr.trim().lines().next_back().unwrap_or("uv is not installed").to_string();
    Err(said)
}

pub async fn ensure(env: &Env, python: &Python) -> Result<(), String> {
    let Python::Default(venv) = python else { return Ok(()) };
    if python.exists() {
        return Ok(());
    }
    uv(env, &["venv", "--python-preference", "only-managed", "--seed", "--python", PYTHON, venv])
        .await
        .map_err(|why| format!("cannot make the default Python environment in {venv}: {why}"))
}

pub async fn install(env: &Env, python: &Python, packages: &[&str]) -> Result<(), String> {
    let Some(venv) = python.venv() else { return Ok(()) };
    let mut args = vec!["pip", "install", "--python", venv];
    args.extend(packages);
    uv(env, &args)
        .await
        .map_err(|why| format!("cannot install {} into {venv}: {why}", packages.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_project_or_the_folders_own_environment_comes_before_the_default_one() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().to_string_lossy().into_owned();
        let default = Python::Default("/d/venv".into());
        assert_eq!(choose(&at, "/d", &[]), default);
        assert!(!default.exists() && !default.has("marimo") && !default.imports("ipykernel"));

        let own = format!("{at}/.venv");
        std::fs::create_dir_all(format!("{own}/bin")).unwrap();
        std::fs::write(format!("{own}/bin/python"), "").unwrap();
        std::fs::write(format!("{own}/bin/marimo"), "").unwrap();
        std::fs::create_dir_all(format!("{own}/lib/python3.13/site-packages/ipykernel")).unwrap();
        let chosen = choose(&at, "/d", &[]);
        assert_eq!(chosen, Python::Own(own.clone()));
        assert!(chosen.exists() && chosen.has("marimo") && chosen.imports("ipykernel"));
        assert!(!chosen.has("jupyter-lab") && !chosen.imports("torch"));

        let mut env: Env = [("PATH".to_string(), "/usr/bin".to_string())].into();
        chosen.activate(&mut env);
        assert_eq!(env["PATH"], format!("{own}/bin:/usr/bin"));
        assert_eq!(env["VIRTUAL_ENV"], own);

        std::fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        assert_eq!(choose(&at, "/d", &[]), Python::Project);

        let deep = format!("{at}/notebooks/drafts");
        std::fs::create_dir_all(&deep).unwrap();
        assert_eq!(choose(&deep, "/d", &[]), default);
        assert_eq!(choose(&deep, "/d", &[&deep]), default);
        assert_eq!(choose(&deep, "/d", &[&at]), Python::Project);
        std::fs::remove_file(dir.path().join("pyproject.toml")).unwrap();
        assert_eq!(choose(&deep, "/d", &[&at]), Python::Own(own));
        assert!(Python::Project.exists() && Python::Project.venv().is_none());
    }
}
