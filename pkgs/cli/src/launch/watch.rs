use std::time::Duration;

use tokio::process::Child;

use super::gate::Gate;
use super::{Out, Step, activity};
use crate::context::Env;

const RECORD_SECONDS: u64 = 300;

fn append_private(path: &str, line: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new().append(true).create(true).mode(0o600).open(path)?;
    file.write_all(line.as_bytes())
}

pub fn seconds(env: &Env, name: &str, default: u64) -> u64 {
    env.get(name).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn step(limit: u64) -> u64 {
    if limit < 60 { 1 } else { 30 }
}

async fn pause(server: &mut Option<&mut Child>, seconds: u64) -> Option<i32> {
    let wait = tokio::time::sleep(Duration::from_secs(seconds));
    match server {
        Some(child) => tokio::select! {
            status = child.wait() => Some(super::exit_code(status)),
            () = wait => None,
        },
        None => {
            wait.await;
            None
        }
    }
}

pub async fn login(out: Out, env: &Env, mut server: Option<&mut Child>) -> Step<Option<i32>> {
    let limit = seconds(env, "AICENTRE_LOGIN_SECONDS", 43_200);
    let mut ran = 0;
    loop {
        if let Some(code) = pause(&mut server, step(limit)).await {
            return Ok(Some(code));
        }
        ran += step(limit);
        out.tag("BEAT", &ran.to_string())?;
        if ran >= limit {
            out.tag(
                "ERROR",
                &format!(
                    "a login-node session runs for {} h at most, so it ended; start it again",
                    limit / 3600
                ),
            )?;
            return Ok(None);
        }
    }
}

pub async fn idle(
    out: Out,
    env: &Env,
    gate: &Gate,
    mut server: Option<&mut Child>,
    what: &str,
    record: &str,
) -> Step<Option<i32>> {
    let limit = seconds(env, "AICENTRE_IDLE_SECONDS", 600);
    let pid = server.as_ref().and_then(|child| child.id());
    let mut noted = std::time::Instant::now();
    let mut before = pid.map_or(0.0, |pid| activity::cpu_seconds(pid as i32));
    let span = if limit >= 60 { format!("{} min", limit / 60) } else { format!("{limit} s") };
    let mut quiet = 0;
    loop {
        out.tag("BEAT", &quiet.to_string())?;
        quiet = if gate.busy() > 0 { 0 } else { quiet + step(limit) };
        // A record of how much the session works, every 5 minutes. Nothing
        // acts on it yet.
        if let Some(pid) = pid
            && noted.elapsed() >= Duration::from_secs(RECORD_SECONDS)
        {
            let now = activity::cpu_seconds(pid as i32);
            let line = format!(
                "{} connected={} cpu_seconds={:.2}\n",
                jiff::Timestamp::now().strftime("%Y-%m-%dT%H:%M:%SZ"),
                gate.busy(),
                (now - before).max(0.0)
            );
            let _ = append_private(record, &line);
            (noted, before) = (std::time::Instant::now(), now);
        }
        if quiet >= limit {
            out.tag(
                "ERROR",
                &format!("no {what} connection for {span}, so the job ended to free the GPU"),
            )?;
            return Ok(None);
        }
        if let Some(code) = pause(&mut server, step(limit)).await {
            return Ok(Some(code));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_come_from_the_environment_and_short_ones_step_by_the_second() {
        let env: Env = [("AICENTRE_IDLE_SECONDS".to_string(), "3".to_string())].into();
        assert_eq!(seconds(&env, "AICENTRE_IDLE_SECONDS", 600), 3);
        assert_eq!(seconds(&env, "AICENTRE_LOGIN_SECONDS", 43_200), 43_200);
        assert_eq!((step(3), step(60), step(600)), (1, 30, 30));
    }
}
