use crate::check::clusters::SITE;
use crate::ui::style::Style;

const DOCS: &str = "https://hpc.aicentre.dk/cli/";

type Row = (&'static str, &'static str);

struct Section {
    title: &'static str,
    rows: Vec<Row>,
    lines: Vec<&'static str>,
}

struct Page {
    summary: &'static str,
    usage: Vec<String>,
    sections: Vec<Section>,
    note: Option<String>,
}

fn rows(title: &'static str, rows: Vec<Row>) -> Section {
    Section { title, rows, lines: vec![] }
}

fn lines(title: &'static str, lines: Vec<&'static str>) -> Section {
    Section { title, rows: vec![], lines }
}

const PROVIDER: Row = ("-p, --provider <name>", "Only these providers: dtu, gefion, ngc");
const JSON_ROW: Row = ("--json", "Print JSON");
const ANOTHER_PROVIDER: Row = ("-p, --provider <name>", "Another provider than the first you use");

const SESSION_OPTIONS: [Row; 6] = [
    ("--gpu", "On a GPU node instead of the login node"),
    ("--time <2h>", "With --gpu: run limit, as 90, 2h or 1:30 (1h; max 4h)"),
    ("--cpus <n>", "With --gpu: CPU cores (default 4, the most p1i gives)"),
    ("--mem <8GB>", "With --gpu: memory per core (default 8GB)"),
    ("--another", "With --gpu: although a session of yours holds a GPU"),
    ANOTHER_PROVIDER,
];

const SESSION_NOTE: &str = "Without --gpu it runs on the login node: for editing, installs and light runs. With --gpu it is a job with one GPU on p1i, P1 DTU HPC's interactive queue, which also ends after 10 min with nothing connected. With no path it opens your home directory on the cluster; other paths are relative to your project directory. Ctrl-C ends the session.";

const CODE_NOTE: &str = "It opens your desktop editor over ssh when it finds one (VS Code and its forks with Remote-SSH, or Zed), or the command in [desktop] open in your settings. Otherwise VS Code opens in your browser. With --gpu the desktop editor needs --desktop. ";

const NO_BROWSER: Row = ("--no-browser", "Print the address without opening it");

fn session_page(app: &str) -> Page {
    let summary = match app {
        "shell" => "A shell on P1 compute, in this terminal",
        "marimo" => "marimo on P1 compute, in your browser",
        "jupyter" => "JupyterLab on P1 compute, in your browser",
        _ => "Your editor on P1 compute, or VS Code in your browser",
    };
    let extra: Vec<Row> = match app {
        "code" => vec![
            ("--desktop", "Your desktop editor; fail if it cannot open"),
            ("--browser", "VS Code in the browser, even with an editor here"),
            ("--server openvscode", "In the browser: the open build, with Open VSX"),
            NO_BROWSER,
        ],
        "shell" => vec![],
        _ => vec![NO_BROWSER],
    };
    let target = if app == "marimo" || app == "jupyter" { "notebook|folder" } else { "folder" };
    let mut options = SESSION_OPTIONS.to_vec();
    options.extend(extra);
    Page {
        summary,
        usage: vec![format!("aicentre {app} [{target}] [options]")],
        sections: vec![rows("Options", options)],
        note: Some(format!("{}{SESSION_NOTE}", if app == "code" { CODE_NOTE } else { "" })),
    }
}

