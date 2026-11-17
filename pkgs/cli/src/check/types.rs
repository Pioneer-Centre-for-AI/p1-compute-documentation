use std::future::Future;
use std::sync::Arc;

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Warn,
    Fail,
    Skip,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CheckResult {
    pub id: String,
    pub title: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<Vec<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupId {
    Local,
    Dtu,
    Gefion,
    Ngc,
}

impl GroupId {
    pub fn as_str(self) -> &'static str {
        match self {
            GroupId::Local => "local",
            GroupId::Dtu => "dtu",
            GroupId::Gefion => "gefion",
            GroupId::Ngc => "ngc",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GroupReport {
    pub id: GroupId,
    pub name: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub checks: Vec<CheckResult>,
}

pub trait Sink: Send + Sync {
    fn group_start(&self, id: GroupId, name: &str);
    fn step_start(&self, group: GroupId, id: &str, title: &str);
    fn step_end(&self, group: GroupId, result: &CheckResult);
    fn group_end(&self, report: &GroupReport);
}

fn rank(s: Status) -> u8 {
    match s {
        Status::Skip => 0,
        Status::Info => 1,
        Status::Ok => 2,
        Status::Warn => 3,
        Status::Fail => 4,
    }
}

pub fn worst(statuses: impl IntoIterator<Item = Status>) -> Status {
    statuses.into_iter().fold(Status::Skip, |a, b| if rank(b) > rank(a) { b } else { a })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub status: Status,
    pub detail: Option<String>,
    pub hint: Option<Vec<String>>,
}

impl Outcome {
    pub fn new(status: Status) -> Outcome {
        Outcome { status, detail: None, hint: None }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Outcome {
        self.detail = Some(detail.into());
        self
    }

    pub fn hint(mut self, hint: Vec<String>) -> Outcome {
        self.hint = Some(hint);
        self
    }

    pub fn ok(detail: impl Into<String>) -> Outcome {
        Outcome::new(Status::Ok).detail(detail)
    }

    pub fn warn(detail: impl Into<String>) -> Outcome {
        Outcome::new(Status::Warn).detail(detail)
    }

    pub fn fail(detail: impl Into<String>) -> Outcome {
        Outcome::new(Status::Fail).detail(detail)
    }

    pub fn info(detail: impl Into<String>) -> Outcome {
        Outcome::new(Status::Info).detail(detail)
    }

    pub fn skip(detail: impl Into<String>) -> Outcome {
        Outcome::new(Status::Skip).detail(detail)
    }
}

pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub checks: Vec<CheckResult>,
    pub note: Option<String>,
    pub skipped: bool,
    sink: Arc<dyn Sink>,
}

impl Group {
    pub fn new(id: GroupId, name: &str, sink: Arc<dyn Sink>) -> Group {
        sink.group_start(id, name);
        Group { id, name: name.to_string(), checks: vec![], note: None, skipped: false, sink }
    }

    pub async fn step<F>(&mut self, id: &str, title: &str, check: F) -> CheckResult
    where
        F: Future<Output = anyhow::Result<Outcome>>,
    {
        let full_id = if self.id == GroupId::Local {
            id.to_string()
        } else {
            format!("{}.{id}", self.id.as_str())
        };
        self.sink.step_start(self.id, &full_id, title);
        let outcome =
            check.await.unwrap_or_else(|e| Outcome::fail(format!("the check itself failed: {e}")));
        let result = CheckResult {
            id: full_id,
            title: title.to_string(),
            status: outcome.status,
            detail: outcome.detail,
            hint: outcome.hint,
        };
        self.checks.push(result.clone());
        self.sink.step_end(self.id, &result);
        result
    }

    pub async fn skip(&mut self, id: &str, title: &str, detail: &str) -> CheckResult {
        let detail = detail.to_string();
        self.step(id, title, async move { Ok(Outcome::skip(detail)) }).await
    }

    pub fn end(self) -> GroupReport {
        let report = GroupReport {
            id: self.id,
            name: self.name,
            status: if self.skipped {
                Status::Skip
            } else {
                worst(self.checks.iter().map(|c| c.status))
            },
            note: self.note,
            checks: self.checks,
        };
        self.sink.group_end(&report);
        report
    }
}
