use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use tokio::task::JoinHandle;
use unicode_segmentation::UnicodeSegmentation;

use super::width::string_width;
use crate::context::Output;

static ESCAPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:\x1b\[[0-9;?]*[A-Za-z]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\))").unwrap()
});

pub fn truncate(line: &str, max: usize) -> String {
    if string_width(line) <= max {
        return line.to_string();
    }
    let mut out = String::new();
    let mut width = 0;
    let mut i = 0;
    while i < line.len() {
        if let Some(m) = ESCAPE.find(&line[i..]) {
            out.push_str(m.as_str());
            i += m.end();
            continue;
        }
        let rest = &line[i..];
        let next =
            rest.char_indices().skip(1).find(|(_, c)| *c == '\x1b').map_or(rest.len(), |(n, _)| n);
        let text = &rest[..next];
        for g in text.graphemes(true) {
            let w = string_width(g);
            if width + w > max.saturating_sub(1) {
                return format!("{out}…\x1b[0m");
            }
            out.push_str(g);
            width += w;
        }
        i += text.len();
    }
    out
}

pub struct LiveArea {
    out: Output,
    height: usize,
    frame: usize,
}

impl LiveArea {
    pub fn new(out: Output) -> LiveArea {
        LiveArea { out, height: 0, frame: 0 }
    }

    pub fn start(&mut self) {
        self.out.write("\x1b[?25l");
    }

    pub fn next_frame(&mut self) -> usize {
        let f = self.frame;
        self.frame += 1;
        f
    }

    fn clear(&self) -> String {
        if self.height > 0 { format!("\r\x1b[{}A\x1b[J", self.height) } else { "\r\x1b[J".into() }
    }

    pub fn draw(&mut self, lines: Vec<String>) {
        let cols = self.out.columns().unwrap_or(80).max(20);
        let rows = self.out.rows().unwrap_or(24).max(5);
        let mut lines: Vec<String> = lines.iter().map(|l| truncate(l, cols - 1)).collect();
        if lines.len() > rows - 1 {
            lines = lines.split_off(lines.len() - (rows - 1));
        }
        let body: String = lines.iter().map(|l| format!("{l}\n")).collect();
        self.out.write(&format!("\x1b[?2026h{}{body}\x1b[?2026l", self.clear()));
        self.height = lines.len();
    }

    pub fn print(&mut self, lines: &[String]) {
        let body: String = lines.iter().map(|l| format!("{l}\n")).collect();
        self.out.write(&format!("\x1b[?2026h{}{body}\x1b[?2026l", self.clear()));
        self.height = 0;
    }

    pub fn stop(&mut self) {
        self.out.write(&format!("{}\x1b[?25h", self.clear()));
        self.height = 0;
    }
}

pub struct Ticker(Option<JoinHandle<()>>);

impl Ticker {
    pub fn every(interval: Duration, mut f: impl FnMut() + Send + 'static) -> Ticker {
        Ticker(Some(tokio::spawn(async move {
            let mut iv = tokio::time::interval(interval);
            iv.tick().await;
            loop {
                iv.tick().await;
                f();
            }
        })))
    }

    pub fn stop(&mut self) {
        if let Some(t) = self.0.take() {
            t.abort();
        }
    }
}

impl Drop for Ticker {
    fn drop(&mut self) {
        self.stop();
    }
}
