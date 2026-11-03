use super::*;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const ADDRESS: &str = "game.example:8081";

/// A certificate authority that does or doesn't vouch for everything.
#[derive(Debug)]
struct Authority(bool);

impl ServerCertVerifier for Authority {
    fn verify_server_cert(
        &self,
        _: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        if self.0 {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General("unknown issuer".into()))
        }
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        unreachable!()
    }

    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        unreachable!()
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        Vec::new()
    }
}

/// A known servers file of its own, removed at the end.
struct Folder(PathBuf);

impl Folder {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let number = NEXT.fetch_add(1, Ordering::Relaxed);
        let name = format!("how-trust-{}-{number}", std::process::id());
        let folder = env::temp_dir().join(name);
        fs::create_dir_all(&folder).unwrap();
        Self(folder)
    }

    fn file(&self) -> PathBuf {
        self.0.join("known_servers.ron")
    }

    fn trust(&self, authority: bool, pinned: Option<String>) -> ServerTrust {
        ServerTrust {
            address: ADDRESS.into(),
            pinned,
            authority: Arc::new(Authority(authority)),
            known: KnownServers(Some(self.file())),
            refusal: Refusal::default(),
            provider: Arc::new(default_provider()),
        }
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A certificate with a new key, valid from a day ago for `days` days.
fn certificate(days: i64) -> CertificateDer<'static> {
    let now = time::OffsetDateTime::now_utc();
    let key = rcgen::KeyPair::generate().unwrap();
    let mut params = rcgen::CertificateParams::new(vec!["localhost".to_owned()]).unwrap();
    params.not_before = now - time::Duration::days(1);
    params.not_after = now + time::Duration::days(days);
    params.self_signed(&key).unwrap().der().clone()
}

fn check(trust: &ServerTrust, certificate: &CertificateDer<'_>) -> bool {
    trust
        .verify_server_cert(
            certificate,
            &[],
            &ServerName::try_from("game.example").unwrap(),
            &[],
            UnixTime::since_unix_epoch(Duration::from_secs(
                time::OffsetDateTime::now_utc().unix_timestamp() as u64,
            )),
        )
        .is_ok()
}

fn fingerprint(certificate: &CertificateDer<'_>) -> String {
    server::certificate_facts(certificate)
        .unwrap()
        .key_fingerprint
}

#[test]
fn a_new_key_is_remembered_and_accepted_again() {
    let folder = Folder::new();
    let certificate = certificate(30);
    assert!(check(&folder.trust(false, None), &certificate));
    let trust = folder.trust(false, None);
    assert!(check(&trust, &certificate));
    assert_eq!(trust.refusal.take(), None);
}

#[test]
fn a_changed_key_is_refused_for_good() {
    let folder = Folder::new();
    assert!(check(&folder.trust(false, None), &certificate(30)));
    let trust = folder.trust(false, None);
    assert!(!check(&trust, &certificate(30)));
    assert!(trust.refusal.take().unwrap().contains("key has changed"));
}

#[test]
fn keys_are_remembered_per_server() {
    let folder = Folder::new();
    assert!(check(&folder.trust(false, None), &certificate(30)));
    let mut other = folder.trust(false, None);
    other.address = "game.example:9000".into();
    assert!(check(&other, &certificate(30)));
}

#[test]
fn an_authority_is_enough_and_is_then_required() {
    let folder = Folder::new();
    assert!(check(&folder.trust(true, None), &certificate(30)));
    // A renewed certificate, with a new key: still vouched for.
    assert!(check(&folder.trust(true, None), &certificate(30)));
    let trust = folder.trust(false, None);
    assert!(!check(&trust, &certificate(30)));
    assert!(trust.refusal.take().unwrap().contains("no longer backed"));
}

#[test]
fn an_authority_overrides_a_remembered_key() {
    let folder = Folder::new();
    assert!(check(&folder.trust(false, None), &certificate(30)));
    assert!(check(&folder.trust(true, None), &certificate(30)));
}

#[test]
fn an_expired_certificate_is_refused_and_not_remembered() {
    let folder = Folder::new();
    let trust = folder.trust(false, None);
    assert!(!check(&trust, &certificate(-1)));
    assert!(trust.refusal.take().unwrap().contains("expired"));
    assert!(!folder.file().exists());
}

#[test]
fn a_named_key_is_all_that_is_accepted() {
    let folder = Folder::new();
    let certificate = certificate(30);
    let named = fingerprint(&certificate);
    assert!(check(
        &folder.trust(false, Some(named.clone())),
        &certificate
    ));
    // Even if an authority vouches for another one.
    let other = self::certificate(30);
    let trust = folder.trust(true, Some(named.clone()));
    assert!(!check(&trust, &other));
    assert!(trust.refusal.take().unwrap().contains("HOW_SERVER_KEY"));
    let expired = self::certificate(-1);
    let trust = folder.trust(false, Some(fingerprint(&expired)));
    assert!(!check(&trust, &expired));
}

#[test]
fn an_unreadable_file_refuses_rather_than_forgets() {
    let folder = Folder::new();
    fs::write(folder.file(), "not what it should be").unwrap();
    let trust = folder.trust(false, None);
    assert!(!check(&trust, &certificate(30)));
    assert!(trust.refusal.take().is_some());
    assert_eq!(
        fs::read_to_string(folder.file()).unwrap(),
        "not what it should be"
    );
}
