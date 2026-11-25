use std::collections::BTreeMap;

use serde::Serialize;

use crate::check::clusters::{CLUSTER_IDS, ClusterId};
use crate::gallery::gallery::GalleryRequest;
use crate::launch::LaunchRequest;
use crate::session::interactive::InteractiveRequest;
use crate::session::lsf::{App, Server, View, gigabytes, is_memory, parse_minutes};

/// What a --gpu session asks p1i for, by name. Memory is for each of the 4
/// cores, as LSF counts it: 32, 96 and 200 GB in all. The largest stays
/// under the 225 GB at which the queue's profile ends a job.
pub struct Size {
    pub name: &'static str,
    pub gpus: u32,
    pub time: &'static str,
    pub mem_per_cpu: &'static str,
}

pub const SIZE_CPUS: u32 = 4;

pub const SIZES: [Size; 3] = [
    Size { name: "small", gpus: 1, time: "2h", mem_per_cpu: "8GB" },
    Size { name: "medium", gpus: 1, time: "4h", mem_per_cpu: "24GB" },
    Size { name: "large", gpus: 2, time: "4h", mem_per_cpu: "50GB" },
];

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
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        keep_current: bool,
    },
    Uninstall {
        purge: bool,
    },
    SelfInstall {
        keep_current: bool,
    },
    Launch {
        launch: LaunchRequest,
    },
    UnpackExample {
        url: String,
        slug: String,
        parent: String,
    },
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

#[derive(Clone, Copy)]
struct Opt {
    long: &'static str,
    takes_value: bool,
    short: Option<char>,
}

const fn flag(long: &'static str) -> Opt {
    Opt { long, takes_value: false, short: None }
}

const fn string(long: &'static str) -> Opt {
    Opt { long, takes_value: true, short: None }
}

const PROVIDER: Opt = Opt { long: "provider", takes_value: true, short: Some('p') };
const HELP: Opt = Opt { long: "help", takes_value: false, short: Some('h') };

