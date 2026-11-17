use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;

use super::live::{LiveArea, Ticker};
use super::style::Style;
use crate::check::types::{CheckResult, GroupId, GroupReport, Sink, Status};
use crate::context::Output;

struct GroupState {
    id: GroupId,
    name: String,
    checks: Vec<CheckResult>,
    running: Option<String>,
    report: Option<GroupReport>,
    printed: bool,
}

pub struct Summary {
    pub ok: bool,
    pub lines: Vec<String>,
}

pub trait Reporter: Sink {
    fn finish(&self, reports: &[GroupReport], summary: &Summary);
    fn abort(&self);
}

fn check_lines(c: &CheckResult, s: &Style) -> Vec<String> {
    let loud = matches!(c.status, Status::Fail | Status::Warn);
    let detail = match &c.detail {
        Some(d) if !d.is_empty() => format!("  {}", if loud { d.clone() } else { s.dim(d) }),
        _ => String::new(),
    };
    let mut lines = vec![format!("  {} {}{detail}", s.symbol(c.status), c.title)];
    if loud || c.status == Status::Info {
        for h in c.hint.iter().flatten() {
            lines.push(if h.starts_with("  ") {
                format!("    {}", s.cyan(h))
            } else if !h.is_empty() {
                format!("    {}", s.dim(h))
            } else {
                String::new()
            });
        }
    }
    lines
}

fn group_lines(g: &GroupState, s: &Style, frame: Option<usize>) -> Vec<String> {
    if let Some(r) = &g.report
        && r.status == Status::Skip
        && let Some(note) = r.note.as_deref().filter(|n| !n.is_empty())
    {
        return vec![
            format!("{} {}  {}", s.symbol(Status::Skip), s.bold(&g.name), s.dim(note)),
            String::new(),
        ];
    }
    let mut lines = vec![s.bold(&g.name)];
    for c in &g.checks {
        lines.extend(check_lines(c, s));
    }
    if let (Some(running), Some(frame)) = (&g.running, frame) {
        lines.push(format!("  {} {running}", s.spinner(frame)));
    }
    lines.push(String::new());
    lines
}

struct Ordered {
    style: Style,
    groups: Vec<GroupState>,
}

impl Ordered {
    fn find(&mut self, id: GroupId) -> &mut GroupState {
        self.groups.iter_mut().find(|g| g.id == id).expect("no such group")
    }

    fn group_start(&mut self, id: GroupId, name: &str) {
        self.groups.push(GroupState {
            id,
            name: name.to_string(),
            checks: vec![],
            running: None,
            report: None,
            printed: false,
        });
    }

    fn step_start(&mut self, group: GroupId, title: &str) {
        self.find(group).running = Some(title.to_string());
    }

    fn step_end(&mut self, group: GroupId, result: &CheckResult) {
        let g = self.find(group);
        g.running = None;
        g.checks.push(result.clone());
    }

    fn group_end(&mut self, report: &GroupReport) -> Vec<String> {
        self.find(report.id).report = Some(report.clone());
        let mut lines = Vec::new();
        for g in &mut self.groups {
            if g.printed {
                continue;
            }
            if g.report.is_none() {
                break;
            }
            g.printed = true;
            lines.extend(group_lines(g, &self.style, None));
        }
        lines
    }

    fn pending(&self, frame: usize) -> Vec<String> {
        self.groups
            .iter()
            .filter(|g| !g.printed)
            .flat_map(|g| group_lines(g, &self.style, Some(frame)))
            .collect()
    }
}

struct Live {
    ordered: Ordered,
    area: LiveArea,
}

impl Live {
    fn redraw(&mut self) {
        let frame = self.area.next_frame();
        let lines = self.ordered.pending(frame);
        self.area.draw(lines);
    }
}

pub struct LiveReporter {
    state: Arc<Mutex<Live>>,
    ticker: Mutex<Option<Ticker>>,
    out: Output,
}

impl LiveReporter {
    pub fn new(style: Style, out: &Output) -> LiveReporter {
        let mut area = LiveArea::new(out.clone());
        area.start();
        let state = Arc::new(Mutex::new(Live { ordered: Ordered { style, groups: vec![] }, area }));
        state.lock().unwrap().redraw();
        let shared = state.clone();
        let ticker = Ticker::every(Duration::from_millis(80), move || {
            shared.lock().unwrap_or_else(|e| e.into_inner()).redraw();
        });
        LiveReporter { state, ticker: Mutex::new(Some(ticker)), out: out.clone() }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Live> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn stop(&self) {
        if let Some(mut t) = self.ticker.lock().unwrap_or_else(|e| e.into_inner()).take() {
            t.stop();
        }
        self.lock().area.stop();
    }
}

impl Sink for LiveReporter {
    fn group_start(&self, id: GroupId, name: &str) {
        self.lock().ordered.group_start(id, name);
    }

