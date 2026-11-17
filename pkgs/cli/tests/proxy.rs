mod common;

use common::{Sandbox, dtu_host};
use serde_json::json;

fn sandbox() -> Sandbox {
    let sb = Sandbox::new();
    sb.file(".ssh/config", "");
    dtu_host(&sb);
    sb
}

#[tokio::test]
async fn aicentre_proxy_reaches_the_session_the_editor_was_given_through_the_shared_login() {
    let sb = sandbox();
    let notes = format!("{}/.local/state/aicentre/gpu", sb.home);
    std::fs::create_dir_all(&notes).unwrap();
    let note = |jobid: &str, node: &str, started_at: &str| {
        let first = jobid == "122";
        let doc = json!({
            "jobid": jobid,
            "app": "code",
            "pid": std::process::id(),
            "startedAt": started_at,
            "alias": "dtu",
            "node": node,
            "port": 40123,
            "hostKey": "ssh-ed25519 AAAAkey aicentre-gpu",
            "secret": (if first { "a" } else { "b" }).repeat(64),
            "host": if first { "aicentre-gpu-122" } else { "aicentre-gpu" },
        });
        std::fs::write(format!("{notes}/{jobid}.json"), doc.to_string()).unwrap();
    };
    note("122", "n-62-30-2", "2026-11-13T10:00:00Z");
    note("123", "n-62-30-1", "2026-11-13T11:00:00Z");
    sb.run(&["proxy", "aicentre-gpu"], &sb.ctx()).await;
    sb.run(&["proxy", "aicentre-gpu-122"], &sb.ctx()).await;
    let calls = sb.read("calls");
    let calls: Vec<&str> = calls.split('\n').filter(|l| l.contains(" -W ")).collect();
    let first: Vec<&str> = calls[0].split(' ').collect();
    assert_eq!(first.len(), 7, "{}", calls[0]);
    assert_eq!(first[0], "-S");
    assert_eq!(first[2..], ["-o", "ControlMaster=no", "-W", "n-62-30-1:40123", "dtu"]);
    assert!(calls[1].contains("-W n-62-30-2:40123 dtu"));
    assert_eq!(sb.read("proxy-stdin"), "a".repeat(64));
    assert_eq!(sb.run(&["proxy", "aicentre-gpu-999"], &sb.ctx()).await, 1);
    assert!(sb.err.text().contains("no aicentre code --gpu session for job 999"));
    assert_eq!(sb.out.text(), "");
}