#[derive(Default)]
struct Values(BTreeMap<&'static str, Vec<String>>);

impl Values {
    fn flag(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    fn text(&self, name: &str) -> Option<&str> {
        self.0.get(name)?.last().map(String::as_str)
    }

    fn list(&self, name: &str) -> &[String] {
        self.0.get(name).map_or(&[], Vec::as_slice)
    }
}

fn parse_args(
    args: &[String],
    options: &[Opt],
    allow_positionals: bool,
) -> Result<(Values, Vec<String>)> {
    let known = || options.iter().chain([&HELP]);
    let mut values = Values::default();
    let mut positionals = Vec::new();
    let mut rest = args.iter().peekable();
    while let Some(arg) = rest.next() {
        let (opt, inline) = if arg == "--" {
            positionals.extend(rest.by_ref().cloned());
            break;
        } else if let Some(long) = arg.strip_prefix("--") {
            let (name, value) = long.split_once('=').map_or((long, None), |(n, v)| (n, Some(v)));
            (known().find(|o| o.long == name), value)
        } else if arg.len() > 1 && arg.starts_with('-') {
            let mut letters = arg[1..].chars();
            let short = letters.next();
            (known().find(|o| o.short == short), Some(letters.as_str()).filter(|v| !v.is_empty()))
        } else {
            positionals.push(arg.clone());
            continue;
        };
        let Some(opt) = opt else {
            return usage(format!("Unknown option '{}'", arg.split('=').next().unwrap_or(arg)));
        };
        let value = match (opt.takes_value, inline) {
            (false, None) => None,
            (false, Some(_)) => {
                return usage(format!("Option '--{}' does not take an argument", opt.long));
            }
            (true, Some(value)) => Some(value.to_string()),
            (true, None) => {
                let Some(value) = rest.next_if(|next| !(next.len() > 1 && next.starts_with('-')))
                else {
                    return usage(format!("Option '--{} <value>' argument missing", opt.long));
                };
                Some(value.clone())
            }
        };
        values.0.entry(opt.long).or_default().extend(value);
    }
    if let Some(first) = positionals.first().filter(|_| !allow_positionals) {
        return usage(format!(
            "Unexpected argument '{first}'. This command does not take positional arguments"
        ));
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
    if gpu && values.flag("login") {
        return usage("choose --gpu or --login, not both");
    }
    for f in ["size", "time", "cpus", "mem"] {
        if values.text(f).is_some() && !gpu {
            return usage(format!(
                "--{f} sizes a --gpu session; a login-node session runs until you end it"
            ));
        }
    }
    let size = values.text("size").unwrap_or("small");
    let Some(preset) = SIZES.iter().find(|s| s.name == size) else {
        return usage(format!("--size \"{size}\" is not small, medium or large"));
    };
    let time = values.text("time");
    let Some(minutes) = parse_minutes(time.unwrap_or(preset.time)) else {
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
        None => Some(i64::from(SIZE_CPUS)),
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
    let mem_per_cpu = mem_text.unwrap_or(preset.mem_per_cpu).to_uppercase();
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
            gpus: preset.gpus,
            ask_size: !values.flag("login")
                && ["size", "time", "cpus", "mem"].iter().all(|f| values.text(f).is_none()),
            browser: !no_browser,
            view,
            another: values.flag("another"),
            yes: values.flag("yes"),
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
    flag("login"),
    string("size"),
    flag("another"),
    flag("yes"),
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
const UPGRADE: &[Opt] = &[string("version"), flag("keep-current")];
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
        "upgrade" => Ok(Command::Upgrade {
            version: values.text("version").map(str::to_string),
            keep_current: values.flag("keep-current"),
        }),
        "uninstall" => Ok(Command::Uninstall { purge: values.flag("purge") }),
        "list" => {
            let what = args.first().map(String::as_str);
            let Some(what) = what.filter(|w| ["jobs", "gpus", "sizes"].contains(w)) else {
                return usage(match what {
                    Some(w) if !w.is_empty() => {
                        format!("cannot list \"{w}\"; try aicentre list jobs, gpus or sizes")
                    }
                    _ => "aicentre list needs something to list: jobs, gpus or sizes".into(),
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

const GONE: &[(&str, &str)] = &[
    (
        "chat",
        "chat is now aicentre mcp, added to the agent tool you already use; see aicentre help mcp",
    ),
    ("profiles", "the sizes of a --gpu session are listed by aicentre list sizes"),
    ("sizes", "the sizes of a --gpu session are listed by aicentre list sizes"),
    ("gpu", "interactive GPU sessions are now aicentre shell, marimo, jupyter or code with --gpu"),
    ("submit", "submit is gone; in aicentre shell, submit with bsub on DTU or sbatch on Gefion"),
    ("logs", "logs is gone; in aicentre shell, read the job's output file"),
    (
        "cancel",
        "cancel is gone; Ctrl-C ends a session, and in aicentre shell bkill or scancel ends a job",
    ),
];

pub fn parse_command(argv: &[String]) -> Result<Command> {
    let Some(first) = argv.first() else { return Ok(Command::Help { topic: None }) };
    let rest = &argv[1..];
    if let Some((_, says)) = GONE.iter().find(|(name, _)| name == first) {
        return usage(*says);
    }
    match first.as_str() {
        "help" => return Ok(Command::Help { topic: rest.first().cloned() }),
        "-h" | "--help" => return Ok(Command::Help { topic: None }),
        "-V" | "--version" | "version" => return Ok(Command::Version),
        "self-install" => {
            let (values, _) = parse_args(rest, &[flag("keep-current")], false)?;
            if values.flag("help") {
                return Ok(Command::Help { topic: None });
            }
            return Ok(Command::SelfInstall { keep_current: values.flag("keep-current") });
        }
        "unpack-example" => {
            let [url, slug, parent] = rest else {
                return usage("aicentre unpack-example is run by aicentre itself, on a cluster");
            };
            return Ok(Command::UnpackExample {
                url: url.clone(),
                slug: slug.clone(),
                parent: parent.clone(),
            });
        }
        "launch" => {
            let mut words = rest;
            let (mut login, mut needs, mut approved) = (false, false, false);
            while let Some((first, tail)) = words.split_first() {
                match first.as_str() {
                    "--login" => login = true,
                    "--needs" => needs = true,
                    "--yes" => approved = true,
                    _ => break,
                }
                words = tail;
            }
            let [app, target, server, data, base, bind] = words else {
                return usage("aicentre launch is run by aicentre itself, on a cluster");
            };
            return Ok(Command::Launch {
                launch: LaunchRequest {
                    login,
                    needs,
                    approved,
                    app: app.clone(),
                    target: target.clone(),
                    server: server.clone(),
                    data: data.clone(),
                    base: base.clone(),
                    bind: bind.clone(),
                },
            });
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
            Command::Upgrade { version: Some("0.2.0".into()), keep_current: false }
        );
        assert_eq!(parse(&["mcp", "--help"]).unwrap(), Command::Help { topic: Some("mcp".into()) });
        assert_eq!(parse(&["self-install", "--help"]).unwrap(), Command::Help { topic: None });
        let dtu = parse(&["list", "jobs", "-p", "dtu"]).unwrap();
        for same in [&["list", "jobs", "-pdtu"][..], &["list", "--provider=dtu", "--", "jobs"]] {
            assert_eq!(parse(same).unwrap(), dtu, "{same:?}");
        }
        for (argv, says) in [
            (&["upgrade", "--version"][..], "'--version <value>' argument missing"),
            (&["list", "jobs", "-p", "--json"], "'--provider <value>' argument missing"),
            (&["check", "--json=1"], "'--json' does not take an argument"),
            (&["check", "-x"], "Unknown option '-x'"),
            (&["upgrade", "now"], "Unexpected argument 'now'"),
        ] {
            assert!(parse(argv).unwrap_err().0.contains(says), "{argv:?}");
        }
    }

    #[test]
    fn a_size_names_what_a_gpu_session_asks_for_and_single_options_change_one_part() {
        let asked = |argv: &[&str]| match parse(argv).unwrap() {
            Command::Session { session: s } => (s.gpus, s.cpus, s.mem_per_cpu, s.minutes),
            other => panic!("{other:?}"),
        };
        assert_eq!(asked(&["marimo", "--gpu"]), (1, 4, "8GB".into(), 120));
        assert_eq!(asked(&["marimo", "--gpu", "--size", "small"]), (1, 4, "8GB".into(), 120));
        assert_eq!(asked(&["marimo", "--gpu", "--size", "medium"]), (1, 4, "24GB".into(), 240));
        assert_eq!(asked(&["shell", "--gpu", "--size", "large"]), (2, 4, "50GB".into(), 240));
        assert_eq!(
            asked(&["code", "--gpu", "--size", "large", "--time", "30", "--mem", "16gb"]),
            (2, 4, "16GB".into(), 30)
        );
        for size in SIZES {
            let total = gigabytes(size.mem_per_cpu).unwrap() * 4.0;
            assert!(total <= MAX_GB && parse_minutes(size.time).unwrap() <= MAX_MINUTES);
        }
        let asks = |argv: &[&str]| match parse(argv).unwrap() {
            Command::Session { session: s } => s.ask_size,
            other => panic!("{other:?}"),
        };
        assert!(asks(&["marimo", "--gpu"]) && asks(&["marimo"]) && !asks(&["marimo", "--login"]));
        assert!(parse(&["marimo", "--gpu", "--login"]).is_err());
        assert!(!asks(&["marimo", "--gpu", "--size", "small"]));
        assert!(!asks(&["marimo", "--gpu", "--time", "30"]));
        for argv in [&["marimo", "--size", "large"][..], &["marimo", "--gpu", "--size", "huge"]] {
            assert!(parse(argv).is_err(), "{argv:?}");
        }
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
