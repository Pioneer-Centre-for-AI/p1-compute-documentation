use std::sync::Arc;

use ring::digest::{SHA256, digest};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, ServerConfig, SignatureScheme};
use tokio_rustls::{TlsAcceptor, TlsConnector};

const NAME: &str = "aicentre-gate";

fn provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

pub fn fingerprint(certificate: &[u8]) -> String {
    hex::encode(digest(&SHA256, certificate))
}

pub struct Sealed {
    pub acceptor: TlsAcceptor,
    pub fingerprint: String,
}

pub fn sealed() -> Result<Sealed, String> {
    let made = rcgen::generate_simple_self_signed(vec![NAME.to_string()])
        .map_err(|e| format!("cannot make a certificate: {e}"))?;
    let certificate = made.cert.der().clone();
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(made.signing_key.serialize_der()));
    let print = fingerprint(&certificate);
    let config = ServerConfig::builder_with_provider(provider())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| e.to_string())?
        .with_no_client_auth()
        .with_single_cert(vec![certificate], key)
        .map_err(|e| e.to_string())?;
    Ok(Sealed { acceptor: TlsAcceptor::from(Arc::new(config)), fingerprint: print })
}

/// Accepts exactly one certificate, by its fingerprint. The signature checks
/// stay, so the other side must also hold that certificate's key.
#[derive(Debug)]
struct Pinned {
    fingerprint: String,
    provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for Pinned {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        if fingerprint(end_entity) == self.fingerprint {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General("this is not the session's gate".into()))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider.signature_verification_algorithms.supported_schemes()
    }
}

pub fn connector(fingerprint: &str) -> Result<TlsConnector, String> {
    let provider = provider();
    let pinned = Pinned { fingerprint: fingerprint.to_string(), provider: provider.clone() };
    let config = ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| e.to_string())?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(pinned))
        .with_no_client_auth();
    Ok(TlsConnector::from(Arc::new(config)))
}

pub fn name() -> ServerName<'static> {
    ServerName::try_from(NAME).expect("a valid name")
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    use super::*;

    async fn serve(gate: &Sealed) -> u16 {
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = server.local_addr().unwrap().port();
        let acceptor = gate.acceptor.clone();
        tokio::spawn(async move {
            while let Ok((conn, _)) = server.accept().await {
                if let Ok(mut tls) = acceptor.accept(conn).await {
                    let _ = tls.write_all(b"hello").await;
                    let _ = tls.shutdown().await;
                }
            }
        });
        port
    }

    #[tokio::test]
    async fn the_laptop_talks_only_to_the_gate_whose_fingerprint_it_was_told() {
        let gate = sealed().unwrap();
        assert_eq!(gate.fingerprint.len(), 64);
        let port = serve(&gate).await;

        let conn = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let mut tls = connector(&gate.fingerprint).unwrap().connect(name(), conn).await.unwrap();
        let mut said = String::new();
        tls.read_to_string(&mut said).await.unwrap();
        assert_eq!(said, "hello");

        let other = sealed().unwrap();
        assert_ne!(other.fingerprint, gate.fingerprint);
        let conn = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let refused = connector(&other.fingerprint).unwrap().connect(name(), conn).await;
        assert!(refused.unwrap_err().to_string().contains("this is not the session's gate"));
    }
}