fn page(topic: &str) -> Option<Page> {
    let p = |summary, usage: &[&str], sections, note: Option<&str>| Page {
        summary,
        usage: usage.iter().map(|u| u.to_string()).collect(),
        sections,
        note: note.map(str::to_string),
    };
    Some(match topic {
        "main" => p(
            "Work with P1 compute from your laptop",
            &["aicentre <command> [options]"],
            vec![
                rows(
                    "Interactive",
                    vec![
                        ("shell", "A shell on a login node, or a GPU node with --gpu"),
                        ("marimo", "marimo in your browser, with or without a GPU"),
                        ("jupyter", "JupyterLab in your browser, with or without a GPU"),
                        ("code", "Your desktop editor, or VS Code in your browser"),
                    ],
                ),
                rows(
                    "Jobs",
                    vec![
                        ("list jobs", "Your jobs across every provider"),
                        ("list gpus", "The GPUs and how busy they are"),
                    ],
                ),
                rows(
                    "Data",
                    vec![
                        ("push", "Copy code or data to the cluster"),
                        ("pull", "Copy results back from the cluster"),
                        ("gallery", "The site's examples; gallery get puts one on the cluster"),
                    ],
                ),
                rows("Assistant", vec![("mcp", "P1 tools for the agent tool you already use")]),
                rows(
                    "Setup",
                    vec![
                        ("check", "Check that this machine is ready for P1 compute"),
                        ("upgrade", "Update to the latest release"),
                        ("uninstall", "Remove aicentre"),
                    ],
                ),
                lines(
                    "Examples",
                    vec![
                        "aicentre marimo project/notebook.py",
                        "aicentre code project --gpu --time 2h",
                        "aicentre push datasets/ project",
                    ],
                ),
            ],
            Some(
                "A concept: illustrative for now, and it may change. It covers every provider you use. Add --help to any command for more, or --verbose to see the commands it runs.",
            ),
        ),
        "shell" | "marimo" | "jupyter" | "code" => session_page(topic),
        "push" => p(
            "Copy code or data to the cluster",
            &["aicentre push <local> [remote] [options]"],
            vec![rows("Options", vec![ANOTHER_PROVIDER])],
            Some(
                "Like cp -r: datasets/ lands in <remote>/datasets. Remote paths are relative to your project directory. It resumes where a stopped copy left off.",
            ),
        ),
        "pull" => p(
            "Copy results back from the cluster",
            &["aicentre pull <remote> [local] [options]"],
            vec![rows("Options", vec![ANOTHER_PROVIDER])],
            Some(
                "The remote path is relative to your project directory; local defaults to this folder.",
            ),
        ),
        "gallery" => p(
            "The examples at hpc.aicentre.dk/gallery",
            &["aicentre gallery", "aicentre gallery get <code> [dir] [options]"],
            vec![rows("Options", vec![ANOTHER_PROVIDER])],
            Some(
                "get has the login node download the example itself, into examples/<code> in your project directory, and prints what to run next. Nothing to clone.",
            ),
        ),
        "mcp" => p(
            "P1 tools for the agent tool you already use",
            &["aicentre mcp"],
            vec![lines(
                "Add it",
                vec![
                    "claude mcp add --scope user aicentre -- aicentre mcp",
                    "codex mcp add aicentre -- aicentre mcp",
                ],
            )],
            Some(
                "An MCP server, started by your agent tool. It gives the agent the P1 docs, your jobs and your setup. Its tools only read: it does not submit or cancel jobs. The docs show other agent tools.",
            ),
        ),
        "proxy" => p(
            "The ProxyCommand for desktop VS Code on a GPU node",
            &["aicentre proxy <host>"],
            vec![],
            Some(
                "ssh runs it, from the Host aicentre-gpu block that aicentre check prints. It reaches the sshd in the newest aicentre code --gpu session on this machine, or in job <id> for aicentre-gpu-<id>, through the login aicentre already holds.",
            ),
        ),
        "check" => p(
            "Check that this machine is ready for P1 compute",
            &["aicentre check [options]"],
            vec![rows(
                "Options",
                vec![
                    ("--connect", "Also log in: tests your key and your sign-in"),
                    ("--all", "Include providers not set up here, with what they need"),
                    PROVIDER,
                    JSON_ROW,
                ],
            )],
            Some("Looks at ~/.ssh, your keys and your Gefion sign-in. Exits 1 when a check fails."),
        ),
        "list" => p(
            "Your jobs and the GPUs across P1 compute",
            &["aicentre list jobs [options]", "aicentre list gpus [options]"],
            vec![rows("Options", vec![PROVIDER, JSON_ROW])],
            Some(
                "Asks every provider you use at once. NGC is air-gapped: use qstat on its remote desktop.",
            ),
        ),
        "upgrade" => p(
            "Update to the latest release",
            &["aicentre upgrade [options]"],
            vec![rows("Options", vec![("--version <v>", "Install this version instead")])],
            Some("Checks the download against its checksum, then deletes the version it replaces."),
        ),
        "uninstall" => p(
            "Remove aicentre from this machine",
            &["aicentre uninstall [options]"],
            vec![rows("Options", vec![("--purge", "Also remove your settings, without asking")])],
            Some(
                "Removes the program and its state, cache and runtime files. Asks before removing ~/.config/aicentre.",
            ),
        ),
        _ => return None,
    })
}

pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
            line = word.to_string();
        } else if line.is_empty() {
            line = word.to_string();
        } else {
            line = format!("{line} {word}");
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

const MARK: [&str; 3] = ["▀ ▀  ", "▀ ▀ ▀", "  ▀ ▀"];

fn mark_header(style: &Style, version: &str, summary: &str) -> Vec<String> {
    let text = [
        format!("{} {}", style.bold("aicentre"), style.dim(version)),
        summary.to_string(),
        style.dim(SITE.trim_start_matches("https://")),
    ];
    MARK.iter()
        .enumerate()
        .map(|(i, row)| {
            let mark = if i == 1 {
                let chars: Vec<char> = row.chars().collect();
                format!(
                    "{}{}{}",
                    chars[..2].iter().collect::<String>(),
                    style.accent("▀"),
                    chars[3..].iter().collect::<String>()
                )
            } else {
                row.to_string()
            };
            format!("{mark}  {}", text[i])
        })
        .collect()
}

pub fn render_help(topic: Option<&str>, style: &Style, version: &str) -> String {
    let known = topic.filter(|t| *t != "main").and_then(|t| page(t).map(|p| (t, p)));
    let (title, page, known) = match known {
        Some((t, p)) => {
            (format!("aicentre {}", if t == "list" { "list jobs" } else { t }), p, true)
        }
        None => ("aicentre".to_string(), page("main").unwrap(), false),
    };
    let mut out: Vec<String> = Vec::new();
    if !known && style.art {
        out.extend(mark_header(style, version, page.summary));
        out.push(String::new());
    } else {
        let ver = if known { String::new() } else { format!(" {}", style.dim(version)) };
        out.push(format!("{}{ver}  {}", style.bold(&title), style.dim(page.summary)));
        out.push(String::new());
    }
    for (i, u) in page.usage.iter().enumerate() {
        let label = if i == 0 { style.bold("Usage") } else { "     ".into() };
        out.push(format!("{label}  {u}"));
    }
    let width = page
        .sections
        .iter()
        .flat_map(|s| s.rows.iter().map(|(n, _)| n.chars().count()))
        .max()
        .unwrap_or(0);
    for section in &page.sections {
        out.push(String::new());
        out.push(style.bold(section.title));
        for (name, description) in &section.rows {
            out.push(format!("  {}   {description}", style.cyan(&format!("{name:<width$}"))));
        }
        for line in &section.lines {
            out.push(format!("  {} {line}", style.dim("$")));
        }
    }
    out.push(String::new());
    if let Some(note) = &page.note {
        out.extend(wrap(note, 78).iter().map(|l| style.dim(l)));
    }
    out.push(style.dim(&format!("Docs: {DOCS}")));
    out.join("\n") + "\n"
}
