use std::collections::BTreeMap;

use serde::Serialize;

use crate::check::clusters::{CLUSTER_IDS, ClusterId};
use crate::gallery::gallery::GalleryRequest;
use crate::session::interactive::InteractiveRequest;
use crate::session::lsf::{App, Server, View, gigabytes, is_memory, parse_minutes};

const MAX_MINUTES: u32 = 240;
const MAX_CPUS: i64 = 4;
const MAX_GB: f64 = 225.0;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "name", rename_all = "kebab-case")]
pub enum Command {
    Help {
        #[serde(skip_serializing_if = "Option::is_none")]
        topic: Option<String>,
    },
    Version,
    Check {
        clusters: Vec<ClusterId>,
        connect: bool,
        json: bool,
    },
    Upgrade {
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },
    Uninstall {
        purge: bool,
    },
    SelfInstall,
    List {
        what: String,
        clusters: Vec<ClusterId>,
        json: bool,
    },
    Session {
        session: InteractiveRequest,
    },
    Mcp,
    Proxy {
        host: String,
    },
    Transfer {
        direction: Direction,
        from: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider: Option<ClusterId>,
    },
    Gallery {
        gallery: GalleryRequest,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Push,
    Pull,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageError(pub String);

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for UsageError {}

type Result<T> = std::result::Result<T, UsageError>;

fn usage<T>(message: impl Into<String>) -> Result<T> {
    Err(UsageError(message.into()))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    String,
    Boolean,
}

#[derive(Clone, Copy)]
struct Opt {
    long: &'static str,
    kind: Kind,
    short: Option<char>,
    multiple: bool,
}

const fn flag(long: &'static str) -> Opt {
    Opt { long, kind: Kind::Boolean, short: None, multiple: false }
}

const fn string(long: &'static str) -> Opt {
    Opt { long, kind: Kind::String, short: None, multiple: false }
}

const PROVIDER: Opt =
    Opt { long: "provider", kind: Kind::String, short: Some('p'), multiple: true };
const HELP: Opt = Opt { long: "help", kind: Kind::Boolean, short: Some('h'), multiple: false };

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Flag,
    Text(String),
    List(Vec<String>),
}

#[derive(Default, Debug)]
struct Values(BTreeMap<String, Value>);

impl Values {
    fn flag(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    fn text(&self, name: &str) -> Option<&str> {
        match self.0.get(name) {
            Some(Value::Text(s)) => Some(s),
            _ => None,
        }
    }

    fn list(&self, name: &str) -> Vec<String> {
        match self.0.get(name) {
            Some(Value::List(v)) => v.clone(),
            _ => vec![],
        }
    }
}

struct Token {
    name: String,
    raw: String,
    value: Option<String>,
    inline: bool,
    grouped: bool,
}

enum Item {
    Opt(Token),
    Positional(String),
}

fn parse_args(
    args: &[String],
    options: &[Opt],
    allow_positionals: bool,
) -> Result<(Values, Vec<String>)> {
    let mut all: Vec<Opt> = options.to_vec();
    all.push(HELP);
    let find = |name: &str| all.iter().find(|o| o.long == name).copied();
    let long_for_short = |c: char| -> String {
        all.iter().find(|o| o.short == Some(c)).map_or(c.to_string(), |o| o.long.to_string())
    };
    let kind_of = |name: &str| find(name).map(|o| o.kind);

    let mut items = Vec::new();
    let mut remaining: std::collections::VecDeque<(String, bool)> =
        args.iter().map(|a| (a.clone(), false)).collect();
    while let Some((arg, grouped)) = remaining.pop_front() {
        let chars: Vec<char> = arg.chars().collect();
        if arg == "--" {
            items.extend(remaining.drain(..).map(|(a, _)| Item::Positional(a)));
            break;
        }
        let short_start = chars.len() >= 2 && chars[0] == '-' && chars[1] != '-';
        if short_start && chars.len() == 2 {
            let name = long_for_short(chars[1]);
            let mut value = None;
            if kind_of(&name) == Some(Kind::String) && !remaining.is_empty() {
                value = remaining.pop_front().map(|(v, _)| v);
            }
            items.push(Item::Opt(Token { name, raw: arg.clone(), value, inline: false, grouped }));
            continue;
        }
        if short_start && chars.len() > 2 {
            let first = long_for_short(chars[1]);
            if kind_of(&first) != Some(Kind::String) {
                // -fn5
                let mut expanded = Vec::new();
                for (i, c) in chars.iter().enumerate().skip(1) {
                    let name = long_for_short(*c);
                    if kind_of(&name) != Some(Kind::String) || i == chars.len() - 1 {
                        expanded.push(format!("-{c}"));
                    } else {
                        expanded.push(format!("-{}", chars[i..].iter().collect::<String>()));
                        break;
                    }
                }
                for e in expanded.into_iter().rev() {
                    remaining.push_front((e, true));
                }
                continue;
            }
            // -n5
            let value: String = chars[2..].iter().collect();
            items.push(Item::Opt(Token {
                name: first,
                raw: format!("-{}", chars[1]),
                value: Some(value),
                inline: true,
                grouped,
            }));
            continue;
        }
        if chars.len() > 2 && arg.starts_with("--") {
            if let Some(eq) = arg.get(3..).and_then(|r| r.find('=')).map(|i| i + 3) {
                let name = arg[2..eq].to_string();
                items.push(Item::Opt(Token {
                    raw: format!("--{name}"),
                    name,
                    value: Some(arg[eq + 1..].to_string()),
                    inline: true,
                    grouped: false,
                }));
            } else {
                let name = arg[2..].to_string();
                let mut value = None;
                if kind_of(&name) == Some(Kind::String) && !remaining.is_empty() {
                    value = remaining.pop_front().map(|(v, _)| v);
                }
                items.push(Item::Opt(Token {
                    name,
                    raw: arg.clone(),
                    value,
                    inline: false,
                    grouped: false,
                }));
            }
            continue;
        }
        items.push(Item::Positional(arg));
    }

    let mut values = Values::default();
    let mut positionals = Vec::new();
    for item in items {
        match item {
            Item::Positional(p) => {
                if !allow_positionals {
                    return usage(format!(
                        "Unexpected argument '{p}'. This command does not take positional arguments"
                    ));
                }
                positionals.push(p);
            }
            Item::Opt(t) => {
                let Some(opt) = find(&t.name) else {
                    let raw = if t.grouped { t.raw.trim_start_matches('-') } else { &t.raw };
                    return usage(format!("Unknown option '{raw}'"));
                };
                let short_and_long = match opt.short {
                    Some(c) => format!("-{c}, --{}", opt.long),
                    None => format!("--{}", opt.long),
                };
                if opt.kind == Kind::String && t.value.is_none() {
                    return usage(format!("Option '{short_and_long} <value>' argument missing"));
                }
                if opt.kind == Kind::Boolean && t.value.is_some() {
                    return usage(format!("Option '{short_and_long}' does not take an argument"));
                }
                if let Some(v) = &t.value
                    && !t.inline
                    && v.chars().count() > 1
                    && v.starts_with('-')
                {
                    let example = if t.raw.starts_with("--") {
                        format!("'{}=-XYZ'", t.raw)
                    } else {
                        format!("'--{}=-XYZ' or '{}-XYZ'", t.name, t.raw)
                    };
                    return usage(format!(
                        "Option '{raw}' argument is ambiguous.\nDid you forget to specify the option argument for '{raw}'?\nTo specify an option argument starting with a dash use {example}.",
                        raw = t.raw
                    ));
                }
                let value = match t.value {
                    None => Value::Flag,
                    Some(v) if opt.multiple => {
                        let mut list = values.list(opt.long);
                        list.push(v);
                        Value::List(list)
                    }
                    Some(v) => Value::Text(v),
                };
                values.0.insert(opt.long.to_string(), value);
            }
        }
    }
    Ok((values, positionals))
}

fn providers(values: &Values) -> Result<Vec<ClusterId>> {
    let mut named: Vec<ClusterId> = Vec::new();
    for raw in values.list("provider") {
        for part in raw.split(',') {
            let n = part.trim().to_lowercase();
            if n.is_empty() {
                continue;
            }
            let Some(id) = ClusterId::parse(&n) else {
                let ids = CLUSTER_IDS.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(", ");
                return usage(format!("unknown provider \"{n}\"; expected {ids}"));
            };
            if !named.contains(&id) {
                named.push(id);
            }
        }
    }
    Ok(named)
}

fn one_provider(values: &Values) -> Result<Option<ClusterId>> {
    let named = providers(values)?;
    if named.len() > 1 {
        return usage("this command works on one provider at a time");
    }
    Ok(named.first().copied())
}

fn no_positionals(command: &str, positionals: &[String]) -> Result<()> {
    let Some(first) = positionals.first() else { return Ok(()) };
    if ClusterId::parse(first).is_some() {
        return usage(format!(
            "aicentre {command} covers every provider you use; to narrow it, add --provider {first}"
        ));
    }
    usage(format!("aicentre {command} takes no argument \"{first}\""))
}

fn session(values: &Values, args: &[String], app: App) -> Result<Command> {
    if args.len() > 1 {
        return usage(format!("aicentre {} takes one path, not {}", app.as_str(), args.len()));
    }
    let gpu = values.flag("gpu");
    for f in ["time", "cpus", "mem"] {
        if values.text(f).is_some() && !gpu {
            return usage(format!(
                "--{f} sizes a --gpu session; a login-node session runs until you end it"
            ));
        }
    }
    let time = values.text("time");
    let Some(minutes) = parse_minutes(time.unwrap_or("60")) else {
        return usage(format!(
            "--time \"{}\" is not a duration like 90, 2h or 1:30",
            time.unwrap_or("undefined")
        ));
    };
    if minutes > MAX_MINUTES {
        return usage(format!(
            "--time \"{}\" is more than {} h, the longest a --gpu session runs; longer work is a batch job",
            time.unwrap_or(""),
            MAX_MINUTES / 60
        ));
    }
    let cpus_text = values.text("cpus");
    let cpus = match cpus_text {
        None => Some(4),
        Some(t) => t.trim().parse::<i64>().ok().filter(|c| *c >= 1),
    };
    let Some(cpus) = cpus else {
        return usage(format!("--cpus \"{}\" is not a number of CPUs", cpus_text.unwrap_or("")));
    };
    if cpus > MAX_CPUS {
        return usage(format!(
            "--cpus \"{cpus}\" is more than {MAX_CPUS}, the most DTU gives a session on p1i"
        ));
    }
    let mem_text = values.text("mem");
    let mem_per_cpu = mem_text.unwrap_or("8GB").to_uppercase();
    if !is_memory(&mem_per_cpu) {
        return usage(format!("--mem \"{}\" is not a size like 8GB", mem_text.unwrap_or("")));
    }
    let total = gigabytes(&mem_per_cpu).unwrap_or(0.0) * cpus as f64;
    if total > MAX_GB {
        return usage(format!(
            "--mem {mem_per_cpu} for each of {cpus} CPUs is {total:.0} GB, more than {MAX_GB:.0} GB, the most DTU gives a session on p1i"
        ));
    }
    let server = match values.text("server").unwrap_or("vscode") {
        "vscode" => Server::Vscode,
        "openvscode" => Server::Openvscode,
        other => return usage(format!("--server \"{other}\" is not vscode or openvscode")),
    };
    let has_server = values.text("server").is_some_and(|s| !s.is_empty());
    if has_server && app != App::Code {
        return usage("--server only applies to aicentre code");
    }
    let no_browser = values.flag("no-browser");
    if no_browser && app == App::Shell {
        return usage("--no-browser does not apply to a shell");
    }
    for f in ["desktop", "browser"] {
        if values.flag(f) && app != App::Code {
            return usage(format!("--{f} only applies to aicentre code"));
        }
    }
    let (desktop, browser) = (values.flag("desktop"), values.flag("browser"));
    if desktop && browser {
        return usage("choose --desktop or --browser, not both");
    }
    if desktop && has_server {
        return usage("--server picks the browser server; leave it out with --desktop");
    }
    if desktop && no_browser {
        return usage("--no-browser is for the browser; leave it out with --desktop");
    }
    let provider = one_provider(values)?;
    let view = if app != App::Code || browser || has_server || no_browser {
        View::Browser
    } else if desktop {
        View::Desktop
    } else {
        View::Auto
    };
    Ok(Command::Session {
        session: InteractiveRequest {
            app,
            path: args.first().cloned().unwrap_or_default(),
            gpu,
            provider,
            server,
            minutes,
            cpus: cpus as u32,
            mem_per_cpu,
            browser: !no_browser,
            view,
            another: values.flag("another"),
        },
    })
}

fn transfer(values: &Values, args: &[String], direction: Direction) -> Result<Command> {
    if args.is_empty() || args.len() > 2 {
        return usage(match direction {
            Direction::Push => {
                "aicentre push takes a local path, and where on the cluster to put it"
            }
            Direction::Pull => {
                "aicentre pull takes a path on the cluster, and where to put it here"
            }
        });
    }
    let provider = one_provider(values)?;
    Ok(Command::Transfer { direction, from: args[0].clone(), to: args.get(1).cloned(), provider })
}

const SESSION_OPTIONS: &[Opt] = &[
    flag("gpu"),
    flag("another"),
    string("time"),
    string("cpus"),
    string("mem"),
    string("server"),
    flag("no-browser"),
    flag("desktop"),
    flag("browser"),
    PROVIDER,
];

const CHECK: &[Opt] = &[flag("connect"), flag("json"), flag("all"), PROVIDER];
const UPGRADE: &[Opt] = &[string("version")];
const UNINSTALL: &[Opt] = &[flag("purge")];
const LIST: &[Opt] = &[flag("json"), PROVIDER];
const ONLY_PROVIDER: &[Opt] = &[PROVIDER];

fn spec(name: &str) -> Option<(&'static [Opt], bool)> {
    Some(match name {
        "check" => (CHECK, true),
        "upgrade" => (UPGRADE, false),
        "uninstall" => (UNINSTALL, false),
        "list" => (LIST, true),
        "shell" | "marimo" | "jupyter" | "code" => (SESSION_OPTIONS, true),
        "push" | "pull" | "gallery" => (ONLY_PROVIDER, true),
        "mcp" => (&[], false),
        "proxy" => (&[], true),
        _ => return None,
    })
}

fn build(name: &str, values: &Values, args: &[String]) -> Result<Command> {
    match name {
        "check" => {
            no_positionals("check", args)?;
            let named = providers(values)?;
            let all = values.flag("all");
            if all && !named.is_empty() {
                return usage("--all already checks every provider; leave out --provider");
            }
            Ok(Command::Check {
                clusters: if all { CLUSTER_IDS.to_vec() } else { named },
                connect: values.flag("connect"),
                json: values.flag("json"),
            })
        }
        "upgrade" => Ok(Command::Upgrade { version: values.text("version").map(str::to_string) }),
        "uninstall" => Ok(Command::Uninstall { purge: values.flag("purge") }),
        "list" => {
            let what = args.first().map(String::as_str);
            let Some(what) = what.filter(|w| ["jobs", "gpus"].contains(w)) else {
                return usage(match what {
                    Some(w) if !w.is_empty() => {
                        format!("cannot list \"{w}\"; try aicentre list jobs or aicentre list gpus")
                    }
                    _ => "aicentre list needs something to list: jobs or gpus".into(),
                });
            };
            no_positionals(&format!("list {what}"), &args[1..])?;
            Ok(Command::List {
                what: what.into(),
                clusters: providers(values)?,
                json: values.flag("json"),
            })
        }
        "shell" | "marimo" | "jupyter" | "code" => session(values, args, App::parse(name).unwrap()),
        "mcp" => Ok(Command::Mcp),
        "proxy" => {
            if args.len() != 1 {
                return usage(
                    "aicentre proxy takes the ssh host name, as in ProxyCommand aicentre proxy %n",
                );
            }
            Ok(Command::Proxy { host: args[0].clone() })
        }
        "push" => transfer(values, args, Direction::Push),
        "pull" => transfer(values, args, Direction::Pull),
        "gallery" => {
            let action = args.first().map(String::as_str);
            if action.is_none() || action == Some("list") {
                return Ok(Command::Gallery { gallery: GalleryRequest::List });
            }
            if action != Some("get") {
                return usage(format!(
                    "aicentre gallery takes get <code>, not \"{}\"",
                    action.unwrap_or("")
                ));
            }
            let code = args.get(1).filter(|c| !c.is_empty());
            let (Some(code), true) = (code, args.len() <= 3) else {
                return usage("aicentre gallery get takes an example code, and where to put it");
            };
            let provider = one_provider(values)?;
            Ok(Command::Gallery {
                gallery: GalleryRequest::Get {
                    code: code.clone(),
                    dir: args.get(2).cloned(),
                    provider,
                },
            })
        }
        _ => usage(format!("unknown command \"{name}\"")),
    }
}

pub fn parse_command(argv: &[String]) -> Result<Command> {
    let Some(first) = argv.first() else { return Ok(Command::Help { topic: None }) };
    let rest = &argv[1..];
    match first.as_str() {
        "help" => return Ok(Command::Help { topic: rest.first().cloned() }),
        "-h" | "--help" => return Ok(Command::Help { topic: None }),
        "-V" | "--version" | "version" => return Ok(Command::Version),
        "chat" => {
            return usage(
                "chat is now aicentre mcp, added to the agent tool you already use; see aicentre help mcp",
            );
        }
        "gpu" => {
            return usage(
                "interactive GPU sessions are now aicentre shell, marimo, jupyter or code with --gpu",
            );
        }
        "submit" => {
            return usage(
                "submit is gone; in aicentre shell, submit with bsub on DTU or sbatch on Gefion",
            );
        }
        "logs" => return usage("logs is gone; in aicentre shell, read the job's output file"),
        "cancel" => {
            return usage(
                "cancel is gone; Ctrl-C ends a session, and in aicentre shell bkill or scancel ends a job",
            );
        }
        "self-install" => {
            let (values, _) = parse_args(rest, &[], false)?;
            if values.flag("help") {
                return Ok(Command::Help { topic: None });
            }
            return Ok(Command::SelfInstall);
        }
        _ => {}
    }
    let name = if first == "update" { "upgrade" } else { first.as_str() };
    let Some((options, positionals)) = spec(name) else {
        return usage(format!("unknown command \"{first}\""));
    };
    let (values, args) = parse_args(rest, options, positionals)?;
    if values.flag("help") {
        return Ok(Command::Help { topic: Some(name.to_string()) });
    }
    build(name, &values, &args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(argv: &[&str]) -> Result<Command> {
        parse_command(&argv.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn commands() {
        assert_eq!(parse(&[]).unwrap(), Command::Help { topic: None });
        assert_eq!(
            parse(&["check", "-p", "gefion,NGC", "--connect"]).unwrap(),
            Command::Check {
                clusters: vec![ClusterId::Gefion, ClusterId::Ngc],
                connect: true,
                json: false
            }
        );
        assert_eq!(
            parse(&["update", "--version", "0.2.0"]).unwrap(),
            Command::Upgrade { version: Some("0.2.0".into()) }
        );
        assert_eq!(parse(&["mcp", "--help"]).unwrap(), Command::Help { topic: Some("mcp".into()) });
        assert_eq!(parse(&["self-install", "--help"]).unwrap(), Command::Help { topic: None });
    }

    #[test]
    fn mistakes_are_usage_errors() {
        for argv in [
            &["check", "-p", "lumi"][..],
            &["check", "--all", "-p", "dtu"],
            &["check", "--conect"],
            &["list"],
            &["mcp", "serve"],
            &["shell", "--time", "2h"],
            &["code", "--gpu", "--mem", "lots"],
            &["code", "-p", "dtu,gefion"],
            &["jupyter", "--server", "openvscode"],
            &["gallery", "fetch", "x"],
            &["gpu", "shell"],
            &["chat", "--model", "x"],
            &["frobnicate"],
        ] {
            assert!(parse(argv).is_err(), "{argv:?}");
        }
        for (argv, says) in [
            (&["jupyter", "--gpu", "--time", "5h"][..], "is more than 4 h"),
            (&["jupyter", "--gpu", "--cpus", "5"], "is more than 4"),
            (&["jupyter", "--gpu", "--cpus", "4", "--mem", "64GB"], "is 256 GB, more than 225 GB"),
        ] {
            assert!(parse(argv).unwrap_err().0.contains(says), "{argv:?}");
        }
        assert!(
            parse(&["jupyter", "--gpu", "--time", "4h", "--cpus", "4", "--mem", "56GB"]).is_ok()
        );
        for gone in ["submit", "logs", "cancel"] {
            let e = parse(&[gone, "--help"]).unwrap_err();
            assert!(e.0.starts_with(&format!("{gone} is gone; ")), "{}", e.0);
        }
        assert!(
            parse(&["list", "jobs", "gefion"]).unwrap_err().0.contains("add --provider gefion")
        );
    }
}
