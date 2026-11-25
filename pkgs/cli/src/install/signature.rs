use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ring::digest::{SHA256, SHA512, digest};
use ring::signature::{ED25519, UnparsedPublicKey};

use crate::context::{Env, for_tests};

pub const SIGNERS: &str = include_str!("../../release.pub");
pub const NAMESPACE: &str = "aicentre-release";
pub const FILE: &str = "SHA256SUMS.sig";
const KIND: &str = "ssh-ed25519";
const MAGIC: &[u8] = b"SSHSIG";

pub fn signers(env: &Env) -> String {
    let base = env.get("AICENTRE_DOWNLOAD_BASE").filter(|b| !b.is_empty());
    match (base, for_tests(env, "AICENTRE_RELEASE_SIGNERS")) {
        (Some(_), Some(own)) => own.to_string(),
        _ => SIGNERS.to_string(),
    }
}

fn keys(signers: &str) -> Vec<Vec<u8>> {
    signers
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace().skip_while(|w| *w != KIND);
            words.next()?;
            let blob = STANDARD.decode(words.next()?).ok()?;
            let mut r = Reader(&blob);
            (r.string()? == KIND.as_bytes()).then_some(())?;
            Some(r.string()?.to_vec())
        })
        .collect()
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.0.len() < n {
            return None;
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Some(head)
    }

    fn string(&mut self) -> Option<&'a [u8]> {
        let n = u32::from_be_bytes(self.take(4)?.try_into().ok()?);
        self.take(n as usize)
    }
}

fn put(out: &mut Vec<u8>, s: &[u8]) {
    out.extend_from_slice(&(s.len() as u32).to_be_bytes());
    out.extend_from_slice(s);
}

pub fn verify(message: &[u8], armored: &str, signers: &str) -> Result<(), String> {
    let bad = || "its signature cannot be read".to_string();
    let body: String = armored.lines().filter(|l| !l.starts_with("-----")).collect();
    let blob = STANDARD.decode(body.trim()).map_err(|_| bad())?;
    let mut r = Reader(&blob);
    if r.take(MAGIC.len()) != Some(MAGIC) || r.take(4) != Some(&[0, 0, 0, 1]) {
        return Err(bad());
    }
    let mut key = Reader(r.string().ok_or_else(bad)?);
    let namespace = r.string().ok_or_else(bad)?;
    let reserved = r.string().ok_or_else(bad)?;
    let hash = r.string().ok_or_else(bad)?;
    let mut signature = Reader(r.string().ok_or_else(bad)?);
    if key.string() != Some(KIND.as_bytes()) || signature.string() != Some(KIND.as_bytes()) {
        return Err("its signature is not an Ed25519 one".into());
    }
    let key = key.string().ok_or_else(bad)?;
    let signature = signature.string().ok_or_else(bad)?;
    if !keys(signers).iter().any(|k| k == key) {
        return Err("it is signed with a key that is not aicentre's release key".into());
    }
    if namespace != NAMESPACE.as_bytes() {
        return Err("its signature was made for something else".into());
    }
    let hashed = match hash {
        b"sha512" => digest(&SHA512, message),
        b"sha256" => digest(&SHA256, message),
        _ => return Err(bad()),
    };
    let mut signed = MAGIC.to_vec();
    put(&mut signed, namespace);
    put(&mut signed, reserved);
    put(&mut signed, hash);
    put(&mut signed, hashed.as_ref());
    UnparsedPublicKey::new(&ED25519, key)
        .verify(&signed, signature)
        .map_err(|_| "its signature does not match its checksums".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "release@hpc.aicentre.dk namespaces=\"aicentre-release\" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIH31tcOuS/7MS5A20WpHLcsg9PgvNa6F6kbXMxnOTif7\n";
    const MESSAGE: &[u8] = b"abc  aicentre-linux-x64.tar.gz\n";
    const SIG: &str = "-----BEGIN SSH SIGNATURE-----
U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAgffW1w65L/sxLkDbRakctyyD0+C
81roXqRtczGc5OJ/sAAAAQYWljZW50cmUtcmVsZWFzZQAAAAAAAAAGc2hhNTEyAAAAUwAA
AAtzc2gtZWQyNTUxOQAAAEBAfMZ+MRtKssdlxF36RgRyf+GVAS2ZepTGxWHHeofOW93Kbv
bksajcgnuhlKK8qIOQC7rokBZlIQ0Uq6mKjicD
-----END SSH SIGNATURE-----
";

    #[test]
    fn a_signature_ssh_keygen_made_verifies_and_nothing_else_does() {
        assert_eq!(verify(MESSAGE, SIG, KEY), Ok(()));
        assert_eq!(
            verify(b"abd  aicentre-linux-x64.tar.gz\n", SIG, KEY),
            Err("its signature does not match its checksums".into())
        );
        assert_eq!(
            verify(MESSAGE, SIG, SIGNERS),
            Err("it is signed with a key that is not aicentre's release key".into())
        );
        assert_eq!(verify(MESSAGE, "", KEY), Err("its signature cannot be read".into()));
        assert_eq!(
            verify(MESSAGE, "-----BEGIN-----\nAAAA\n", KEY),
            Err("its signature cannot be read".into())
        );
    }

    #[test]
    fn the_release_key_in_the_repository_is_one_ed25519_key_and_the_installer_carries_it() {
        assert_eq!(keys(SIGNERS).len(), 1);
        assert_eq!(keys(SIGNERS)[0].len(), 32);
        assert!(include_str!("../../install.sh").contains(SIGNERS.trim()));
    }

    #[test]
    fn another_key_counts_only_next_to_another_download_base() {
        let env = |pairs: &[(&str, &str)]| -> Env {
            pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
        };
        assert_eq!(signers(&env(&[("AICENTRE_RELEASE_SIGNERS", KEY)])), SIGNERS);
        let both = env(&[("AICENTRE_RELEASE_SIGNERS", KEY), ("AICENTRE_DOWNLOAD_BASE", "/x")]);
        assert_eq!(signers(&both), KEY);
    }
}
