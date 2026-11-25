mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use aicentre::context::{Ctx, Env};
use aicentre::install::layout::{place_binary, read_manifest};
use aicentre::install::release::sha256;
use aicentre::install::self_install::{commit, self_install};
use aicentre::install::uninstall::uninstall;
use aicentre::install::upgrade::upgrade;
use aicentre::platform::{asset_name, current_platform};
use common::Sandbox;

const ASSET: &str = "aicentre-linux-x64.tar.gz";

fn tar(env: &Env, args: &[&str]) {
    let status = Command::new("tar").args(args).env_clear().envs(env).status().unwrap();
    assert!(status.success(), "tar {args:?}");
}

fn sums(dir: &str, asset: &str, version: &str) {
    fs::write(format!("{dir}/VERSION"), format!("{version}\n")).unwrap();
    let hex = sha256(&format!("{dir}/{asset}")).unwrap();
    let named = sha256(&format!("{dir}/VERSION")).unwrap();
    fs::write(format!("{dir}/SHA256SUMS"), format!("{hex}  {asset}\n{named}  VERSION\n")).unwrap();
    sign(dir, key());
}

struct Key {
    file: String,
    signers: String,
}

fn key() -> &'static Key {
    static KEY: std::sync::OnceLock<Key> = std::sync::OnceLock::new();
    KEY.get_or_init(|| {
        let dir = tempfile::Builder::new().prefix("aicentre-key-").tempdir().unwrap().keep();
        let file = format!("{}/key", dir.to_string_lossy());
        let made = Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-C", "test", "-f", &file])
            .status()
            .unwrap();
        assert!(made.success(), "ssh-keygen made no key");
        let public = fs::read_to_string(format!("{file}.pub")).unwrap();
        let words: Vec<&str> = public.split_whitespace().collect();
        let signers = format!(
            "release@hpc.aicentre.dk namespaces=\"aicentre-release\" {} {}",
            words[0], words[1]
        );
        Key { file, signers }
    })
}

fn sign(dir: &str, key: &Key) {
    let _ = fs::remove_file(format!("{dir}/SHA256SUMS.sig"));
    // Without the agent: ssh-keygen asks it first, and one that does not
    // answer would hold the test up.
    let out = Command::new("ssh-keygen")
        .args(["-Y", "sign", "-f", &key.file, "-n", "aicentre-release"])
        .arg(format!("{dir}/SHA256SUMS"))
        .env_remove("SSH_AUTH_SOCK")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}

fn release(sb: &Sandbox, version: &str, corrupt: bool) -> String {
    let dir = format!("{}/rel-{version}", sb.root);
    let stage = format!("{dir}/stage");
    fs::create_dir_all(&stage).unwrap();
    let binary = format!("{stage}/aicentre");
    fs::write(&binary, format!("#!/bin/sh\necho {version}\n")).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    tar(&sb.env, &["-czf", &format!("{dir}/{ASSET}"), "-C", &stage, "aicentre"]);
    sums(&dir, ASSET, version);
    if corrupt {
        fs::write(format!("{dir}/{ASSET}"), "not a tarball").unwrap();
    }
    dir
}

fn installed(sb: &Sandbox) -> Ctx {
    let ctx = sb.ctx();
    let fake = format!("{}/fake-0.1.0", sb.root);
    fs::write(&fake, "#!/bin/sh\necho 0.1.0\n").unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let placed = place_binary(&ctx.paths, "0.1.0", &fake).unwrap();
    commit(&ctx, "0.1.0", &placed.created).unwrap();
    ctx
}