    fn step_start(&self, group: GroupId, _id: &str, title: &str) {
        self.lock().ordered.step_start(group, title);
    }

    fn step_end(&self, group: GroupId, result: &CheckResult) {
        self.lock().ordered.step_end(group, result);
    }

    fn group_end(&self, report: &GroupReport) {
        let mut live = self.lock();
        let lines = live.ordered.group_end(report);
        if !lines.is_empty() {
            live.area.print(&lines);
            live.redraw();
        }
    }
}

impl Reporter for LiveReporter {
    fn finish(&self, _reports: &[GroupReport], summary: &Summary) {
        self.stop();
        self.out.write(&summary.lines.iter().map(|l| format!("{l}\n")).collect::<String>());
    }

    fn abort(&self) {
        self.stop();
    }
}

pub struct PlainReporter {
    ordered: Mutex<Ordered>,
    out: Output,
}

impl PlainReporter {
    pub fn new(style: Style, out: &Output) -> PlainReporter {
        PlainReporter { ordered: Mutex::new(Ordered { style, groups: vec![] }), out: out.clone() }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Ordered> {
        self.ordered.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn emit(&self, lines: &[String]) {
        self.out.write(&lines.iter().map(|l| format!("{l}\n")).collect::<String>());
    }
}

impl Sink for PlainReporter {
    fn group_start(&self, id: GroupId, name: &str) {
        self.lock().group_start(id, name);
    }

    fn step_start(&self, group: GroupId, _id: &str, title: &str) {
        self.lock().step_start(group, title);
    }

    fn step_end(&self, group: GroupId, result: &CheckResult) {
        self.lock().step_end(group, result);
    }

    fn group_end(&self, report: &GroupReport) {
        let lines = self.lock().group_end(report);
        if !lines.is_empty() {
            self.emit(&lines);
        }
    }
}

impl Reporter for PlainReporter {
    fn finish(&self, _reports: &[GroupReport], summary: &Summary) {
        self.emit(&summary.lines);
    }

    fn abort(&self) {}
}

pub struct JsonReporter {
    out: Output,
    version: String,
    connect: bool,
}

impl JsonReporter {
    pub fn new(out: &Output, version: &str, connect: bool) -> JsonReporter {
        JsonReporter { out: out.clone(), version: version.to_string(), connect }
    }
}

impl Sink for JsonReporter {
    fn group_start(&self, _id: GroupId, _name: &str) {}
    fn step_start(&self, _group: GroupId, _id: &str, _title: &str) {}
    fn step_end(&self, _group: GroupId, _result: &CheckResult) {}
    fn group_end(&self, _report: &GroupReport) {}
}

#[derive(Serialize)]
struct Doc<'a> {
    version: &'a str,
    connect: bool,
    ok: bool,
    groups: &'a [GroupReport],
}

impl Reporter for JsonReporter {
    fn finish(&self, reports: &[GroupReport], summary: &Summary) {
        let doc =
            Doc { version: &self.version, connect: self.connect, ok: summary.ok, groups: reports };
        self.out.write(&format!("{}\n", serde_json::to_string_pretty(&doc).unwrap_or_default()));
    }

    fn abort(&self) {}
}

pub fn summarize(reports: &[GroupReport], style: &Style, connect: bool) -> Summary {
    let count =
        |s: Status| reports.iter().flat_map(|r| &r.checks).filter(|c| c.status == s).count();
    let (ok_n, warn_n, fail_n) = (count(Status::Ok), count(Status::Warn), count(Status::Fail));
    let checked =
        reports.iter().filter(|r| r.id != GroupId::Local && r.status != Status::Skip).count();
    let ok = fail_n == 0;

    let mut parts = vec![style.green(&format!("{ok_n} passed"))];
    if warn_n > 0 {
        let word = if warn_n == 1 { "warning" } else { "warnings" };
        parts.push(style.yellow(&format!("{warn_n} {word}")));
    }
    if fail_n > 0 {
        parts.push(style.red(&format!("{fail_n} failed")));
    }
    let mut lines = vec![parts.join(&style.dim(" · "))];
    if checked == 0 {
        lines.push(style.dim(
            "No P1 provider is set up on this machine yet. aicentre check --all shows what each one needs.",
        ));
    } else if ok && !connect {
        lines.push(
            style.dim("Files look right. aicentre check --connect also logs in to each provider."),
        );
    }
    Summary { ok, lines }
}
