use crate::check::types::Status;
use crate::context::{Env, Output};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Accent {
    TrueColor,
    Ansi256,
    Red,
}

#[derive(Clone, Debug)]
pub struct Style {
    pub color: bool,
    pub art: bool,
    ascii: bool,
    accent: Accent,
}

pub fn color_enabled(env: &Env, out: &Output) -> bool {
    let get = |k: &str| env.get(k).map(String::as_str).unwrap_or("");
    if !get("NO_COLOR").is_empty() {
        return false;
    }
    if !get("FORCE_COLOR").is_empty() && get("FORCE_COLOR") != "0" {
        return true;
    }
    if get("TERM") == "dumb" {
        return false;
    }
    out.is_tty
}

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn make_style(env: &Env, out: &Output) -> Style {
    let get = |k: &str| env.get(k).map(String::as_str).unwrap_or("");
    let ascii = get("TERM") == "dumb" || get("AICENTRE_ASCII") == "1";
    let colorterm = get("COLORTERM").to_ascii_lowercase();
    let accent = if colorterm == "truecolor" || colorterm == "24bit" {
        Accent::TrueColor
    } else if get("TERM").contains("256") {
        Accent::Ansi256
    } else {
        Accent::Red
    };
    Style { color: color_enabled(env, out), art: out.is_tty && !ascii, ascii, accent }
}

impl Style {
    fn sgr(&self, open: u8, close: u8, s: &str) -> String {
        if self.color { format!("\x1b[{open}m{s}\x1b[{close}m") } else { s.to_string() }
    }

    pub fn green(&self, s: &str) -> String {
        self.sgr(32, 39, s)
    }

    pub fn red(&self, s: &str) -> String {
        self.sgr(31, 39, s)
    }

    pub fn yellow(&self, s: &str) -> String {
        self.sgr(33, 39, s)
    }

    pub fn cyan(&self, s: &str) -> String {
        self.sgr(36, 39, s)
    }

    pub fn dim(&self, s: &str) -> String {
        self.sgr(2, 22, s)
    }

    pub fn bold(&self, s: &str) -> String {
        self.sgr(1, 22, s)
    }

    pub fn accent(&self, s: &str) -> String {
        if !self.color {
            return s.to_string();
        }
        match self.accent {
            Accent::TrueColor => format!("\x1b[38;2;253;80;97m{s}\x1b[39m"),
            Accent::Ansi256 => format!("\x1b[38;5;203m{s}\x1b[39m"),
            Accent::Red => self.red(s),
        }
    }

    pub fn symbol(&self, status: Status) -> String {
        let (unicode, ascii) = match status {
            Status::Ok => ("✓", "+"),
            Status::Warn => ("!", "!"),
            Status::Fail => ("✗", "x"),
            Status::Skip => ("○", "-"),
            Status::Info => ("·", "*"),
        };
        let s = if self.ascii { ascii } else { unicode };
        match status {
            Status::Ok => self.green(s),
            Status::Warn => self.yellow(s),
            Status::Fail => self.red(s),
            Status::Skip => self.dim(s),
            Status::Info => self.cyan(s),
        }
    }

    pub fn spinner(&self, frame: usize) -> String {
        if self.ascii {
            let frames = ["-", "\\", "|", "/"];
            self.cyan(frames[frame % 4])
        } else {
            self.cyan(SPINNER[frame % SPINNER.len()])
        }
    }
}
