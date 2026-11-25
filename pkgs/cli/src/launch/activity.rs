use std::collections::HashMap;

pub fn parse_stat(stat: &str) -> Option<(i32, u64)> {
    let rest = &stat[stat.rfind(')')? + 1..];
    let fields: Vec<&str> = rest.split_whitespace().collect();
    // After the name: state, ppid, ..., utime and stime as the 12th and 13th.
    let parent = fields.get(1)?.parse().ok()?;
    let user: u64 = fields.get(11)?.parse().ok()?;
    let system: u64 = fields.get(12)?.parse().ok()?;
    Some((parent, user + system))
}

fn processes() -> HashMap<i32, (i32, u64)> {
    let mut all = HashMap::new();
    for entry in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|n| n.parse::<i32>().ok()) else {
            continue;
        };
        let stat = std::fs::read_to_string(entry.path().join("stat")).unwrap_or_default();
        if let Some(found) = parse_stat(&stat) {
            all.insert(pid, found);
        }
    }
    all
}

pub fn ticks_below(all: &HashMap<i32, (i32, u64)>, root: i32) -> u64 {
    let mut total = 0;
    let mut next = vec![root];
    let mut seen = std::collections::HashSet::new();
    while let Some(pid) = next.pop() {
        if !seen.insert(pid) {
            continue;
        }
        total += all.get(&pid).map_or(0, |(_, ticks)| *ticks);
        next.extend(all.iter().filter(|(_, (parent, _))| *parent == pid).map(|(child, _)| *child));
    }
    total
}

pub fn cpu_seconds(server: i32) -> f64 {
    // SAFETY: sysconf reads a constant of the system.
    let per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as f64;
    ticks_below(&processes(), server) as f64 / per_second
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_with_spaces_and_brackets_does_not_shift_the_fields() {
        let stat = "4242 (python3 (a b)) S 100 4242 4242 0 -1 4194304 1 0 0 0 250 50 0 0 20 0";
        assert_eq!(parse_stat(stat), Some((100, 300)));
        assert_eq!(parse_stat("no brackets"), None);
    }

    #[test]
    fn a_session_is_its_server_and_everything_below_it() {
        let all: HashMap<i32, (i32, u64)> =
            [(10, (1, 5)), (11, (10, 7)), (12, (11, 100)), (20, (1, 1000)), (21, (20, 1))].into();
        assert_eq!(ticks_below(&all, 10), 112);
        assert_eq!(ticks_below(&all, 11), 107);
        assert_eq!(ticks_below(&all, 99), 0);
        if !std::path::Path::new("/proc/self/stat").exists() {
            return;
        }
        let before = cpu_seconds(std::process::id() as i32);
        let mut busy = std::process::Command::new("sh")
            .args(["-c", "i=0; while [ $i -lt 300000 ]; do i=$((i+1)); done; sleep 2"])
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let during = cpu_seconds(std::process::id() as i32);
        let _ = busy.kill();
        let _ = busy.wait();
        assert!(during - before > 0.05, "{before} then {during}");
    }
}
