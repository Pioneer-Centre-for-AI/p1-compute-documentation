use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

use crate::exec::shq;

pub use crate::exec::login_shell;

pub const QUEUE: &str = "p1i";
pub const JOB_PREFIX: &str = "aicentre-";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum App {
    Shell,
    Marimo,
    Code,
    Jupyter,
}

impl App {
    pub fn as_str(self) -> &'static str {
        match self {
            App::Shell => "shell",
            App::Marimo => "marimo",
            App::Code => "code",
            App::Jupyter => "jupyter",
        }
    }

    pub fn parse(s: &str) -> Option<App> {
        [App::Shell, App::Marimo, App::Code, App::Jupyter].into_iter().find(|a| a.as_str() == s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Server {
    Vscode,
    Openvscode,
}

impl Server {
    pub fn as_str(self) -> &'static str {
        match self {
            Server::Vscode => "vscode",
            Server::Openvscode => "openvscode",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum View {
    Auto,
    Desktop,
    Browser,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub app: App,
    pub path: String,
    pub server: Server,
    pub view: Option<View>,
    pub minutes: u32,
    pub cpus: u32,
    pub mem_per_cpu: String,
}

static MINUTES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)m?$").unwrap());
static CLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+):(\d{2})$").unwrap());
static HOURS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:(\d+)h)?(?:(\d+)m)?$").unwrap());
static MEMORY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\d+(?:\.\d+)?[KMGT]B$").unwrap());

fn num(s: Option<regex::Match>) -> u64 {
    s.and_then(|m| m.as_str().parse::<u64>().ok()).unwrap_or(0)
}

fn nonzero(n: u64) -> Option<u32> {
    (n > 0).then(|| n.min(u32::MAX as u64) as u32)
}

pub fn parse_minutes(text: &str) -> Option<u32> {
    let t = text.trim().to_lowercase();
    if let Some(m) = MINUTES.captures(&t) {
        return nonzero(num(m.get(1)));
    }
    if let Some(m) = CLOCK.captures(&t) {
        return nonzero(num(m.get(1)) * 60 + num(m.get(2)));
    }
    if let Some(m) = HOURS.captures(&t)
        && (m.get(1).is_some() || m.get(2).is_some())
    {
        return nonzero(num(m.get(1)) * 60 + num(m.get(2)));
    }
    None
}

pub fn is_memory(text: &str) -> bool {
    MEMORY.is_match(text)
}

pub fn gigabytes(text: &str) -> Option<f64> {
    let t = text.trim().to_uppercase();
    let (value, unit) = t.split_at(t.len().checked_sub(2)?);
    let scale = match unit {
        "KB" => 1e-6,
        "MB" => 1e-3,
        "GB" => 1.0,
        "TB" => 1e3,
        _ => return None,
    };
    value.parse::<f64>().ok().map(|v| v * scale)
}

pub fn hm(minutes: u32) -> String {
    format!("{}:{:02}", minutes / 60, minutes % 60)
}

pub fn bsub_command(
    r: &Request,
    launcher: &str,
    data: &str,
    base: &str,
    interactive_flag: &str,
) -> String {
    let mut args: Vec<String> = [
        interactive_flag,
        "-q",
        QUEUE,
        "-app",
        "h100app",
        "-J",
        &format!("{JOB_PREFIX}{}", r.app.as_str()),
        "-env",
        "none",
        "-W",
        &hm(r.minutes),
        "-n",
        &r.cpus.to_string(),
        "-R",
        &format!("rusage[mem={}]", r.mem_per_cpu),
        "-R",
        "span[hosts=1]",
        "-gpu",
        "num=1:mode=shared",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    args.extend(
        launcher_args(r.app, &r.path, r.server, r.view, launcher, data, base, "0.0.0.0")
            .into_iter()
            .map(|a| if shq(&a) == a { a } else { format!("hex:{}", hex::encode(&a)) }),
    );
    let quoted: Vec<String> = args.iter().map(|a| shq(a)).collect();
    format!("ESUB_BYPASS=1 ESUB_QUIET=1 bsub {}", quoted.join(" "))
}

#[allow(clippy::too_many_arguments)]
pub fn launcher_args(
    app: App,
    path: &str,
    server: Server,
    view: Option<View>,
    launcher: &str,
    data: &str,
    base: &str,
    bind: &str,
) -> Vec<String> {
    let app = if view == Some(View::Desktop) { "sshd" } else { app.as_str() };
    ["bash", "-l", launcher, app, path, server.as_str(), data, base, bind]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Submitted { jobid: String },
    Queued,
    Busy,
    Started { host: String },
    Tag { name: String, value: String },
    Text { line: String },
}

static SUBMITTED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Job <(\d+)> is submitted").unwrap());
static STARTED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^<<Starting on (.+?)>>").unwrap());
static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^@@([A-Z]+) ?(.*)$").unwrap());

pub fn parse_line(line: &str) -> Event {
    if let Some(m) = SUBMITTED.captures(line) {
        return Event::Submitted { jobid: m[1].to_string() };
    }
    if line.starts_with("<<Waiting for dispatch") {
        return Event::Queued;
    }
    if line.starts_with("LSF is processing") {
        return Event::Busy;
    }
    if let Some(m) = STARTED.captures(line) {
        return Event::Started { host: m[1].to_string() };
    }
    if let Some(m) = TAG.captures(line) {
        return Event::Tag {
            name: m[1].to_string(),
            value: m.get(2).map_or("", |v| v.as_str()).to_string(),
        };
    }
    Event::Text { line: line.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(parse_minutes("90"), Some(90));
        assert_eq!(parse_minutes("90m"), Some(90));
        assert_eq!(parse_minutes("1:30"), Some(90));
        assert_eq!(parse_minutes("2H"), Some(120));
        assert_eq!(parse_minutes("1h30m"), Some(90));
        assert_eq!(parse_minutes("0"), None);
        assert_eq!(parse_minutes("abc"), None);
        assert_eq!(parse_minutes(""), None);
        assert!(is_memory("8GB") && is_memory("0.5tb") && !is_memory("8"));
    }

    #[test]
    fn the_job_command_holds_no_character_lsf_would_parse_again() {
        let r = Request {
            app: App::Jupyter,
            path: "my  nb;id $(x).ipynb".into(),
            server: Server::Vscode,
            view: None,
            minutes: 60,
            cpus: 4,
            mem_per_cpu: "8GB".into(),
        };
        let line = bsub_command(&r, "/d/session.sh", "/d", "/b", "-I");
        let job = line.split(" num=1:mode=shared ").nth(1).unwrap();
        assert_eq!(
            job,
            format!(
                "bash -l /d/session.sh jupyter hex:{} vscode /d /b 0.0.0.0",
                hex::encode("my  nb;id $(x).ipynb")
            )
        );
        let r = Request { path: "~".into(), ..r };
        assert!(bsub_command(&r, "l", "d", "b", "-I").contains(" jupyter hex:7e vscode "));
    }

    #[test]
    fn lines() {
        assert_eq!(
            parse_line("Job <123> is submitted to queue <p1i>."),
            Event::Submitted { jobid: "123".into() }
        );
        assert_eq!(
            parse_line("<<Starting on n-62-30-1>>"),
            Event::Started { host: "n-62-30-1".into() }
        );
        assert_eq!(parse_line("@@PATH"), Event::Tag { name: "PATH".into(), value: "".into() });
        assert_eq!(
            parse_line("@@PORT 4000"),
            Event::Tag { name: "PORT".into(), value: "4000".into() }
        );
    }
}
