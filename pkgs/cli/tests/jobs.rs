mod common;

use common::{Sandbox, dtu_host, gefion_ready, in_hours};
use regex::Regex;
use serde_json::{Value, json};

const FAKE_RSYNC: &str = "#!/bin/sh\n[ \"$1\" = --version ] && { echo \"rsync  version 3.4.1  protocol version 32\"; exit 0; }\nprintf \"%s\\n\" \"$@\" > \"$FAKE/rsync\"\n";

fn base(sb: &Sandbox) {
    sb.file(".ssh/config", "");
    dtu_host(sb);
}

fn sandbox() -> Sandbox {
    let sb = Sandbox::new();
    base(&sb);
    sb
}

fn jobs(sb: &Sandbox) {
    gefion_ready(sb, in_hours(5));
    sb.answer(
        "bjobs",
        "124|PEND|p1|sweep|-|0 second(s)|-|Not enough GPUs\n123|RUN|p1i|aicentre-marimo|4*n-62-30-1|300 second(s)|0:55 L|-\n",
    );
    sb.answer("squeue", "98765|RUNNING|p1|train|dgx-01|2:03:04|21:56:56|None\n");
}

fn site(sb: &Sandbox, examples: &[Value]) -> String {
    let dir = format!("{}/site", sb.root);
    std::fs::create_dir_all(format!("{dir}/gallery")).unwrap();
    std::fs::write(
        format!("{dir}/gallery/index.json"),
        json!({ "examples": examples }).to_string(),
    )
    .unwrap();
    format!("file://{dir}")
}

fn rsync(sb: &Sandbox) {
    sb.file_mode(&format!("{}/bin/rsync", sb.fake), FAKE_RSYNC, 0o755);
}

#[tokio::test]
async fn list_jobs_puts_every_provider_in_one_table_running_first_wherever_it_runs() {
    let sb = sandbox();
    jobs(&sb);
    assert_eq!(sb.run(&["list", "jobs"], &sb.ctx()).await, 0);
    assert_eq!(sb.err.text(), "");
    let out = sb.out.text();
    assert!(out.contains("Your jobs on P1 compute  3 jobs"), "{out}");
    let order: Vec<usize> =
        ["123", "98765", "124"].iter().map(|id| out.find(&format!("  {id}  ")).unwrap()).collect();
    let mut sorted = order.clone();
    sorted.sort();
    assert_eq!(order, sorted);
    let pending =
        Regex::new(r"DTU\s+124\s+pending\s+p1\s+sweep\s+-\s+-\s+-\s+Not enough GPUs").unwrap();
    assert!(pending.is_match(&out), "{out}");
    let running =
        Regex::new(r"Gefion\s+98765\s+running\s+p1\s+train\s+dgx-01\s+2:03:04\s+21:56:56").unwrap();
    assert!(running.is_match(&out), "{out}");
}

#[tokio::test]
async fn list_gpus_shows_each_gpu_on_dtu_and_one_line_per_queue_and_partition() {
    let sb = sandbox();
    gefion_ready(&sb, in_hours(5));
    sb.answer(
        "bhosts",
        "@@QUEUE p1i\np1i|Open:Active|0|8\nHOST_NAME GPU_ID MODEL MUSED MRSV NJOBS RUN SUSP RSV\nn-62-12-83 0 NVIDIAH100PCIe 2.8G 0M 7 7 0 0\n 1 NVIDIAH100PCIe 466M 0M 8 8 0 0\n@@QUEUE p1\np1|Open:Active|24|92\nn-62-11-44 0 NVIDIAH100PCIe 74G 0M 1 1 0 0\n 1 NVIDIAH100PCIe 0M 0M 0 0 0 0\n",
    );
    sb.answer("sinfo", "gpu*|3|idle|gpu:8\ngpu*|12|allocated|gpu:8\n");
    assert_eq!(sb.run(&["list", "gpus"], &sb.ctx()).await, 0);
    assert_eq!(sb.err.text(), "");
    let out = sb.out.text();
    assert!(out.contains("GPUs on P1 compute  asked DTU and Gefion"), "{out}");
    for part in [
        "\nDTU p1i  sessions · 8 running, 0 waiting\n",
        "  NODE        GPU  MODEL     MEMORY USED  JOBS\n",
        "  n-62-12-83    1  H100PCIe       466 MB     8\n",
        "  The GPUs are shared: a new session starts at once",
        "\nDTU p1  batch jobs · 1 of 2 GPUs free, 92 running, 24 waiting\n",
        "  n-62-11-44    1  H100PCIe         0 MB  free\n",
        "\nGefion gpu  3 of 15 nodes idle, 0 partly used, each with gpu:8\n",
    ] {
        assert!(out.contains(part), "{part:?} is not in:\n{out}");
    }

    let sb = sandbox();
    sb.answer("bhosts", "");
    assert_eq!(sb.run(&["list", "gpus", "--provider", "dtu", "--json"], &sb.ctx()).await, 1);
    let doc: Value = serde_json::from_str(&sb.out.text()).unwrap();
    assert_eq!(doc["gpus"], json!([]));
    assert_eq!(doc["providers"][0]["error"], "bhosts failed: it listed no GPUs");
}

