use std::collections::BTreeMap;

use serde::Serialize;

use crate::check::clusters::ClusterId;
use crate::session::lsf::QUEUE;

pub const BATCH: &str = "p1";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Gpu {
    pub queue: String,
    pub node: String,
    pub gpu: String,
    pub model: String,
    pub used: String,
    pub jobs: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Queue {
    pub queue: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub status: String,
    pub shared: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpus: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free: Option<u32>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub nodes: BTreeMap<String, u32>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub gres: String,
}

pub fn bhosts_command() -> String {
    format!(
        r#"for q in {QUEUE} {BATCH}; do echo "@@QUEUE $q"; bqueues -noheader -o "queue_name status pend run delimiter='|'" "$q"; bhosts -gpu $(bqueues -l "$q" | sed -n 's/^HOSTS: *//p' | tr -d /); done"#
    )
}

pub fn sinfo_command() -> String {
    "sinfo -h -o '%P|%D|%T|%G'".into()
}

fn number(text: &str) -> u32 {
    text.trim().parse().unwrap_or(0)
}

pub fn parse_bhosts(text: &str) -> (Vec<Queue>, Vec<Gpu>) {
    let (mut queues, mut gpus): (Vec<Queue>, Vec<Gpu>) = (Vec::new(), Vec::new());
    let mut node = String::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("@@QUEUE ") {
            queues.push(Queue {
                queue: name.into(),
                shared: name == QUEUE,
                gpus: Some(0),
                free: Some(0),
                ..Default::default()
            });
            node.clear();
            continue;
        }
        let Some(queue) = queues.last_mut() else { continue };
        let bars: Vec<&str> = line.split('|').collect();
        if bars.len() == 4 && bars[0] == queue.queue {
            queue.status = bars[1].into();
            queue.waiting = Some(number(bars[2]));
            queue.running = Some(number(bars[3]));
            continue;
        }
        let mut words: Vec<&str> = line.split_whitespace().collect();
        if words.len() == 9 {
            node = words.remove(0).into();
        }
        if words.len() != 8 || node.is_empty() || words[0].parse::<u32>().is_err() {
            continue;
        }
        let jobs = number(words[4]);
        queue.gpus = queue.gpus.map(|n| n + 1);
        if jobs == 0 {
            queue.free = queue.free.map(|n| n + 1);
        }
        gpus.push(Gpu {
            queue: queue.queue.clone(),
            node: node.clone(),
            gpu: words[0].into(),
            model: words[1].strip_prefix("NVIDIA").unwrap_or(words[1]).into(),
            used: words[2].into(),
            jobs,
        });
    }
    (queues, gpus)
}

pub fn parse_sinfo(text: &str) -> Vec<Queue> {
    let mut queues: Vec<Queue> = Vec::new();
    for line in text.lines() {
        let parts: Vec<&str> = line.trim().split('|').collect();
        if parts.len() != 4 || parts[1].parse::<u32>().is_err() {
            continue;
        }
        let name = parts[0].trim_end_matches('*');
        let at = match queues.iter().position(|q| q.queue == name) {
            Some(at) => at,
            None => {
                queues.push(Queue { queue: name.into(), ..Default::default() });
                queues.len() - 1
            }
        };
        let queue = &mut queues[at];
        *queue.nodes.entry(parts[2].trim_end_matches(['*', '~', '#', '$']).into()).or_default() +=
            number(parts[1]);
        if queue.gres.is_empty() && parts[3] != "(null)" {
            queue.gres = parts[3].into();
        }
    }
    queues
}

fn count(n: u32, one: &str, many: &str) -> String {
    if n == 1 { format!("1 {one}") } else { format!("{n} {many}") }
}

pub fn memory(used: &str) -> String {
    match used.char_indices().last() {
        Some((at, unit)) if "KMGT".contains(unit) && at > 0 => format!("{} {unit}B", &used[..at]),
        _ => used.to_string(),
    }
}

