use std::sync::LazyLock;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use regex::Regex;

use crate::context::Env;
use crate::exec::{RunOptions, run};
use crate::paths::join;

pub fn tsh_home(env: &Env, home: &str) -> String {
    match env.get("TELEPORT_HOME").filter(|v| !v.is_empty()) {
        Some(dir) => dir.clone(),
        None => join(&[home, ".tsh"]),
    }
}

pub fn has_profile(tsh: &str, proxy: &str) -> bool {
    std::path::Path::new(&join(&[tsh, &format!("{proxy}.yaml")])).exists()
}

pub fn profile_users(tsh: &str, proxy: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(join(&[tsh, "keys", proxy])) else { return vec![] };
    let mut users: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|name| name.strip_suffix("-ssh").map(str::to_string))
        .collect();
    users.sort();
    users
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TshVersion {
    Missing,
    Unknown,
    Known { major: u32, minor: u32, text: String },
}

static TSH_VERSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^v?([0-9]+)\.([0-9]+)").unwrap());

pub async fn tsh_version(env: &Env) -> TshVersion {
    let r = run(
        "tsh",
        &["version", "--client", "--format=json"],
        RunOptions::new(env).timeout_ms(5_000),
    )
    .await;
    if r.missing {
        return TshVersion::Missing;
    }
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&r.stdout) else {
        return TshVersion::Unknown;
    };
    let text = match doc.get("version") {
        None | Some(serde_json::Value::Null) => String::new(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    };
    match TSH_VERSION.captures(&text) {
        Some(m) => TshVersion::Known {
            major: m[1].parse().unwrap_or(u32::MAX),
            minor: m[2].parse().unwrap_or(u32::MAX),
            text: text.clone(),
        },
        None => TshVersion::Unknown,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Validity {
    pub from: Option<Timestamp>,
    pub to: Option<Timestamp>,
}

static VALID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Valid:\s+(?:from\s+(\S+)\s+to\s+(\S+)|(forever))").unwrap());

// ssh-keygen prints local time
fn date(s: Option<&str>) -> Option<Timestamp> {
    let s = s.filter(|s| *s != "forever")?;
    if let Ok(t) = s.parse::<Timestamp>() {
        return Some(t);
    }
    let local = s.parse::<jiff::civil::DateTime>().ok()?;
    Some(local.to_zoned_datetime(TimeZone::system()).ok()?.timestamp())
}

pub fn parse_validity(output: &str) -> Option<Validity> {
    let m = VALID.captures(output)?;
    if m.get(3).is_some() {
        return Some(Validity { from: None, to: None });
    }
    Some(Validity {
        from: date(m.get(1).map(|x| x.as_str())),
        to: date(m.get(2).map(|x| x.as_str())),
    })
}

pub async fn cert_validity(cert: &str, env: &Env) -> Option<Validity> {
    let r = run("ssh-keygen", &["-L", "-f", cert], RunOptions::new(env).timeout_ms(5_000)).await;
    if r.code != 0 {
        return None;
    }
    parse_validity(&r.stdout)
}

pub fn left_ms(v: &Validity) -> Option<i64> {
    v.to.map(|to| to.as_millisecond() - Timestamp::now().as_millisecond())
}

pub fn duration(ms: i64) -> String {
    let min = (ms.unsigned_abs() as f64 / 60_000.0).round() as u64;
    if min < 60 {
        return format!("{min} min");
    }
    let h = min / 60;
    if h < 48 {
        return if !min.is_multiple_of(60) {
            format!("{h} h {} min", min % 60)
        } else {
            format!("{h} h")
        };
    }
    format!("{} days", (h as f64 / 24.0).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validity_and_durations() {
        let v = parse_validity("        Valid: forever\n").unwrap();
        assert_eq!(v.to, None);
        let v = parse_validity("        Valid: from 2026-01-01T00:00:00 to 2026-01-02T00:00:00\n")
            .unwrap();
        let (from, to) = (v.from.unwrap(), v.to.unwrap());
        assert_eq!(to.as_second() - from.as_second(), 86_400);
        assert_eq!(parse_validity("no dates"), None);
        assert_eq!(duration(-3_600_000), "1 h");
        assert_eq!(duration(5 * 3_600_000 + 59_000), "5 h 1 min");
        assert_eq!(duration(29_000), "0 min");
        assert_eq!(duration(30_000), "1 min");
        assert_eq!(duration(72 * 3_600_000), "3 days");
    }
}
