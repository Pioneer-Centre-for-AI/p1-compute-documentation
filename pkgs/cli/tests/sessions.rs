mod common;

use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use aicentre::Ctx;
use aicentre::session::interactive::{InteractiveRequest, SessionDeps, interactive};
use aicentre::session::lsf::{App, Server, View};
use common::{Sandbox, dtu_host};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

static PORTS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn sandbox() -> Sandbox {
    let sb = Sandbox::new();
    sb.file(".ssh/config", "");
    dtu_host(&sb);
    sb
}

fn request() -> InteractiveRequest {
    InteractiveRequest {
        app: App::Marimo,
        path: String::new(),
        gpu: true,
        provider: None,
        server: Server::Vscode,
        minutes: 60,
        cpus: 4,
        mem_per_cpu: "8GB".into(),
        browser: true,
        view: View::Browser,
        another: false,
    }
}

fn code() -> InteractiveRequest {
    InteractiveRequest { app: App::Code, gpu: false, view: View::Auto, ..request() }
}

type Seen = Arc<Mutex<Vec<String>>>;

struct Deps {
    d: SessionDeps,
    opened: Seen,
    folders: Seen,
}

fn cancel_soon(token: &CancellationToken) {
    let token = token.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        token.cancel();
    });
}

fn deps() -> Deps {
    let cancel = CancellationToken::new();
    let opened: Seen = Arc::default();
    let folders: Seen = Arc::default();
    let next = Arc::new(AtomicU16::new(55555));
    let (url_seen, url_cancel) = (opened.clone(), cancel.clone());
    let (folder_seen, folder_cancel) = (folders.clone(), cancel.clone());
    let d = SessionDeps {
        cancel,
        free_port: Arc::new(move || {
            let port = next.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { Ok(port) })
        }),
        open_url: Arc::new(move |url| {
            url_seen.lock().unwrap().push(url);
            cancel_soon(&url_cancel);
            Box::pin(async {})
        }),
        open_folder: Arc::new(move |argv: Vec<String>| {
            let program = argv[0].rsplit('/').next().unwrap().to_string();
            let words = [vec![program], argv[1..].to_vec()].concat();
            folder_seen.lock().unwrap().push(words.join(" "));
            cancel_soon(&folder_cancel);
            Box::pin(async { None })
        }),
    };
    Deps { d, opened, folders }
}

fn seen(list: &Seen) -> Vec<String> {
    list.lock().unwrap().clone()
}

const SECRET: &str = "abababababababababababababababababababababababababababababababab";

fn gpu_browser() -> String {
    [
        "Job <123> is submitted to queue <p1i>.",
        "<<Starting on n-62-30-1>>",
        "@@PID 777",
        "@@HOSTKEY ssh-ed25519 AAAAkey aicentre-gpu",
        &format!("@@SECRET {SECRET}"),
        "@@GATE 41000",
        "@@DIR /home/s123456",
        "@@HOST n-62-30-1",
        "@@PORT 40123",
        "@@PATH /?access_token=abc",
        "",
    ]
    .join("\n")
}

#[tokio::test]
async fn a_gpu_session_reaches_its_server_through_the_gate_then_ctrl_c_ends_the_job() {
    let _ports = PORTS.lock().await;
    let sb = sandbox();
    sb.answer("bsub.out", &gpu_browser());
    sb.answer("bsub.hold", "");
    let Deps { d, opened, .. } = deps();
    assert_eq!(interactive(&sb.ctx(), &request(), &d).await.unwrap(), 130);
    assert_eq!(sb.err.text(), "");
    assert_eq!(seen(&opened), ["http://127.0.0.1:55556/?access_token=abc"]);
    let out = sb.out.text();
    assert!(out.contains("marimo is ready on n-62-30-1"), "{out}");
    assert!(out.contains("    in /home/s123456\n"), "{out}");
    assert!(out.contains("Ended job 123."), "{out}");
    assert!(
        sb.read("bsub").contains(
            "marimo /home/s123456 vscode /dtu/p1/s123456/.aicentre /dtu/p1/s123456 0.0.0.0"
        )
    );
    assert!(sb.read("calls").contains("-O forward -L 127.0.0.1:55555:n-62-30-1:41000 dtu"));
    assert!(sb.read("cancel").contains("-O cancel -L 127.0.0.1:55555:n-62-30-1:41000 dtu"));
    assert!(sb.read("bkill").contains("bkill 123"));
}