fn tree(sb: &Sandbox) -> Vec<String> {
    fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let r = if rel.is_empty() { name } else { format!("{rel}/{name}") };
            out.push(r.clone());
            if entry.file_type().unwrap().is_dir() {
                walk(&entry.path(), &r, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(Path::new(&sb.home), "", &mut out);
    out.sort();
    out
}

fn from(sb: &Sandbox, base: &str) -> Ctx {
    sb.ctx_env(&[("AICENTRE_DOWNLOAD_BASE", base), ("AICENTRE_RELEASE_SIGNERS", &key().signers)])
}

fn versions(ctx: &Ctx) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(&ctx.paths.versions)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn upgrade_switches_to_the_new_version_and_removes_the_old_one_and_the_download() {
    let sb = Sandbox::new();
    let ctx = installed(&sb);
    let code = upgrade(&from(&sb, &release(&sb, "0.2.0", false)), None, false).await.unwrap();
    assert_eq!(sb.err.text(), "");
    assert_eq!(code, 0);
    assert_eq!(fs::read_link(&ctx.paths.current).unwrap().to_string_lossy(), "versions/0.2.0");
    assert_eq!(versions(&ctx), ["0.2.0"]);
    assert_eq!(read_manifest(&ctx.paths).unwrap().version, "0.2.0");
    assert!(!Path::new(&ctx.paths.cache).exists());
    assert_eq!(
        sb.out.text(),
        "Downloading aicentre 0.2.0 for linux-x64...\n+ Upgraded aicentre 0.1.0 → 0.2.0\n"
    );
}

#[tokio::test]
async fn an_old_version_that_cannot_be_removed_stays_and_the_upgrade_still_holds() {
    use std::os::unix::fs::PermissionsExt;
    let sb = Sandbox::new();
    let ctx = installed(&sb);
    // As on a network home, where the folder of the running program is busy.
    let old = format!("{}/0.1.0", ctx.paths.versions);
    fs::set_permissions(&old, fs::Permissions::from_mode(0o555)).unwrap();
    let code = upgrade(&from(&sb, &release(&sb, "0.2.0", false)), None, false).await.unwrap();
    fs::set_permissions(&old, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!((code, sb.err.text().as_str()), (0, ""));
    assert_eq!(fs::read_link(&ctx.paths.current).unwrap().to_string_lossy(), "versions/0.2.0");
    assert!(Path::new(&ctx.paths.current).join("aicentre").exists());
    assert_eq!(versions(&ctx), ["0.1.0", "0.2.0"]);
}

#[tokio::test]
async fn an_upgrade_without_the_release_keys_signature_changes_nothing() {
    let sb = Sandbox::new();
    let ctx = installed(&sb);
    let before = tree(&sb);
    let dir = release(&sb, "0.2.0", false);

    let ours = sb.ctx_env(&[("AICENTRE_DOWNLOAD_BASE", &dir)]);
    assert_eq!(upgrade(&ours, None, false).await.unwrap(), 1);
    let err = sb.err.text();
    assert!(err.contains("it is signed with a key that is not aicentre's release key"), "{err}");

    let sums = format!("{dir}/SHA256SUMS");
    fs::write(&sums, fs::read_to_string(&sums).unwrap().replace("  ", "  ./")).unwrap();
    assert_eq!(upgrade(&from(&sb, &dir), None, false).await.unwrap(), 1);
    assert!(sb.err.text().contains("its signature does not match its checksums"));

    fs::remove_file(format!("{dir}/SHA256SUMS.sig")).unwrap();
    assert_eq!(upgrade(&from(&sb, &dir), None, false).await.unwrap(), 1);
    assert!(sb.err.text().contains("the 0.2.0 release cannot be trusted: it has no signature"));

    assert_eq!(tree(&sb), before);
    assert_eq!(versions(&ctx), ["0.1.0"]);
}

#[tokio::test]
async fn an_upgrade_with_a_bad_checksum_changes_nothing() {
    let sb = Sandbox::new();
    let ctx = installed(&sb);
    let before = tree(&sb);
    let code = upgrade(&from(&sb, &release(&sb, "0.2.0", true)), None, false).await.unwrap();
    assert_eq!(code, 1);
    assert!(sb.err.text().contains("does not match its checksum"), "{}", sb.err.text());
    assert_eq!(tree(&sb), before);
    assert_eq!(fs::read_link(&ctx.paths.current).unwrap().to_string_lossy(), "versions/0.1.0");
}

#[tokio::test]
async fn an_upgrade_to_a_release_that_holds_another_versions_files_changes_nothing() {
    let sb = Sandbox::new();
    let ctx = installed(&sb);
    let before = tree(&sb);
    let dir = release(&sb, "0.2.0", false);

    assert_eq!(upgrade(&from(&sb, &dir), Some("0.3.0"), false).await.unwrap(), 1);
    let err = sb.err.text();
    assert!(err.contains("the signed checksums of this release are not those of 0.3.0"), "{err}");

    fs::write(format!("{dir}/VERSION"), "0.9.0\n").unwrap();
    assert_eq!(upgrade(&from(&sb, &dir), None, false).await.unwrap(), 1);
    let err = sb.err.text();
    assert!(err.contains("the signed checksums of this release are not those of 0.9.0"), "{err}");

    assert_eq!(tree(&sb), before);
    assert_eq!(versions(&ctx), ["0.1.0"]);
}

#[tokio::test]
async fn an_archive_whose_binary_is_a_link_or_that_brings_other_files_is_not_installed() {
    let sb = Sandbox::new();
    let ctx = installed(&sb);
    let dir = format!("{}/rel-link", sb.root);
    let stage = format!("{dir}/stage");
    fs::create_dir_all(&stage).unwrap();
    std::os::unix::fs::symlink("/bin/sh", format!("{stage}/aicentre")).unwrap();
    tar(&sb.env, &["-czf", &format!("{dir}/{ASSET}"), "-C", &stage, "aicentre"]);
    sums(&dir, ASSET, "0.2.0");
    assert_eq!(upgrade(&from(&sb, &dir), None, false).await.unwrap(), 1);
    assert!(sb.err.text().contains("cannot unpack"), "{}", sb.err.text());
    assert_eq!(versions(&ctx), ["0.1.0"]);

    let dir = format!("{}/rel-extra", sb.root);
    let stage = format!("{dir}/stage");
    fs::create_dir_all(&stage).unwrap();
    fs::write(format!("{stage}/aicentre"), "#!/bin/sh\necho 0.2.0\n").unwrap();
    fs::set_permissions(format!("{stage}/aicentre"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(format!("{stage}/extra"), "x").unwrap();
    tar(&sb.env, &["-czf", &format!("{dir}/{ASSET}"), "-C", &stage, "aicentre", "extra"]);
    sums(&dir, ASSET, "0.2.0");
    assert_eq!(upgrade(&from(&sb, &dir), None, false).await.unwrap(), 0);
    assert!(!Path::new(&format!("{}/0.2.0/extra", ctx.paths.versions)).exists());
    let mode = fs::metadata(format!("{}/0.2.0", ctx.paths.versions)).unwrap().permissions().mode();
    assert_eq!(mode & 0o022, 0);
}

#[tokio::test]
async fn keep_current_adds_a_version_next_to_the_installed_one() {
    let sb = Sandbox::new();
    let ctx = installed(&sb);
    let older = format!("{}/fake-0.0.9", sb.root);
    fs::write(&older, "#!/bin/sh\necho 0.0.9\n").unwrap();
    fs::set_permissions(&older, fs::Permissions::from_mode(0o755)).unwrap();
    let from_laptop = sb.ctx_with(|c| {
        c.version = "0.0.9".into();
        c.exec_path = older.clone();
    });
    assert_eq!(self_install(&from_laptop, true).await.unwrap(), 0);
    assert_eq!(versions(&ctx), ["0.0.9", "0.1.0"]);
    assert_eq!(fs::read_link(&ctx.paths.current).unwrap().to_string_lossy(), "versions/0.1.0");
    assert_eq!(read_manifest(&ctx.paths).unwrap().version, "0.1.0");
    assert!(sb.out.text().contains("Added aicentre 0.0.9"), "{}", sb.out.text());

    assert_eq!(self_install(&from_laptop, false).await.unwrap(), 0);
    assert_eq!(versions(&ctx), ["0.0.9"]);
}

#[tokio::test]
async fn upgrade_keep_current_checks_a_version_and_adds_it_without_switching() {
    let sb = Sandbox::new();
    let ctx = installed(&sb);
    let dir = release(&sb, "0.0.9", false);
    assert_eq!(upgrade(&from(&sb, &dir), Some("0.0.9"), true).await.unwrap(), 0);
    assert_eq!(sb.err.text(), "");
    assert_eq!(versions(&ctx), ["0.0.9", "0.1.0"]);
    assert_eq!(fs::read_link(&ctx.paths.current).unwrap().to_string_lossy(), "versions/0.1.0");
    assert!(sb.out.text().contains("Added aicentre 0.0.9; 0.1.0 stays the installed version."));

    fs::remove_file(format!("{dir}/SHA256SUMS.sig")).unwrap();
    fs::remove_dir_all(format!("{}/0.0.9", ctx.paths.versions)).unwrap();
    assert_eq!(upgrade(&from(&sb, &dir), Some("0.0.9"), true).await.unwrap(), 1);
    assert_eq!(versions(&ctx), ["0.1.0"]);
}

#[tokio::test]
async fn uninstall_removes_everything_of_its_own_but_the_settings() {
    let sb = Sandbox::new();
    let ctx = installed(&sb);
    sb.file(".config/aicentre/config.toml", "[dtu]\nuser = \"s1\"\n");
    sb.file(".local/state/aicentre/last-check", "x");
    sb.file(".cache/aicentre/leftover", "x");
    fs::create_dir_all(&ctx.paths.runtime).unwrap();
    assert_eq!(uninstall(&ctx, false).await.unwrap(), 0);
    assert!(!Path::new(&ctx.paths.runtime).exists());
    assert_eq!(
        tree(&sb),
        [
            ".cache",
            ".config",
            ".config/aicentre",
            ".config/aicentre/config.toml",
            ".local",
            ".local/state"
        ]
    );
    assert!(sb.out.text().contains("Kept ~/.config/aicentre"));
}

fn run(cmd: &str, args: &[&str], env: &Env) -> std::process::Output {
    Command::new(cmd).args(args).env_clear().envs(env).output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_string()
}

fn build(version: &str, into: &str) {
    let target = concat!(env!("CARGO_TARGET_TMPDIR"), "/e2e-target");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .args(["build", "--release", "--bin", "aicentre", "--features", "test-overrides"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("AICENTRE_BUILD_VERSION", version)
        .env("CARGO_TARGET_DIR", target)
        .status()
        .unwrap();
    assert!(status.success());
    fs::create_dir_all(into).unwrap();
    fs::copy(format!("{target}/release/aicentre"), format!("{into}/aicentre")).unwrap();
}

#[test]
fn end_to_end_install_sh_upgrade_uninstall() {
    if std::env::var("AICENTRE_E2E").as_deref() != Ok("1") {
        return;
    }
    let sb = Sandbox::new();
    let asset = asset_name(current_platform());
    let cli = env!("CARGO_MANIFEST_DIR");
    let mut env = sb.env.clone();
    let path = format!("{}:{}", sb.env["PATH"], std::env::var("PATH").unwrap_or_default());
    env.insert("PATH".into(), path);
    if let Ok(ld) = std::env::var("NIX_LD") {
        env.insert("NIX_LD".into(), ld);
    }
    for v in ["0.1.0", "0.2.0"] {
        let dir = format!("{}/real-{v}", sb.root);
        build(v, &dir);
        tar(&env, &["-czf", &format!("{dir}/{asset}"), "-C", &dir, "aicentre"]);
        sums(&dir, &asset, v);
    }
    let mut first = env.clone();
    first.insert("AICENTRE_DOWNLOAD_BASE".into(), format!("{}/real-0.1.0", sb.root));
    first.insert("AICENTRE_RELEASE_SIGNERS".into(), key().signers.clone());
    first.insert("PATH".into(), std::env::var("PATH").unwrap_or_default());
    let install = run("sh", &[&format!("{cli}/install.sh")], &first);
    assert_eq!(text(&install.stderr), "");
    assert!(install.status.success());
    let bin = format!("{}/.local/bin/aicentre", sb.home);
    assert_eq!(text(&run(&bin, &["--version"], &env).stdout), "0.1.0");

    let mut theirs = first.clone();
    theirs.remove("AICENTRE_RELEASE_SIGNERS");
    let refused = run("sh", &[&format!("{cli}/install.sh")], &theirs);
    assert!(!refused.status.success());
    assert!(
        text(&refused.stderr).contains("SHA256SUMS is not signed with aicentre's release key"),
        "{}",
        text(&refused.stderr)
    );

    let mut other = first.clone();
    other.insert("AICENTRE_VERSION".into(), "0.3.0".into());
    let refused = run("sh", &[&format!("{cli}/install.sh")], &other);
    assert!(!refused.status.success());
    assert!(
        text(&refused.stderr).contains("asked for 0.3.0, but the release is 0.1.0"),
        "{}",
        text(&refused.stderr)
    );

    let mut next = env.clone();
    next.insert("AICENTRE_DOWNLOAD_BASE".into(), format!("{}/real-0.2.0", sb.root));
    next.insert("AICENTRE_RELEASE_SIGNERS".into(), key().signers.clone());
    let up = run(&bin, &["upgrade"], &next);
    assert!(up.status.success(), "{}", text(&up.stderr));
    assert_eq!(text(&run(&bin, &["--version"], &env).stdout), "0.2.0");
    let versions: Vec<String> = fs::read_dir(format!("{}/.local/share/aicentre/versions", sb.home))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(versions, ["0.2.0"]);

    sb.file(".config/aicentre/config.toml", "[dtu]\nuser = \"s1\"\n");
    assert!(run(&bin, &["uninstall"], &env).status.success());
    let left: Vec<String> = tree(&sb).into_iter().filter(|p| p.contains("aicentre")).collect();
    assert_eq!(left, [".config/aicentre", ".config/aicentre/config.toml"]);
}
