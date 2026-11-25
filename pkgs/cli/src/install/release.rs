use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use sha2::{Digest, Sha256};

use crate::context::Env;
use crate::paths::is_absolute;

pub const REPO: &str = "https://github.com/Pioneer-Centre-for-AI/p1-compute-documentation";

pub fn download_base(env: &Env, version: Option<&str>) -> String {
    if let Some(base) = env.get("AICENTRE_DOWNLOAD_BASE").filter(|b| !b.is_empty()) {
        return base.trim_end_matches('/').to_string();
    }
    match version {
        Some(v) => format!("{REPO}/releases/download/cli-v{v}"),
        None => format!("{REPO}/releases/latest/download"),
    }
}

fn local_path(base: &str) -> Option<String> {
    if let Some(rest) = base.strip_prefix("file://") {
        let path = rest.strip_prefix("localhost").unwrap_or(rest);
        return Some(percent_decode(path));
    }
    is_absolute(base).then(|| base.to_string())
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadError(pub String);

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DownloadError {}

async fn open(base: &str, name: &str) -> Result<Vec<u8>, DownloadError> {
    if let Some(dir) = local_path(base) {
        let path = format!("{dir}/{name}");
        return std::fs::read(&path).map_err(|_| DownloadError(format!("{path} does not exist")));
    }
    let url = format!("{base}/{name}");
    let res = crate::http::get(&url, Duration::from_secs(120))
        .await
        .map_err(|e| DownloadError(format!("cannot download {url}: {e}")))?;
    if !res.ok() {
        return Err(DownloadError(format!("cannot download {url}: HTTP {}", res.status)));
    }
    Ok(res.body)
}

pub async fn fetch_bytes(base: &str, name: &str) -> Result<Vec<u8>, DownloadError> {
    open(base, name).await
}

pub async fn fetch_text(base: &str, name: &str) -> Result<String, DownloadError> {
    Ok(String::from_utf8_lossy(&open(base, name).await?).into_owned())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

static SUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^([0-9a-f]{64})\s+\*?(\S+)$").unwrap());

pub fn parse_sums(text: &str) -> BTreeMap<String, String> {
    let mut sums = BTreeMap::new();
    for line in text.split('\n') {
        if let Some(m) = SUM.captures(line.trim()) {
            sums.insert(m[2].to_string(), m[1].to_lowercase());
        }
    }
    sums
}

pub fn sha256(path: &str) -> std::io::Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

static SEMVER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$").unwrap());

pub fn is_version(v: &str) -> bool {
    SEMVER.is_match(v)
}

pub fn version_order(a: &str, b: &str) -> std::cmp::Ordering {
    match (semver::Version::parse(a), semver::Version::parse(b)) {
        (Ok(a), Ok(b)) => a.cmp_precedence(&b),
        _ => a.cmp(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_and_versions() {
        let h = "a".repeat(64);
        let sums = parse_sums(&format!(
            "{h}  aicentre-linux-x64.tar.gz\n{}  *VERSION\nnot a line\n",
            "B".repeat(64)
        ));
        assert_eq!(sums["aicentre-linux-x64.tar.gz"], h);
        assert_eq!(sums["VERSION"], "b".repeat(64));
        assert!(is_version("0.2.0") && is_version("0.2.0-rc.1") && !is_version("0.2"));
        use std::cmp::Ordering::*;
        assert_eq!(version_order("0.2.0", "0.10.0"), Less);
        assert_eq!(version_order("0.2.0-rc.1", "0.2.0"), Less);
        assert_eq!(version_order("0.2.0", "0.2.0"), Equal);
    }
}