pub fn headline(provider: ClusterId, q: &Queue) -> String {
    if provider != ClusterId::Dtu {
        let all: u32 = q.nodes.values().sum();
        let idle = q.nodes.get("idle").copied().unwrap_or(0);
        let mixed = q.nodes.get("mixed").copied().unwrap_or(0);
        let gres =
            if q.gres.is_empty() { String::new() } else { format!(", each with {}", q.gres) };
        return format!(
            "{idle} of {} idle, {mixed} partly used{gres}",
            count(all, "node", "nodes")
        );
    }
    let (running, waiting) = (q.running.unwrap_or(0), q.waiting.unwrap_or(0));
    if q.shared {
        return format!("sessions · {running} running, {waiting} waiting");
    }
    format!(
        "batch jobs · {} of {} free, {running} running, {waiting} waiting",
        q.free.unwrap_or(0),
        count(q.gpus.unwrap_or(0), "GPU", "GPUs")
    )
}

pub fn advice(provider: ClusterId, q: &Queue) -> Option<&'static str> {
    if provider != ClusterId::Dtu || !q.shared {
        return None;
    }
    Some(if q.waiting.unwrap_or(0) == 0 {
        "The GPUs are shared: a new session starts at once and shares their memory."
    } else {
        "The GPUs are shared, but sessions are waiting, so a new one may wait too."
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DTU: &str = "@@QUEUE p1i
p1i|Open:Active|0|8
HOST_NAME            GPU_ID           MODEL     MUSED      MRSV  NJOBS    RUN   SUSP    RSV
n-62-12-83                0  NVIDIAH100PCIe      2.8G        0M      7      7      0      0
                          1  NVIDIAH100PCIe      466M        0M      8      8      0      0
@@QUEUE p1
p1|Open:Active|24|92
HOST_NAME            GPU_ID           MODEL     MUSED      MRSV  NJOBS    RUN   SUSP    RSV
n-62-11-44                0  NVIDIAH100PCIe       74G        0M      1      1      0      0
                          1  NVIDIAH100PCIe        0M        0M      0      0      0      0
";

    #[test]
    fn what_dtu_prints_becomes_gpus_and_one_line_per_queue() {
        let (queues, gpus) = parse_bhosts(DTU);
        assert_eq!(gpus.len(), 4);
        assert_eq!(
            gpus[1],
            Gpu {
                queue: "p1i".into(),
                node: "n-62-12-83".into(),
                gpu: "1".into(),
                model: "H100PCIe".into(),
                used: "466M".into(),
                jobs: 8,
            }
        );
        assert_eq!((gpus[3].node.as_str(), gpus[3].jobs), ("n-62-11-44", 0));
        assert_eq!(headline(ClusterId::Dtu, &queues[0]), "sessions · 8 running, 0 waiting");
        assert!(advice(ClusterId::Dtu, &queues[0]).unwrap().contains("starts at once"));
        assert_eq!(
            headline(ClusterId::Dtu, &queues[1]),
            "batch jobs · 1 of 2 GPUs free, 92 running, 24 waiting"
        );
        assert_eq!(advice(ClusterId::Dtu, &queues[1]), None);
        assert_eq!(
            (memory("35.3G"), memory("435M"), memory("-")),
            ("35.3 GB".into(), "435 MB".into(), "-".into())
        );
    }

    #[test]
    fn sinfo_lines_add_up_per_partition() {
        let queues = parse_sinfo(
            "gpu*|3|idle|gpu:8\ngpu*|2|mixed|gpu:8\ngpu*|10|allocated|gpu:8\ncpu|1|down*|(null)\n",
        );
        assert_eq!(queues.len(), 2);
        assert_eq!(
            headline(ClusterId::Gefion, &queues[0]),
            "3 of 15 nodes idle, 2 partly used, each with gpu:8"
        );
        assert_eq!(queues[1].nodes.get("down"), Some(&1));
    }
}
