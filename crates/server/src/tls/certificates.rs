//! `Certificates`: the certificate in use, reloaded when its files change.

use bevy::prelude::*;
use rustls::ServerConfig;
use rustls::crypto::ring::default_provider;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use time::OffsetDateTime;

use super::{CertificateSource, generated, read_pair};
use crate::lets_encrypt::{self, DOMAIN_CHECK_PROTOCOL, Issuing};

/// The certificate being used: it answers every TLS handshake, and can be
/// swapped while the server runs.
#[derive(Debug)]
pub(crate) struct Certificates {
    pub(super) current: RwLock<Current>,
    /// The admin's files, to notice when they are replaced.
    files: Option<(PathBuf, PathBuf)>,
    /// Getting a certificate from Let's Encrypt, if it is being used.
    issuing: Option<Arc<Issuing>>,
}

#[derive(Debug)]
pub(crate) struct Current {
    pub(super) key: Arc<CertifiedKey>,
    /// When it is valid, in seconds since 1970.
    pub not_before: i64,
    pub not_after: i64,
    /// Certificates only, never a key, even if they share a file.
    pub(super) public_pem: String,
    pub fingerprint: String,
    pub(super) changed: (Option<SystemTime>, Option<SystemTime>),
}

impl Certificates {
    /// Loads or makes the certificate. An `Err` should stop the server: an
    /// admin who named files wants those files used, not something else.
    pub fn load(source: &CertificateSource) -> Result<Self, String> {
        let now = OffsetDateTime::now_utc();
        let (certificate, key, files, issuing) = match source {
            CertificateSource::Generated { folder, names } => {
                let (certificate, key) = generated(folder, names, now)?;
                (certificate, key, None, None)
            }
            CertificateSource::Files { certificate, key } => (
                certificate.clone(),
                key.clone(),
                Some((certificate.clone(), key.clone())),
                None,
            ),
            CertificateSource::LetsEncrypt {
                domain,
                email,
                staging,
                folder,
                names,
            } => {
                let (certificate, key) = generated(folder, names, now)?;
                let issuing = Issuing::new(
                    domain.clone(),
                    email.clone(),
                    *staging,
                    lets_encrypt::folder_within(folder),
                )?;
                (certificate, key, None, Some(Arc::new(issuing)))
            }
        };
        Ok(Self {
            current: RwLock::new(read_pair(&certificate, &key)?),
            files,
            issuing,
        })
    }

    /// Getting a certificate from Let's Encrypt, to be run, if it is being used.
    pub fn issuing(&self) -> Option<Arc<Issuing>> {
        self.issuing.clone()
    }

    /// Does something with the certificate being given out.
    fn with_in_use<T>(&self, using: impl Fn(&Current) -> T) -> T {
        if let Some(issuing) = &self.issuing {
            let now = OffsetDateTime::now_utc().unix_timestamp();
            if let Some(answer) = issuing.with_issued(now, &using) {
                return answer;
            }
        }
        using(&self.current.read().expect("never poisoned"))
    }

    /// The public certificate (and any others in its chain), as PEM.
    pub fn public_pem(&self) -> String {
        self.with_in_use(|current| current.public_pem.clone())
    }

    /// The fingerprint of the public key, for checking by hand.
    pub fn fingerprint(&self) -> String {
        self.with_in_use(|current| current.fingerprint.clone())
    }

    /// Whether there are files to keep an eye on.
    pub fn watches_files(&self) -> bool {
        self.files.is_some()
    }

    /// Picks up the admin's files if they have changed. A broken replacement
    /// (perhaps caught half written) is logged and the current one kept.
    pub fn reload_if_changed(&self) {
        let Some((certificate, key)) = &self.files else {
            return;
        };
        let changed = (modified(certificate), modified(key));
        if changed == self.current.read().expect("never poisoned").changed {
            return;
        }
        match read_pair(certificate, key) {
            Ok(new) => {
                info!(
                    "Using the replaced certificate: key fingerprint {}",
                    new.fingerprint
                );
                *self.current.write().expect("never poisoned") = new;
            }
            Err(error) => error!("Keeping the current certificate: {error}"),
        }
    }

    /// The TLS settings that use this certificate. Only HTTP/1.1 is offered,
    /// which is what the WebSockets and web server here speak.
    pub fn server_config(self: &Arc<Self>) -> ServerConfig {
        let mut config = ServerConfig::builder_with_provider(Arc::new(default_provider()))
            .with_safe_default_protocol_versions()
            .expect("ring supports the default TLS versions")
            .with_no_client_auth()
            .with_cert_resolver(self.clone());
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        if self.issuing.is_some() {
            config.alpn_protocols.push(DOMAIN_CHECK_PROTOCOL.to_vec());
        }
        config
    }
}

impl ResolvesServerCert for Certificates {
    fn resolve(&self, hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        if let Some(issuing) = &self.issuing {
            let checking = hello
                .alpn()
                .is_some_and(|mut offered| offered.any(|name| name == DOMAIN_CHECK_PROTOCOL));
            if checking {
                return issuing.check_answer(hello.server_name());
            }
        }
        Some(self.with_in_use(|current| current.key.clone()))
    }
}

pub(super) fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|file| file.modified()).ok()
}
