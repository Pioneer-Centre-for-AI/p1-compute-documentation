use std::sync::LazyLock;

use regex::Regex;

use super::types::{Job, JobState, clock};
use crate::exec::shq;

const FIELDS: &str = "jobid stat queue job_name exec_host run_time time_left pend_reason";

pub fn bjobs_command() -> String {
    format!("bjobs -noheader -o {}", shq(&format!("{FIELDS} delimiter='|'")))
}

fn state(stat: &str) -> JobState {
    match stat {
        "RUN" => JobState::Running,
        "PEND" => JobState::Pending,
        "PSUSP" | "USUSP" | "SSUSP" => JobState::Suspended,
        _ => JobState::Other,
    }
}

static SLOTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+\*").unwrap());
static RUN_TIME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([0-9]+) second").unwrap());
static CLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([0-9]+):([0-9]{2})(?::([0-9]{2}))?").unwrap());
static LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+\|").unwrap());
static NO_JOBS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)No unfinished job found|No matching job found|No job found").unwrap()
});

fn first_host(exec_host: &str) -> String {
    let first = exec_host.split(':').next().unwrap_or("");
    if first == "-" { String::new() } else { SLOTS.replace(first, "").into_owned() }
}

fn seconds(text: &str) -> u64 {
    text.parse().unwrap_or(u64::MAX)
}

fn hms(text: &str) -> Option<u64> {
    let m = CLOCK.captures(text)?;
    let (a, b) = (seconds(&m[1]), seconds(&m[2]));
    Some(match m.get(3) {
        Some(s) => a.saturating_mul(3_600).saturating_add(b * 60 + seconds(s.as_str())),
        None => a.saturating_mul(3_600).saturating_add(b * 60),
    })
}

fn run_time(text: &str) -> String {
    let text = text.trim();
    let seconds = match RUN_TIME.captures(text) {
        Some(m) => Some(seconds(&m[1])),
        None => hms(text).filter(|_| text.matches(':').count() == 2),
    };
    match seconds {
        Some(n) if n > 0 => clock(n),
        _ => String::new(),
    }
}

fn time_left(text: &str) -> String {
    hms(text.trim()).map(clock).unwrap_or_default()
}

pub fn parse_bjobs(text: &str) -> Vec<Job> {
    let mut jobs = Vec::new();
    for line in text.split('\n') {
        let line = line.trim();
        if !LINE.is_match(line) {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        let field = |i: usize| parts.get(i).copied().unwrap_or("");
        let stat = field(1);
        let reason = if stat == "PEND" {
            let all = parts.get(7..).map(|r| r.join("|")).unwrap_or_default();
            if all == "-" { String::new() } else { all.trim().to_string() }
        } else {
            String::new()
        };
        jobs.push(Job {
            id: field(0).into(),
            state: state(stat),
            raw_state: stat.into(),
            queue: field(2).into(),
            name: field(3).into(),
            node: first_host(field(4)),
            reason,
            elapsed: run_time(field(5)),
            left: time_left(field(6)),
        });
    }
    jobs
}

pub fn no_jobs(text: &str) -> bool {
    NO_JOBS.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bjobs_output() {
        let jobs = parse_bjobs(
            &[
                "123|RUN|p1|train|4*n-62-30-1:4*n-62-30-2|3725 second(s)|70:35 L|-",
                "124|PEND|p1|sweep[3]|-|0 second(s)|-|Not enough GPUs available: 2 hosts;",
                "125|RUN|p1i|aicentre-jupyter|n-62-12-83|00:00:20|00:04:40 L|-",
                "not a job line",
            ]
            .join("\n"),
        );
        assert_eq!((jobs[2].elapsed.as_str(), jobs[2].left.as_str()), ("0:00:20", "0:04:40"));
        assert_eq!(
            jobs[..2],
            [
                Job {
                    id: "123".into(),
                    state: JobState::Running,
                    raw_state: "RUN".into(),
                    queue: "p1".into(),
                    name: "train".into(),
                    node: "n-62-30-1".into(),
                    reason: "".into(),
                    elapsed: "1:02:05".into(),
                    left: "2-22:35:00".into(),
                },
                Job {
                    id: "124".into(),
                    state: JobState::Pending,
                    raw_state: "PEND".into(),
                    queue: "p1".into(),
                    name: "sweep[3]".into(),
                    node: "".into(),
                    reason: "Not enough GPUs available: 2 hosts;".into(),
                    elapsed: "".into(),
                    left: "".into(),
                },
            ]
        );
        assert_eq!(
            bjobs_command(),
            "bjobs -noheader -o 'jobid stat queue job_name exec_host run_time time_left pend_reason delimiter='\\''|'\\'''"
        );
        assert!(no_jobs("No unfinished job found\n"));
        assert!(!no_jobs("Permission denied"));
    }
}