#[tokio::test]
async fn a_second_gpu_session_from_this_laptop_is_refused_unless_asked_for() {
    let sb = sandbox();
    let note = aicentre::session::state::SessionNote {
        jobid: "122".into(),
        app: "jupyter".into(),
        pid: std::process::id() as i32,
        ..Default::default()
    };
    aicentre::session::state::save_note(&sb.ctx().paths, &note).unwrap();
    let Deps { d, .. } = deps();
    assert_eq!(interactive(&sb.ctx(), &request(), &d).await.unwrap(), 1);
    let err = sb.err.text();
    assert!(err.contains("job 122 from this laptop already holds a GPU on p1i"), "{err}");
    assert!(err.contains("add --another"), "{err}");
    assert_eq!(sb.read("bsub"), "");
}

#[tokio::test]
async fn a_job_whose_end_cannot_be_confirmed_is_never_reported_as_ended() {
    let _ports = PORTS.lock().await;
    let sb = sandbox();
    sb.answer("bsub.out", &gpu_browser());
    sb.answer("bsub.hold", "");
    sb.answer(
        "bkill.out",
        "ssh: connect to host login9.hpc.dtu.dk port 22: Network is unreachable\n",
    );
    sb.answer("bkill.code", "255");
    let Deps { d, .. } = deps();
    assert_eq!(interactive(&sb.ctx(), &request(), &d).await.unwrap(), 130);
    let out = sb.out.text();
    assert!(!out.contains("Ended job 123."), "{out}");
    assert!(out.contains("Could not confirm that job 123 ended: ssh: connect to host"), "{out}");
    assert!(out.contains("bkill 123 ends it"), "{out}");
}

#[tokio::test]
async fn a_gpu_session_that_ends_without_ctrl_c_still_releases_its_job() {
    let _ports = PORTS.lock().await;
    let sb = sandbox();
    sb.answer("bsub.out", &gpu_browser());
    sb.answer("bkill.out", "Job <123>: Job has already finished\n");
    sb.answer("bkill.code", "255");
    let Deps { d, .. } = deps();
    assert_eq!(interactive(&sb.ctx(), &request(), &d).await.unwrap(), 0);
    assert!(sb.read("bkill").contains("bkill 123"));
    let out = sb.out.text();
    assert!(out.contains("Job 123 ended"), "{out}");
    assert!(!out.contains("Could not confirm"), "{out}");
}

#[tokio::test]
async fn a_lost_connection_is_named_as_the_reason_the_job_was_ended() {
    let _ports = PORTS.lock().await;
    let sb = sandbox();
    sb.answer("bsub.out", &gpu_browser());
    sb.answer("bsub.code", "255");
    let Deps { d, .. } = deps();
    assert_eq!(interactive(&sb.ctx(), &request(), &d).await.unwrap(), 0);
    assert!(sb.read("bkill").contains("bkill 123"));
    let out = sb.out.text();
    assert!(
        out.contains(
            "Job 123 ended: the connection to dtu was lost, so the job was ended to free the GPU."
        ),
        "{out}"
    );
}

#[tokio::test]
async fn a_path_or_host_from_the_cluster_that_could_lead_elsewhere_opens_nothing() {
    let _ports = PORTS.lock().await;
    for (from, to) in [
        ("@@PATH /?access_token=abc", "@@PATH @evil.example/?access_token=abc"),
        ("@@HOST n-62-30-1", "@@HOST n-62-30-1:22:evil.example"),
    ] {
        let sb = sandbox();
        sb.answer("bsub.out", &gpu_browser().replace(from, to));
        sb.answer("bsub.hold", "");
        let Deps { d, opened, .. } = deps();
        assert_eq!(interactive(&sb.ctx(), &request(), &d).await.unwrap(), 1);
        assert!(sb.out.text().contains("that cannot be used, so nothing was opened"));
        assert!(seen(&opened).is_empty());
        assert!(!sb.read("calls").contains("-O forward"));
        assert!(sb.read("bkill").contains("bkill 123"));
    }
}

#[tokio::test]
async fn escape_codes_in_what_the_cluster_says_never_reach_the_terminal() {
    let _ports = PORTS.lock().await;
    let sb = sandbox();
    sb.answer(
        "bsub.out",
        &gpu_browser()
            .replace("@@PID 777", "@@STATUS starting \x1b]52;c;ZXZpbA==\x07 marimo\n@@PID 777"),
    );
    sb.answer("bsub.hold", "");
    let Deps { d, .. } = deps();
    assert_eq!(interactive(&sb.ctx(), &request(), &d).await.unwrap(), 130);
    let out = sb.out.text();
    assert!(!out.contains("\x1b]52") && !out.contains('\x07'), "{out:?}");
}

