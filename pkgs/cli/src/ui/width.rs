use std::sync::LazyLock;

use regex::Regex;
use unicode_width::UnicodeWidthStr;

static ESCAPES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-9;?]*[A-Za-z]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)").unwrap()
});

pub fn strip_ansi(s: &str) -> String {
    ESCAPES.replace_all(s, "").into_owned()
}

pub fn string_width(s: &str) -> usize {
    if !s.contains('\x1b') {
        return s.width();
    }
    strip_ansi(s).width()
}
