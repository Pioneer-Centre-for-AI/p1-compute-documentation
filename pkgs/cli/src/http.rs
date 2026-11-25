use std::sync::Once;
use std::time::Duration;

pub struct Fetched {
    pub status: u16,
    pub body: Vec<u8>,
}

impl Fetched {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

const MAX_BODY: usize = 256 << 20;

fn client(timeout: Duration, https_only: bool) -> Result<reqwest::Client, String> {
    static PROVIDER: Once = Once::new();
    PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
    reqwest::Client::builder()
        .timeout(timeout)
        .https_only(https_only)
        .user_agent(concat!("aicentre/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())
}

fn message(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        return "The operation timed out.".into();
    }
    let mut text = e.to_string();
    let mut source = std::error::Error::source(e);
    while let Some(s) = source {
        text = format!("{text}: {s}");
        source = s.source();
    }
    text
}

pub async fn get(url: &str, timeout: Duration) -> Result<Fetched, String> {
    get_up_to(url, timeout, MAX_BODY).await
}

pub async fn get_up_to(url: &str, timeout: Duration, most: usize) -> Result<Fetched, String> {
    let https = url.starts_with("https://");
    let mut res = client(timeout, https)?.get(url).send().await.map_err(|e| message(&e))?;
    let status = res.status().as_u16();
    let mut body = Vec::new();
    while let Some(chunk) = res.chunk().await.map_err(|e| message(&e))? {
        if body.len() + chunk.len() > most {
            return Err(format!("{url}: more than {} MB", most >> 20));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(Fetched { status, body })
}
