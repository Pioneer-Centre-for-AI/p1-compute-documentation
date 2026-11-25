use crate::context::Ctx;

pub async fn confirm_yes(ctx: &Ctx, question: &str) -> bool {
    ctx.out.write(&format!("{question} [Y/n] "));
    let answer = tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).map(|read| (read, line))
    })
    .await;
    match answer {
        Ok(Ok((read, line))) if read > 0 => {
            let a = line.trim().to_lowercase();
            a.is_empty() || a == "y" || a == "yes"
        }
        _ => false,
    }
}

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
