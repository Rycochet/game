//! Native clients' checks on a `wss://` server's certificate. (In a browser,
//! the browser checks, and a page can't add to that.)
//!
//! A certificate is accepted if the operating system's certificate authorities
//! vouch for it. Otherwise it is self-signed or private, and as nobody vouches
//! for it, its public key is remembered the first time and must match from then
//! on, as SSH does. `HOW_SERVER_KEY` names the key in advance, closing the gap
//! that first time leaves. TLS itself proves the server holds the matching
//! private key.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs, io};

use bevy::log::{info, warn};
use rustls::DigitallySignedStruct;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::ring::default_provider;
use rustls::crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls_platform_verifier::Verifier;
use serde::{Deserialize, Serialize};
use tungstenite::Connector;

use super::socket::Refusal;

/// What the client has learned about a server, by `host:port`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum KnownServer {
    /// A certificate authority vouched for it, so it must go on doing so.
    Authority,
    /// Self-signed, with a public key of this fingerprint.
    Key(String),
}

/// The file of servers this client has met.
#[derive(Debug)]
struct KnownServers(Option<PathBuf>);

impl KnownServers {
    /// `HOW_KNOWN_SERVERS`, or `Hall of Worlds/client` in the user's data
    /// folder, beside the server's own data and separate from it.
    fn from_environment() -> Self {
        Self(
            env::var_os("HOW_KNOWN_SERVERS")
                .map(Into::into)
                .or_else(|| {
                    dirs::data_dir().map(|folder| {
                        folder
                            .join("Hall of Worlds")
                            .join("client")
                            .join("known_servers.ron")
                    })
                }),
        )
    }

    /// A file that can't be read is an error, not an empty list: that would
    /// quietly forget the keys being relied on.
    fn read(&self) -> Result<BTreeMap<String, KnownServer>, String> {
        let Some(path) = &self.0 else {
            return Ok(BTreeMap::new());
        };
        match fs::read_to_string(path) {
            Ok(text) => {
                ron::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(error) => Err(format!("{}: {error}", path.display())),
        }
    }

    fn remember(&self, address: &str, known: KnownServer) {
        let Some(path) = &self.0 else {
            return;
        };
        let saved = self.read().map_err(io::Error::other).and_then(|mut all| {
            all.insert(address.to_owned(), known);
            let text = ron::ser::to_string_pretty(&all, ron::ser::PrettyConfig::default())
                .map_err(io::Error::other)?;
            write_through_new(path, &text)
        });
        if let Err(error) = saved {
            warn!("Can't remember this server in {}: {error}", path.display());
        }
    }
}

/// Writes all of a file or none of it.
fn write_through_new(path: &Path, text: &str) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let new = path.with_extension("ron.new");
    fs::write(&new, text)?;
    fs::rename(new, path)
}

/// The checks for connecting to `host:port`. An `Err` says why it can't be done.
pub(super) fn connector(host: &str, port: u16, refusal: Refusal) -> Result<Connector, String> {
    let provider = Arc::new(default_provider());
    let authority = Verifier::new(provider.clone())
        .map_err(|error| format!("Can't check certificates: {error}"))?;
    let trust = ServerTrust {
        address: format!("{host}:{port}"),
        pinned: env::var("HOW_SERVER_KEY")
            .ok()
            .map(|key| key.trim().to_lowercase()),
        authority: Arc::new(authority),
        known: KnownServers::from_environment(),
        refusal,
        provider: provider.clone(),
    };
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| format!("Can't set up TLS: {error}"))?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(trust))
        .with_no_client_auth();
    Ok(Connector::Rustls(Arc::new(config)))
}

/// The certificate checks of one connection.
#[derive(Debug)]
struct ServerTrust {
    address: String,
    /// The fingerprint `HOW_SERVER_KEY` asks for.
    pinned: Option<String>,
    authority: Arc<dyn ServerCertVerifier>,
    known: KnownServers,
    /// Where to say why, as a failed check mustn't be tried again.
    refusal: Refusal,
    provider: Arc<CryptoProvider>,
}

impl ServerTrust {
    /// Gives up on this server for good, saying why.
    fn refuse(&self, reason: String) -> rustls::Error {
        self.refusal.set(reason.clone());
        rustls::Error::General(reason)
    }

    /// With no authority to vouch for it: is this certificate the one expected?
    fn check_key(&self, fingerprint: &str) -> Result<(), rustls::Error> {
        let address = &self.address;
        let known = self.known.read().map_err(|error| {
            self.refuse(format!("Can't read the servers this game knows: {error}"))
        })?;
        let forget = match &self.known.0 {
            Some(path) => format!("If this is expected, remove it from {}.", path.display()),
            None => "Don't connect unless this is expected.".to_owned(),
        };
        match known.get(address) {
            None => {
                warn!("First visit to {address}: trusting its key, fingerprint {fingerprint}");
                self.known
                    .remember(address, KnownServer::Key(fingerprint.to_owned()));
                Ok(())
            }
            Some(KnownServer::Key(expected)) if expected == fingerprint => Ok(()),
            Some(KnownServer::Key(_)) => Err(self.refuse(format!(
                "{address}'s key has changed, so someone may be pretending to be it. {forget}"
            ))),
            Some(KnownServer::Authority) => Err(self.refuse(format!(
                "{address}'s certificate is no longer backed by a certificate authority, so someone may be pretending to be it. {forget}"
            ))),
        }
    }
}

impl ServerCertVerifier for ServerTrust {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let address = &self.address;
        let Some(facts) = server::certificate_facts(end_entity) else {
            return Err(self.refuse(format!("{address} sent something that isn't a certificate")));
        };
        let seconds = now.as_secs() as i64;
        let in_date = (facts.not_before..=facts.not_after).contains(&seconds);
        let fingerprint = facts.key_fingerprint;

        if let Some(pinned) = &self.pinned {
            if *pinned != fingerprint {
                return Err(self.refuse(format!(
                    "{address}'s key isn't the one HOW_SERVER_KEY names: it has {fingerprint}"
                )));
            }
        } else if self
            .authority
            .verify_server_cert(end_entity, intermediates, server_name, ocsp_response, now)
            .is_ok()
        {
            if self
                .known
                .read()
                .ok()
                .and_then(|all| all.get(address).cloned())
                != Some(KnownServer::Authority)
            {
                info!("{address} is backed by a certificate authority");
                self.known.remember(address, KnownServer::Authority);
            }
            return Ok(ServerCertVerified::assertion());
        } else if in_date {
            self.check_key(&fingerprint)?;
        }
        if !in_date {
            return Err(self.refuse(format!(
                "{address}'s certificate has expired or isn't valid yet"
            )));
        }
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        certificate: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            certificate,
            signature,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        certificate: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            certificate,
            signature,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[cfg(test)]
mod tests {
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
}
