mod common;

use common::{Sandbox, gefion_ready, ssh_g};
use serde_json::Value;

fn sandbox() -> Sandbox {
    let sb = Sandbox::new();
    sb.file(".ssh/config", "# managed by the test\n");
    sb
}

struct Doc(Value);

impl Doc {
    fn result(&self, id: &str) -> Value {
        self.0["groups"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|g| g["checks"].as_array().unwrap())
            .find(|c| c["id"] == id)
            .cloned()
            .unwrap_or_else(|| panic!("no check {id}"))
    }

    fn group(&self, id: &str) -> Value {
        self.0["groups"].as_array().unwrap().iter().find(|g| g["id"] == id).cloned().unwrap()
    }
}

async fn check(sb: &Sandbox, args: &[&str]) -> (i32, Doc) {
    let mut argv = vec!["check", "--json"];
    argv.extend_from_slice(args);
    sb.out.clear();
    let code = sb.run(&argv, &sb.ctx()).await;
    (code, Doc(serde_json::from_str(&sb.out.text()).unwrap()))
}

#[tokio::test]
async fn a_ready_dtu_setup_passes() {
    let sb = sandbox();
    let known = format!("{}/.ssh/known_hosts", sb.home);
    sb.answer(
        "ssh-G-dtu",
        &ssh_g(&[
            ("host", &["dtu"]),
            ("hostname", &["login9.hpc.dtu.dk"]),
            ("user", &["s123456"]),
            ("port", &["22"]),
            ("serveraliveinterval", &["60"]),
            ("identityfile", &["~/.ssh/id_rsa", "~/.ssh/id_ed25519"]),
            ("userknownhostsfile", &[&known]),
        ]),
    );
    sb.file(".ssh/id_ed25519", "key");
    sb.answer("known-login9.hpc.dtu.dk", "");
    let (code, doc) = check(&sb, &[]).await;
    assert_eq!(code, 0);
    assert_eq!(doc.group("dtu")["status"], "ok");
    assert_eq!(doc.result("dtu.key")["detail"], "~/.ssh/id_ed25519");
}

#[tokio::test]
async fn a_provider_named_without_a_host_block_fails_with_the_block_to_add() {
    let sb = sandbox();
    let (code, doc) = check(&sb, &["-p", "dtu"]).await;
    assert_eq!(code, 1);
    let ids: Vec<&str> =
        doc.0["groups"].as_array().unwrap().iter().map(|g| g["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["local", "dtu"]);
    let host = doc.result("dtu.host");
    assert_eq!(host["status"], "fail");
    let hint: Vec<&str> =
        host["hint"].as_array().unwrap().iter().map(|h| h.as_str().unwrap()).collect();
    assert!(hint.join("\n").contains("HostName login9.hpc.dtu.dk"));
}

#[tokio::test]
async fn an_expired_teleport_login_fails_with_the_command_to_fix_it() {
    let sb = sandbox();
    gefion_ready(&sb, common::in_hours(-1));
    let (code, doc) = check(&sb, &["-p", "gefion"]).await;
    assert_eq!(code, 1);
    let login = doc.result("gefion.login");
    assert_eq!(login["status"], "fail");
    assert_eq!(login["detail"], "expired 1 h ago");
    assert_eq!(login["hint"][0], "tsh login --proxy secureremote.dcai.dk --user jdoe");
}
