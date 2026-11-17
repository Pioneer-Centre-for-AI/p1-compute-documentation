use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use regex::Regex;
use serde_json::{Value, json};

use crate::BoxFuture;
use crate::check::clusters::{ClusterId, SITE};
use crate::check::command::check;
use crate::context::{Captured, Ctx, Output};
use crate::http::Fetched;
use crate::jobs::list::list_jobs;

pub const MCP_INSTRUCTIONS: &str = r#"The p1_* tools come from aicentre, the command line tool for P1 compute. They help researchers at the Pioneer Centre for AI use P1 compute, which is three providers:

- P1 DTU HPC (dtu): LSF. Queue p1 runs batch GPU jobs on seven H100 nodes; p1i is the one queue for interactive GPU work. Project storage is /dtu/p1/<username>; home directories hold 30 GB. login9 takes SSH from the internet; off DTU's network it asks for the DTU password after the key, and on DTU's network or the DTU VPN the key is enough.
- P1 Gefion HPC (gefion): Slurm, reached through Teleport (tsh login). GPU work runs as batch jobs only, charged to P1's Slurm account iu_0096 unless a project has its own.
- P1 NGC HPC (ngc): for sensitive data, air-gapped, reached only through its remote desktop.

How to help:
- Answer from the documentation with p1_docs rather than from memory, and say which page you used. If the docs do not say, say so and suggest the compute coordinator.
- Look at the user's situation with p1_jobs and p1_check before giving advice about it.
- aicentre does not submit, follow or cancel batch jobs. The user does that in aicentre shell: bsub, bjobs and bkill on DTU, sbatch, squeue and scancel on Gefion. You can write a job script as a file in the working directory, with #BSUB lines for DTU or #SBATCH lines for Gefion, and copy it up with aicentre push.
- Interactive sessions are started by the user in another terminal: aicentre shell, aicentre marimo [notebook], aicentre jupyter [notebook] or aicentre code [folder], with --gpu for a GPU on p1i. aicentre code opens the user's desktop editor over ssh when one is installed here, and VS Code in the browser otherwise. Suggest the exact command.
- If a tool says a provider wants a password, tell the user to run any aicentre command in another terminal, such as aicentre list jobs, which logs in once for 30 minutes, and then try again.
- Be brief and concrete. Use the user's own paths and job ids."#;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Answer {
    pub text: String,
    pub code: Option<i32>,
}

impl Answer {
    fn say(text: impl Into<String>) -> Answer {
        Answer { text: text.into(), code: None }
    }
}

pub type Fetcher = Arc<dyn Fn(String) -> BoxFuture<'static, Result<Fetched, String>> + Send + Sync>;

pub fn web_fetcher() -> Fetcher {
    Arc::new(|url: String| {
        Box::pin(async move { crate::http::get(&url, Duration::from_secs(20)).await })
    })
}

pub const READ_ONLY: &str = "AICENTRE_READ_ONLY";

pub struct P1Tool {
    pub name: &'static str,
    pub label: &'static str,
    pub description: String,
    pub parameters: Value,
}

fn quiet(ctx: &Ctx) -> (Ctx, Captured) {
    let (out, text) = Output::capture(false, Some(100));
    let mut c = ctx.clone();
    c.out = out.clone();
    c.err = out;
    c.interactive = false;
    c.env.insert("NO_COLOR".into(), "1".into());
    c.env.insert(READ_ONLY.into(), "1".into());
    (c, text)
}

fn said(text: &Captured, code: i32) -> Answer {
    Answer { text: text.text().trim().to_string(), code: Some(code) }
}

fn providers(list: &Value) -> Vec<ClusterId> {
    let items = list.as_array().map(Vec::as_slice).unwrap_or_default();
    items.iter().filter_map(|p| p.as_str().and_then(ClusterId::parse)).collect()
}

static SCHEME_HOST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^https?://[^/]+").unwrap());

