use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    Running,
    Pending,
    Suspended,
    #[default]
    Other,
}

impl JobState {
    pub fn as_str(self) -> &'static str {
        match self {
            JobState::Running => "running",
            JobState::Pending => "pending",
            JobState::Suspended => "suspended",
            JobState::Other => "other",
        }
    }

    fn order(self) -> u8 {
        match self {
            JobState::Running => 0,
            JobState::Suspended => 1,
            JobState::Pending => 2,
            JobState::Other => 3,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub state: JobState,
    pub raw_state: String,
    pub queue: String,
    pub name: String,
    pub node: String,
    pub reason: String,
    pub elapsed: String,
    pub left: String,
}

impl Job {
    pub fn order(&self) -> (u8, u64, &str) {
        let digits = self.id.bytes().take_while(u8::is_ascii_digit).count();
        (self.state.order(), self.id[..digits].parse().unwrap_or(u64::MAX), &self.id)
    }
}

pub fn clock(seconds: u64) -> String {
    let d = seconds / 86_400;
    let h = (seconds % 86_400) / 3_600;
    let m = (seconds % 3_600) / 60;
    let s = seconds % 60;
    let mmss = format!("{m:02}:{s:02}");
    if d > 0 { format!("{d}-{h:02}:{mmss}") } else { format!("{h}:{mmss}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clocks() {
        assert_eq!(clock(3725), "1:02:05");
        assert_eq!(clock(254_100), "2-22:35:00");
        assert_eq!(clock(60), "0:01:00");
    }

    #[test]
    fn jobs_run_first_then_by_number_with_array_tasks_next_to_their_job() {
        let mut jobs: Vec<Job> = [
            ("98767_[1-4]", JobState::Pending),
            ("98765", JobState::Running),
            ("98767_2", JobState::Running),
            ("98767_1", JobState::Running),
            ("9", JobState::Pending),
            ("98770", JobState::Suspended),
        ]
        .into_iter()
        .map(|(id, state)| Job { id: id.into(), state, ..Job::default() })
        .collect();
        jobs.sort_by(|a, b| a.order().cmp(&b.order()));
        let ids: Vec<&str> = jobs.iter().map(|j| j.id.as_str()).collect();
        assert_eq!(ids, ["98765", "98767_1", "98767_2", "98770", "9", "98767_[1-4]"]);
    }
}
