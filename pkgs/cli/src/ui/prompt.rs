use crate::context::Ctx;

pub async fn confirm(ctx: &Ctx, question: &str) -> bool {
    ctx.out.write(&format!("{question} [y/N] "));
    let answer = tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).map(|_| line)
    })
    .await;
    match answer {
        Ok(Ok(line)) => {
            let a = line.trim().to_lowercase();
            a == "y" || a == "yes"
        }
        _ => false,
    }
}