pub async fn fetch_docs(path: &str, fetcher: &Fetcher) -> Result<String, String> {
    let bare = SCHEME_HOST.replace(path, "");
    let trimmed = bare.trim_start_matches('/').trim_end_matches('/');
    let clean = trimmed.strip_suffix(".md").unwrap_or(trimmed);
    let url =
        if clean.is_empty() { format!("{SITE}/llms.txt") } else { format!("{SITE}/{clean}.md") };
    let res = fetcher(url.clone()).await?;
    if !res.ok() {
        return Err(format!("{url}: HTTP {}", res.status));
    }
    Ok(format!("Source: {url}\n\n{}", res.text()))
}

fn providers_schema() -> Value {
    json!({
        "type": "array",
        "items": {
            "anyOf": [
                { "type": "string", "const": "dtu" },
                { "type": "string", "const": "gefion" },
                { "type": "string", "const": "ngc" }
            ]
        },
        "description": "Only these providers; leave out for every provider the user is set up for"
    })
}

fn list() -> Vec<P1Tool> {
    vec![
        P1Tool {
            name: "p1_docs",
            label: "P1 docs",
            description: "Read the P1 compute documentation at hpc.aicentre.dk. With no path, returns the index of every page. With a path such as \"providers/dtu\", \"providers/gefion\", \"providers/ngc\", \"cli\", \"forms\" or \"gallery/int-gpu\", returns that page as markdown.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Page path, e.g. providers/dtu; empty for the index"
                    }
                }
            }),
        },
        P1Tool {
            name: "p1_jobs",
            label: "P1 jobs",
            description: "The user's running, waiting and suspended jobs across P1 compute, as JSON: provider, id, state, queue, name, node, elapsed, time left, and why a job waits.".into(),
            parameters: json!({
                "type": "object",
                "properties": { "providers": providers_schema() }
            }),
        },
        P1Tool {
            name: "p1_check",
            label: "P1 setup check",
            description: "Whether this machine is set up for P1 compute: OpenSSH, ~/.ssh/config Host blocks, keys, the Teleport login for Gefion. As JSON, with a hint for every problem. connect also logs in to each provider.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "providers": providers_schema(),
                    "connect": {
                        "type": "boolean",
                        "description": "Also reach each provider over the network and log in"
                    }
                }
            }),
        },
    ]
}

const FRESH: Duration = Duration::from_secs(60);
const RETRY: Duration = Duration::from_secs(15);

pub struct P1Tools {
    ctx: Ctx,
    fetcher: Fetcher,
    pub tools: Vec<P1Tool>,
    asked: tokio::sync::Mutex<HashMap<String, (Instant, Duration, Answer)>>,
}

impl P1Tools {
    pub fn new(ctx: &Ctx, fetcher: Fetcher) -> P1Tools {
        P1Tools { ctx: ctx.clone(), fetcher, tools: list(), asked: Default::default() }
    }

    pub fn get(&self, name: &str) -> Option<&P1Tool> {
        self.tools.iter().find(|t| t.name == name)
    }

    /// `params` must match the tool's schema.
    pub async fn run(&self, name: &str, params: &Value) -> anyhow::Result<Answer> {
        let text = |key: &str| params.get(key).and_then(Value::as_str);
        let flag = |key: &str| params.get(key).and_then(Value::as_bool).unwrap_or(false);
        let ctx = &self.ctx;
        match name {
            "p1_docs" => {
                Ok(Answer::say(match fetch_docs(text("path").unwrap_or(""), &self.fetcher).await {
                    Ok(page) => page,
                    Err(e) => format!("Could not read the docs: {e}"),
                }))
            }
            "p1_jobs" | "p1_check" => {
                let asked = providers(&params["providers"]);
                let key = format!("{name} {asked:?} {}", flag("connect"));
                let mut seen = self.asked.lock().await;
                if let Some((at, keep, answer)) = seen.get(&key)
                    && at.elapsed() < *keep
                {
                    return Ok(answer.clone());
                }
                let (c, out) = quiet(ctx);
                let code = if name == "p1_jobs" {
                    list_jobs(&c, &asked, true).await?
                } else {
                    check(&c, &asked, flag("connect"), true).await?
                };
                let answer = said(&out, code);
                let keep = if code == 0 { FRESH } else { RETRY };
                seen.insert(key, (Instant::now(), keep, answer.clone()));
                Ok(answer)
            }
            _ => anyhow::bail!("Unknown tool: {name}"),
        }
    }
}
