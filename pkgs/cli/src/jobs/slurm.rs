use std::sync::LazyLock;

use regex::Regex;

use super::types::{Job, JobState, clock};
use crate::exec::shq;

const FORMAT: &str = "%i|%T|%P|%j|%N|%M|%L|%r";

pub fn squeue_command() -> String {
    format!("squeue --noheader -u \"$USER\" --format={}", shq(FORMAT))
}

fn state(text: &str) -> JobState {
    match text {
        "RUNNING" | "COMPLETING" | "CONFIGURING" => JobState::Running,
        "PENDING" => JobState::Pending,
        "SUSPENDED" => JobState::Suspended,
        _ => JobState::Other,
    }
}

static DURATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:([0-9]+)-)?(?:([0-9]+):)?([0-9]+):([0-9]{2})$").unwrap());
static NODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([^,\[]+)(?:\[([0-9]+))?").unwrap());
static LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]+(?:_[0-9]+|_\[[^\]]*\])?\|").unwrap());

fn duration(text: &str) -> String {
    let Some(m) = DURATION.captures(text.trim()) else { return String::new() };
    let n = |i: usize| m.get(i).map_or(0, |x| x.as_str().parse::<u64>().unwrap_or(u64::MAX));
    let seconds = n(1)
        .saturating_mul(86_400)
        .saturating_add(n(2).saturating_mul(3_600))
        .saturating_add(n(3).saturating_mul(60))
        .saturating_add(n(4));
    if seconds > 0 { clock(seconds) } else { String::new() }
}

fn first_node(list: &str) -> String {
    let t = list.trim();
    if t.is_empty() || t == "(null)" {
        return String::new();
    }
    match NODE.captures(t) {
        Some(m) => format!("{}{}", &m[1], m.get(2).map_or("", |x| x.as_str())),
        None => t.to_string(),
    }
}

pub fn parse_squeue(text: &str) -> Vec<Job> {
    let mut jobs = Vec::new();
    for line in text.split('\n') {
        let line = line.trim();
        if !LINE.is_match(line) {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        let field = |i: usize| parts.get(i).copied().unwrap_or("");
        let kind = state(field(1));
        let reason = if kind == JobState::Pending {
            let all = parts.get(7..).map(|r| r.join("|")).unwrap_or_default();
            if all == "None" { String::new() } else { all.trim().to_string() }
        } else {
            String::new()
        };
        jobs.push(Job {
            id: field(0).into(),
            state: kind,
            raw_state: field(1).into(),
            queue: field(2).into(),
            name: field(3).into(),
            node: first_node(field(4)),
            reason,
            elapsed: duration(field(5)),
            left: duration(field(6)),
        });
    }
    jobs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squeue_output() {
        let jobs = parse_squeue(
            &[
                "98765|RUNNING|p1|train|dgx-[01-02]|1-02:03:04|22:56:56|None",
                "98767_[1-4]|PENDING|p1|sweep||0:00|UNLIMITED|(Priority)",
            ]
            .join("\n"),
        );
        assert_eq!(
            (&*jobs[0].id, jobs[0].state, &*jobs[0].node, &*jobs[0].left, &*jobs[0].reason),
            ("98765", JobState::Running, "dgx-01", "22:56:56", "")
        );
        assert_eq!(jobs[0].elapsed, "1-02:03:04");
        assert_eq!(
            (&*jobs[1].id, jobs[1].state, &*jobs[1].left, &*jobs[1].reason),
            ("98767_[1-4]", JobState::Pending, "", "(Priority)")
        );
        assert_eq!(
            squeue_command(),
            "squeue --noheader -u \"$USER\" --format='%i|%T|%P|%j|%N|%M|%L|%r'"
        );
    }
}