#[tokio::test]
async fn a_gpu_session_that_sends_no_secret_is_ended_not_opened() {
    let sb = sandbox();
    sb.answer("bsub.out", &gpu_browser().replace(&format!("@@SECRET {SECRET}\n"), ""));
    sb.answer("bsub.hold", "");
    let Deps { d, opened, .. } = deps();
    assert_eq!(interactive(&sb.ctx(), &request(), &d).await.unwrap(), 1);
    assert!(sb.out.text().contains("the job sent no usable secret or gate port"));
    assert!(seen(&opened).is_empty());
    assert!(!sb.read("calls").contains("-O forward"));
    assert!(sb.read("bkill").contains("bkill 123"));
}

#[tokio::test]
async fn a_cpu_session_runs_on_the_login_node_and_forwards_to_its_127_0_0_1() {
    let sb = sandbox();
    sb.answer("login.out", "@@PID 4242\n@@HOST login9\n@@PORT 40999\n@@PATH /?tkn=xyz\n");
    sb.answer("login.hold", "");
    let Deps { d, opened, .. } = deps();
    let r = InteractiveRequest { app: App::Code, gpu: false, ..request() };
    let code = interactive(&sb.ctx(), &r, &d).await.unwrap();
    assert_eq!(sb.err.text(), "");
    assert_eq!(code, 130);
    assert_eq!(seen(&opened), ["http://127.0.0.1:55555/?tkn=xyz"]);
    assert!(sb.out.text().contains("VS Code is ready on dtu, the P1 DTU HPC login node"));
    assert!(sb.read("calls").contains("-O forward -L 127.0.0.1:55555:127.0.0.1:40999 dtu"));
    assert_eq!(sb.read("kill"), "kill 4242\n");
    assert_eq!(sb.read("bsub"), "");
}

#[tokio::test]
async fn jupyter_runs_jupyterlab_on_the_login_node_and_opens_it() {
    let sb = sandbox();
    sb.answer("login.out", "@@PID 4242\n@@HOST login9\n@@PORT 40999\n@@PATH /lab?token=abc\n");
    sb.answer("login.hold", "");
    let Deps { d, opened, .. } = deps();
    let r = InteractiveRequest { app: App::Jupyter, gpu: false, ..request() };
    assert_eq!(interactive(&sb.ctx(), &r, &d).await.unwrap(), 130);
    assert_eq!(sb.err.text(), "");
    assert_eq!(seen(&opened), ["http://127.0.0.1:55555/lab?token=abc"]);
    assert!(sb.out.text().contains("JupyterLab is ready on dtu"));
    assert!(sb.read("login").contains(" jupyter /home/s123456 vscode /dtu/p1/s123456/.aicentre "));
}

fn fake_editor(sb: &Sandbox, name: &str, extensions: &str) {
    sb.file_mode(
        &format!("{}/bin/{name}", sb.fake),
        &format!(
            "#!/bin/sh\n[ \"$1\" = --list-extensions ] && {{ printf '{extensions}'; exit 0; }}\nexit 1\n"
        ),
        0o755,
    );
}

fn fake_code(sb: &Sandbox, extensions: &str) -> Ctx {
    fake_editor(sb, "code", extensions);
    sb.ctx_env(&[("DISPLAY", ":0")])
}

#[tokio::test]
async fn code_opens_desktop_vs_code_over_remote_ssh_and_leaves_the_connection_to_it() {
    let sb = sandbox();
    let ctx = fake_code(&sb, "ms-python.python\nms-vscode-remote.remote-ssh\n");
    let Deps { d, opened, folders } = deps();
    let r = InteractiveRequest { path: "my project".into(), ..code() };
    assert_eq!(interactive(&ctx, &r, &d).await.unwrap(), 0);
    assert_eq!(sb.err.text(), "");
    assert_eq!(
        seen(&folders),
        ["code --folder-uri vscode-remote://ssh-remote+dtu/dtu/p1/s123456/my%20project"]
    );
    assert_eq!(sb.read("login"), "");
    assert!(seen(&opened).is_empty());
    let out = sb.out.text();
    assert!(out.contains("code opens /dtu/p1/s123456/my project on dtu"), "{out}");
    assert!(out.contains("Any ssh tool reaches it as dtu:/dtu/p1/s123456/my project."), "{out}");
}

fn gpu_bsub() -> String {
    [
        "Job <123> is submitted to queue <p1i>.",
        "<<Starting on n-62-30-1>>",
        "@@PID 777",
        "@@HOSTKEY ssh-ed25519 AAAAkey aicentre-gpu",
        &format!("@@SECRET {SECRET}"),
        "@@GATE 40123",
        "@@DIR /dtu/p1/s123456/proj",
        "@@HOST n-62-30-1",
        "@@PORT 40123",
        "@@PATH ",
        "",
    ]
    .join("\n")
}