#[tokio::test]
async fn list_jobs_after_a_password_prompt_logs_in_once_then_asks_again_through_that_login() {
    let sb = sandbox();
    sb.answer("needs-login", "");
    sb.answer("bjobs", "123|RUN|p1|train|1*n-62-30-1|60 second(s)|0:59 L|-\n");
    let ctx = sb.ctx_with(|c| c.interactive = true);
    assert_eq!(sb.run(&["list", "jobs"], &ctx).await, 0);
    let out = sb.out.text();
    assert!(out.contains("dtu asks for more than your key. Log in here"), "{out}");
    assert!(Regex::new(r"DTU\s+123\s+running").unwrap().is_match(&out), "{out}");
    sb.out.clear();
    assert_eq!(sb.run(&["list", "jobs"], &ctx).await, 0);
    assert!(!sb.out.text().contains("Log in here"));
}

#[tokio::test]
async fn gallery_get_has_the_login_node_fetch_the_example_then_says_what_to_run() {
    let sb = sandbox();
    let url = site(
        &sb,
        &[json!({
            "code": "INT-GPU",
            "slug": "int-gpu",
            "title": "Interactive session with a GPU",
            "summary": "",
            "providers": ["dtu"],
            "page": "https://hpc.aicentre.dk/gallery/int-gpu/",
            "files": "https://hpc.aicentre.dk/gallery/int-gpu.tar.gz",
            "commands": ["aicentre marimo examples/int-gpu/notebook.py --gpu"],
        })],
    );
    let ctx = sb.ctx_env(&[("AICENTRE_SITE", &url)]);
    assert_eq!(sb.run(&["gallery", "get", "INT-GPU"], &ctx).await, 0);
    assert!(sb.read("calls").contains(
        "mkdir -p /dtu/p1/s123456/examples && { curl -fsSL --proto =https,file https://hpc.aicentre.dk/gallery/int-gpu.tar.gz || wget -qO-"
    ));
    let out = sb.out.text();
    assert!(out.contains("INT-GPU is in /dtu/p1/s123456/examples/int-gpu on P1 DTU HPC"), "{out}");
    assert!(out.contains("$ aicentre marimo examples/int-gpu/notebook.py --gpu"), "{out}");
}

#[tokio::test]
async fn push_copies_into_the_project_directory_with_rsync_through_the_shared_login() {
    let sb = sandbox();
    rsync(&sb);
    assert_eq!(sb.run(&["push", "datasets/", "data"], &sb.ctx()).await, 0);
    let said = sb.read("rsync");
    let args: Vec<&str> = said.trim().split('\n').collect();
    assert_eq!(args[args.len() - 2..], ["datasets", "dtu:/dtu/p1/s123456/data/"]);
    let rsh = args[args.iter().position(|a| *a == "-e").unwrap() + 1];
    let want = Regex::new(r"^ssh -S \S+cm-[0-9a-f]{12} -o ControlMaster=no$").unwrap();
    assert!(want.is_match(rsh), "{rsh}");
    assert!(sb.read("mkdir").contains("mkdir -p /dtu/p1/s123456/data/"));
}

#[tokio::test]
async fn gallery_get_refuses_files_that_are_not_the_examples_own_archive_on_the_site() {
    for files in [
        "https://evil.example/gallery/int-gpu.tar.gz",
        "-o/tmp/x",
        "http://hpc.aicentre.dk/gallery/int-gpu.tar.gz",
    ] {
        let sb = sandbox();
        let example = json!({
            "code": "INT-GPU", "slug": "int-gpu", "title": "x\u{1b}[2J", "providers": ["dtu"],
            "page": "https://hpc.aicentre.dk/gallery/int-gpu/", "files": files, "commands": [],
        });
        let at = site(&sb, &[example]);
        let ctx = sb.ctx_env(&[("AICENTRE_SITE", &at)]);
        assert_eq!(sb.run(&["gallery", "get", "int-gpu"], &ctx).await, 1);
        assert!(sb.err.text().contains("that are not its own archive"), "{}", sb.err.text());
        assert!(!sb.read("calls").contains("curl"));
        assert_eq!(sb.run(&["gallery"], &ctx).await, 0);
        assert!(!sb.out.text().contains('\u{1b}'));
    }
}
