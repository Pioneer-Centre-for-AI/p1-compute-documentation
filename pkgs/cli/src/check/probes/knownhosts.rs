use crate::context::Env;
use crate::exec::{RunOptions, run};

pub async fn known_host(host: &str, port: u16, files: &[String], env: &Env) -> Option<String> {
    let name = if port == 22 { host.to_string() } else { format!("[{host}]:{port}") };
    for file in files {
        let r = run(
            "ssh-keygen",
            &["-F", name.as_str(), "-f", file.as_str()],
            RunOptions::new(env).timeout_ms(5_000),
        )
        .await;
        if r.code == 0 {
            return Some(file.clone());
        }
    }
    None
}
