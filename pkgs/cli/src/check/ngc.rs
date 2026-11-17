use super::env::CheckEnv;
use super::types::{Group, Outcome};

pub async fn check_ngc(g: &mut Group, _c: &CheckEnv<'_>) {
    g.step("access", "Remote desktop", async {
        Ok(Outcome::info(
            "you work on NGC through its Omnissa remote desktop, which this check cannot see",
        ))
    })
    .await;
}