fn mode(path: &str) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[tokio::test]
async fn code_gpu_runs_sshd_in_the_job_and_opens_desktop_vs_code_through_aicentre_gpu() {
    let sb = sandbox();
    let ctx = fake_code(&sb, "ms-vscode-remote.remote-ssh\n");
    sb.answer(
        "ssh-G-aicentre-gpu",
        "host aicentre-gpu\nproxycommand ~/.local/bin/aicentre proxy %n\n",
    );
    sb.answer("bsub.out", &gpu_bsub());
    sb.answer("bsub.hold", "");
    let Deps { mut d, opened, folders } = deps();
    let r = InteractiveRequest { gpu: true, view: View::Desktop, path: "proj".into(), ..code() };
    let state = format!("{}/.local/state/aicentre", sb.home);
    let during: Seen = Arc::default();
    let (inner, record, dir) = (d.open_folder.clone(), during.clone(), state.clone());
    d.open_folder = Arc::new(move |argv| {
        let note = format!("{dir}/gpu/123.json");
        let mut record = record.lock().unwrap();
        record.push(std::fs::read_to_string(&note).unwrap());
        record.push(std::fs::read_to_string(format!("{dir}/known_hosts")).unwrap());
        record.push(format!("{:o} {:o}", mode(&note), mode(&dir)));
        inner(argv)
    });
    assert_eq!(interactive(&ctx, &r, &d).await.unwrap(), 130);
    assert_eq!(sb.err.text(), "");
    let bsub = sb.read("bsub");
    assert!(bsub.contains(
        "sshd /dtu/p1/s123456/proj vscode /dtu/p1/s123456/.aicentre /dtu/p1/s123456 0.0.0.0"
    ));
    assert!(bsub.contains("-J aicentre-code"));
    assert_eq!(
        seen(&folders),
        ["code --folder-uri vscode-remote://ssh-remote+aicentre-gpu/dtu/p1/s123456/proj"]
    );
    assert!(seen(&opened).is_empty());
    let during = seen(&during);
    let [note, trusted, modes] = &during[..] else { panic!("{during:?}") };
    assert_eq!(modes, "600 700");
    let note: Value = serde_json::from_str(note).unwrap();
    for (key, want) in [
        ("jobid", Value::from("123")),
        ("alias", "dtu".into()),
        ("node", "n-62-30-1".into()),
        ("port", 40123.into()),
        ("hostKey", "ssh-ed25519 AAAAkey aicentre-gpu".into()),
        ("secret", SECRET.into()),
        ("host", "aicentre-gpu".into()),
    ] {
        assert_eq!(note[key], want, "{key}");
    }
    assert_eq!(trusted, "aicentre-gpu-123,aicentre-gpu ssh-ed25519 AAAAkey aicentre-gpu\n");
    assert_eq!(std::fs::read_to_string(format!("{state}/known_hosts")).unwrap(), "");
    assert_eq!(
        std::fs::read_to_string(format!("{state}/ssh_config")).unwrap(),
        [
            "# Written by aicentre, which keeps it up to date.",
            "Host aicentre-gpu aicentre-gpu-*",
            "  User s123456",
            "  ProxyCommand /nonexistent/aicentre proxy %n",
            "  UserKnownHostsFile ~/.local/state/aicentre/known_hosts",
            "  StrictHostKeyChecking yes",
            "  ServerAliveInterval 30",
            "",
        ]
        .join("\n")
    );
    let out = sb.out.text();
    assert!(out.contains("code is ready on n-62-30-1"), "{out}");
    assert!(out.contains("any ssh tool reaches it as aicentre-gpu:/dtu/p1/s123456/proj"), "{out}");
    assert!(sb.read("bkill").contains("bkill 123"));
    assert!(!sb.read("calls").contains("-O forward"));
}

#[tokio::test]
async fn code_gpu_opens_the_browser_and_says_the_desktop_editor_needs_asking_for() {
    let _ports = PORTS.lock().await;
    let sb = sandbox();
    let ctx = fake_code(&sb, "ms-vscode-remote.remote-ssh\n");
    sb.answer("bsub.out", &gpu_bsub().replace("@@PATH ", "@@PATH /?tkn=xyz"));
    sb.answer("bsub.hold", "");
    let Deps { d, opened, folders } = deps();
    let r = InteractiveRequest { gpu: true, ..code() };
    assert_eq!(interactive(&ctx, &r, &d).await.unwrap(), 130);
    assert!(sb.out.text().contains("needs --desktop: it runs an ssh server of your own"));
    assert!(sb.read("bsub").contains(" code /home/s123456 vscode "));
    assert_eq!(seen(&opened), ["http://127.0.0.1:55556/?tkn=xyz"]);
    assert!(seen(&folders).is_empty());
}
