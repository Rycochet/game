//! The server's certificate: which one to use (`Security`), making and keeping
//! a self-signed one in the data folder, or loading the admin's own files and
//! noticing when they are replaced.
//!
//! A certificate holds the public key and is given to anyone who connects; the
//! private key proves the server owns it and never leaves the server. Clients
//! remember the public key of a self-signed certificate, so a renewed
//! certificate keeps the same key.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use std::{env, fs, io};

use bevy::prelude::*;
use rcgen::{CertificateParams, DnType, KeyPair, PublicKeyData};
use rustls::ServerConfig;
use rustls::crypto::ring::default_provider;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use time::{Duration, OffsetDateTime};
use x509_parser::certificate::X509Certificate;
use x509_parser::extensions::GeneralName;
use x509_parser::prelude::FromDer;

use crate::environment::option_on;

/// Where secure connections are taken, unless `HOW_TLS_ADDRESS` says.
const DEFAULT_TLS_ADDRESS: &str = "0.0.0.0:8081";

/// The names a generated certificate always covers (`HOW_TLS_NAMES` adds more).
const LOCAL_NAMES: [&str; 3] = ["localhost", "127.0.0.1", "::1"];

/// How long a generated certificate lasts.
const LASTS: Duration = Duration::days(3650);

/// A generated certificate with less than this left is renewed on starting.
const RENEW_WITHIN: Duration = Duration::days(365);

/// How the server secures its connections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Security {
    /// Plain HTTP and `ws://`: a reverse proxy in front takes care of TLS.
    BehindProxy,
    /// HTTPS and `wss://` on `address`; the plain port only answers with an error.
    Tls {
        address: SocketAddr,
        source: CertificateSource,
    },
}

/// Where the certificate comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateSource {
    /// Made here and kept in `folder`, covering `names` as well as this machine.
    Generated { folder: PathBuf, names: Vec<String> },
    /// The admin's own files.
    Files { certificate: PathBuf, key: PathBuf },
}

/// What the environment says, before it is checked.
struct Settings {
    behind_proxy: bool,
    address: Option<String>,
    certificate: Option<PathBuf>,
    key: Option<PathBuf>,
    names: Option<String>,
}

impl Security {
    /// Works it out from `HOW_BEHIND_PROXY`, `HOW_TLS_ADDRESS`,
    /// `HOW_TLS_CERTIFICATE`, `HOW_TLS_KEY` and `HOW_TLS_NAMES`.
    pub fn from_environment(data: &Path) -> Result<Self, String> {
        Self::from_settings(
            Settings {
                behind_proxy: option_on("HOW_BEHIND_PROXY", false),
                address: env::var("HOW_TLS_ADDRESS").ok(),
                certificate: env::var_os("HOW_TLS_CERTIFICATE").map(Into::into),
                key: env::var_os("HOW_TLS_KEY").map(Into::into),
                names: env::var("HOW_TLS_NAMES").ok(),
            },
            data,
        )
    }

    fn from_settings(settings: Settings, data: &Path) -> Result<Self, String> {
        if settings.behind_proxy {
            if settings.certificate.is_some() || settings.key.is_some() {
                return Err(
                    "HOW_BEHIND_PROXY=true uses no certificate, so it can't be set with HOW_TLS_CERTIFICATE or HOW_TLS_KEY"
                        .into(),
                );
            }
            return Ok(Self::BehindProxy);
        }
        let address = settings.address.as_deref().unwrap_or(DEFAULT_TLS_ADDRESS);
        let address = address.parse().map_err(|_| {
            format!("HOW_TLS_ADDRESS={address} is not an address and port, such as 0.0.0.0:8081")
        })?;
        let source = match (settings.certificate, settings.key) {
            (Some(certificate), Some(key)) => CertificateSource::Files { certificate, key },
            (None, None) => CertificateSource::Generated {
                folder: data.join("tls"),
                names: settings
                    .names
                    .unwrap_or_default()
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(Into::into)
                    .collect(),
            },
            _ => return Err("HOW_TLS_CERTIFICATE and HOW_TLS_KEY must be set together".into()),
        };
        Ok(Self::Tls { address, source })
    }
}

/// What a client or a log needs to know about a certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateFacts {
    /// SHA-256 of the public key, in lowercase hex: this stays the same when a
    /// certificate is renewed with the same key.
    pub key_fingerprint: String,
    /// When it is valid, in seconds since 1970.
    pub not_before: i64,
    pub not_after: i64,
    /// The names and addresses it covers.
    pub names: Vec<String>,
}

/// Reads a certificate (in its binary form, DER); `None` if it isn't one.
pub fn certificate_facts(der: &[u8]) -> Option<CertificateFacts> {
    let (_, certificate) = X509Certificate::from_der(der).ok()?;
    let validity = certificate.validity();
    let names = certificate
        .subject_alternative_name()
        .ok()
        .flatten()
        .map(|names| {
            names
                .value
                .general_names
                .iter()
                .filter_map(|name| match name {
                    GeneralName::DNSName(name) => Some((*name).to_owned()),
                    GeneralName::IPAddress(bytes) => ip_address(bytes).map(|ip| ip.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    Some(CertificateFacts {
        key_fingerprint: key_fingerprint(certificate.public_key().raw),
        not_before: validity.not_before.timestamp(),
        not_after: validity.not_after.timestamp(),
        names,
    })
}

/// An address as a certificate stores it.
fn ip_address(bytes: &[u8]) -> Option<IpAddr> {
    if let Ok(bytes) = <[u8; 4]>::try_from(bytes) {
        Some(bytes.into())
    } else {
        <[u8; 16]>::try_from(bytes).ok().map(Into::into)
    }
}

/// The fingerprint of a public key (as DER).
fn key_fingerprint(public_key: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, public_key)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The certificate being used: it answers every TLS handshake, and can be
/// swapped while the server runs.
#[derive(Debug)]
pub(crate) struct Certificates {
    current: RwLock<Current>,
    /// The admin's files, to notice when they are replaced.
    files: Option<(PathBuf, PathBuf)>,
}

#[derive(Debug)]
struct Current {
    key: Arc<CertifiedKey>,
    /// Certificates only, never a key, even if they share a file.
    public_pem: String,
    fingerprint: String,
    changed: (Option<SystemTime>, Option<SystemTime>),
}

impl Certificates {
    /// Loads or makes the certificate. An `Err` should stop the server: an
    /// admin who named files wants those files used, not something else.
    pub fn load(source: &CertificateSource) -> Result<Self, String> {
        let (certificate, key, files) = match source {
            CertificateSource::Generated { folder, names } => {
                let (certificate, key) = generated(folder, names, OffsetDateTime::now_utc())?;
                (certificate, key, None)
            }
            CertificateSource::Files { certificate, key } => (
                certificate.clone(),
                key.clone(),
                Some((certificate.clone(), key.clone())),
            ),
        };
        Ok(Self {
            current: RwLock::new(read_pair(&certificate, &key)?),
            files,
        })
    }

    /// The public certificate (and any others in its chain), as PEM.
    pub fn public_pem(&self) -> String {
        self.current
            .read()
            .expect("never poisoned")
            .public_pem
            .clone()
    }

    /// The fingerprint of the public key, for checking by hand.
    pub fn fingerprint(&self) -> String {
        self.current
            .read()
            .expect("never poisoned")
            .fingerprint
            .clone()
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
        config
    }
}

impl ResolvesServerCert for Certificates {
    fn resolve(&self, _: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        Some(self.current.read().expect("never poisoned").key.clone())
    }
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|file| file.modified()).ok()
}

/// Reads a certificate and its key, checking that they belong together.
fn read_pair(certificate: &Path, key: &Path) -> Result<Current, String> {
    let changed = (modified(certificate), modified(key));
    let chain: Vec<CertificateDer> = CertificateDer::pem_file_iter(certificate)
        .and_then(|items| items.collect())
        .map_err(|error| format!("Can't read {}: {error}", certificate.display()))?;
    let first = chain
        .first()
        .ok_or_else(|| format!("{} has no certificate in it", certificate.display()))?;
    let facts = certificate_facts(first)
        .ok_or_else(|| format!("{} is not a certificate", certificate.display()))?;
    let private_key = PrivateKeyDer::from_pem_file(key)
        .map_err(|error| format!("Can't read {}: {error}", key.display()))?;
    let signing_key = default_provider()
        .key_provider
        .load_private_key(private_key)
        .map_err(|error| format!("Can't use {}: {error}", key.display()))?;
    let pair = CertifiedKey::new(chain, signing_key);
    pair.keys_match().map_err(|_| {
        format!(
            "{} is not the key of {}",
            key.display(),
            certificate.display()
        )
    })?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    if now > facts.not_after {
        error!(
            "{} has expired: clients will refuse it until it is replaced",
            certificate.display()
        );
    }
    let text = fs::read_to_string(certificate)
        .map_err(|error| format!("Can't read {}: {error}", certificate.display()))?;
    Ok(Current {
        key: Arc::new(pair),
        public_pem: only_certificates(&text),
        fingerprint: facts.key_fingerprint,
        changed,
    })
}

/// Just the certificates of a PEM file, which may hold its key as well.
fn only_certificates(pem: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in pem.lines() {
        let line = line.trim();
        inside |= line == "-----BEGIN CERTIFICATE-----";
        if inside {
            out.push_str(line);
            out.push('\n');
        }
        inside &= line != "-----END CERTIFICATE-----";
    }
    out
}

/// Makes the key once, and the certificate whenever there is none that covers
/// `names` with long enough left. Returns the paths of both.
fn generated(
    folder: &Path,
    names: &[String],
    now: OffsetDateTime,
) -> Result<(PathBuf, PathBuf), String> {
    let failed = |error: &dyn std::fmt::Display| {
        format!(
            "Can't set up a certificate in {}: {error}",
            folder.display()
        )
    };
    let certificate_path = folder.join("certificate.pem");
    let key_path = folder.join("key.pem");
    fs::create_dir_all(folder).map_err(|error| failed(&error))?;

    let key = match fs::read_to_string(&key_path) {
        Ok(pem) => KeyPair::from_pem(&pem)
            .map_err(|error| format!("Can't use {}: {error}", key_path.display()))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let key = KeyPair::generate().map_err(|error| failed(&error))?;
            write_private(&key_path, &key.serialize_pem()).map_err(|error| failed(&error))?;
            key
        }
        Err(error) => return Err(failed(&error)),
    };

    let mut wanted: Vec<String> = LOCAL_NAMES.iter().map(|&name| name.to_owned()).collect();
    wanted.extend(names.iter().cloned());
    wanted.sort();
    wanted.dedup();

    if !certificate_will_do(&certificate_path, &key, &wanted, now) {
        let mut params = CertificateParams::new(wanted).map_err(|error| failed(&error))?;
        params
            .distinguished_name
            .push(DnType::CommonName, "Hall of Worlds server");
        params.not_before = now - Duration::days(1);
        params.not_after = now + LASTS;
        let certificate = params.self_signed(&key).map_err(|error| failed(&error))?;
        fs::write(&certificate_path, certificate.pem()).map_err(|error| failed(&error))?;
        info!("Made a new certificate in {}", folder.display());
    }
    Ok((certificate_path, key_path))
}

/// Whether the certificate on disk can go on being used.
fn certificate_will_do(path: &Path, key: &KeyPair, wanted: &[String], now: OffsetDateTime) -> bool {
    let Ok(der) = CertificateDer::from_pem_file(path) else {
        return false;
    };
    let Some(facts) = certificate_facts(&der) else {
        return false;
    };
    let mut names = facts.names;
    names.sort();
    facts.key_fingerprint == key_fingerprint(&key.subject_public_key_info())
        && names == wanted
        && facts.not_after > (now + RENEW_WITHIN).unix_timestamp()
}

/// Writes a private key that only this user can read (on Unix; on Windows the
/// user's data folder is already private).
fn write_private(path: &Path, contents: &str) -> io::Result<()> {
    use std::io::Write;

    let new = path.with_extension("pem.new");
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(&new)?.write_all(contents.as_bytes())?;
    fs::rename(new, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::storage::tests::TestFolder;

    fn settings() -> Settings {
        Settings {
            behind_proxy: false,
            address: None,
            certificate: None,
            key: None,
            names: None,
        }
    }

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    fn read_facts(path: &Path) -> CertificateFacts {
        certificate_facts(&CertificateDer::from_pem_file(path).unwrap()).unwrap()
    }

    /// A key and certificate of their own, as an admin might have.
    fn own_files(folder: &Path, name: &str) -> (PathBuf, PathBuf) {
        generated(&folder.join(name), &[], now()).unwrap()
    }

    fn use_files(certificate: &Path, key: &Path) -> Result<Certificates, String> {
        Certificates::load(&CertificateSource::Files {
            certificate: certificate.into(),
            key: key.into(),
        })
    }

    #[test]
    fn tls_is_on_unless_behind_a_proxy() {
        let data = Path::new("data");
        let secure = Security::from_settings(settings(), data).unwrap();
        assert_eq!(
            secure,
            Security::Tls {
                address: DEFAULT_TLS_ADDRESS.parse().unwrap(),
                source: CertificateSource::Generated {
                    folder: data.join("tls"),
                    names: Vec::new(),
                },
            }
        );
        let proxied = Settings {
            behind_proxy: true,
            ..settings()
        };
        assert_eq!(
            Security::from_settings(proxied, data).unwrap(),
            Security::BehindProxy
        );
    }

    #[test]
    fn settings_that_disagree_are_refused() {
        let data = Path::new("data");
        let only_certificate = Settings {
            certificate: Some("c.pem".into()),
            ..settings()
        };
        assert!(Security::from_settings(only_certificate, data).is_err());
        let only_key = Settings {
            key: Some("k.pem".into()),
            ..settings()
        };
        assert!(Security::from_settings(only_key, data).is_err());
        let proxied_with_files = Settings {
            behind_proxy: true,
            certificate: Some("c.pem".into()),
            key: Some("k.pem".into()),
            ..settings()
        };
        assert!(Security::from_settings(proxied_with_files, data).is_err());
        let bad_address = Settings {
            address: Some("nowhere".into()),
            ..settings()
        };
        assert!(Security::from_settings(bad_address, data).is_err());
    }

    #[test]
    fn a_generated_certificate_is_kept_between_runs() {
        let folder = TestFolder::new();
        let (certificate, key) = generated(folder.0.as_path(), &[], now()).unwrap();
        let first = fs::read_to_string(&certificate).unwrap();
        let first_key = fs::read_to_string(&key).unwrap();

        generated(folder.0.as_path(), &[], now()).unwrap();

        assert_eq!(fs::read_to_string(&certificate).unwrap(), first);
        assert_eq!(fs::read_to_string(&key).unwrap(), first_key);
        let facts = read_facts(&certificate);
        assert!(facts.names.iter().any(|name| name == "localhost"));
        assert!(facts.names.iter().any(|name| name == "127.0.0.1"));
        assert!(facts.names.iter().any(|name| name == "::1"));
    }

    #[test]
    fn renewing_keeps_the_key() {
        let folder = TestFolder::new();
        let long_ago = now() - LASTS + Duration::days(10);
        let (certificate, _) = generated(folder.0.as_path(), &[], long_ago).unwrap();
        let old = read_facts(&certificate);

        generated(folder.0.as_path(), &[], now()).unwrap();

        let new = read_facts(&certificate);
        assert!(new.not_after > old.not_after);
        assert_eq!(new.key_fingerprint, old.key_fingerprint);
    }

    #[test]
    fn new_names_make_a_new_certificate_with_the_same_key() {
        let folder = TestFolder::new();
        let (certificate, _) = generated(folder.0.as_path(), &[], now()).unwrap();
        let old = read_facts(&certificate);

        generated(folder.0.as_path(), &["game.example".into()], now()).unwrap();

        let new = read_facts(&certificate);
        assert!(new.names.iter().any(|name| name == "game.example"));
        assert_eq!(new.key_fingerprint, old.key_fingerprint);
    }

    #[test]
    fn own_files_are_used_and_nothing_is_generated() {
        let folder = TestFolder::new();
        let (certificate, key) = own_files(folder.0.as_path(), "own");
        let loaded = use_files(&certificate, &key).unwrap();
        assert_eq!(
            loaded.fingerprint(),
            read_facts(&certificate).key_fingerprint
        );
        assert_eq!(
            fs::read_dir(folder.0.as_path()).unwrap().count(),
            1,
            "only the admin's own folder"
        );
    }

    #[test]
    fn missing_or_mismatched_files_are_refused() {
        let folder = TestFolder::new();
        let (certificate, key) = own_files(folder.0.as_path(), "one");
        let (_, other_key) = own_files(folder.0.as_path(), "two");
        assert!(use_files(&certificate, &other_key).is_err());
        assert!(use_files(&folder.0.as_path().join("none.pem"), &key).is_err());
        assert!(use_files(&certificate, &folder.0.as_path().join("none.pem")).is_err());
        assert!(use_files(&key, &key).is_err(), "a key is not a certificate");
    }

    #[test]
    fn replaced_files_are_picked_up_and_broken_ones_ignored() {
        let folder = TestFolder::new();
        let (certificate, key) = own_files(folder.0.as_path(), "own");
        let (new_certificate, new_key) = own_files(folder.0.as_path(), "new");
        let loaded = use_files(&certificate, &key).unwrap();
        let first = loaded.fingerprint();

        // Half written: there is no certificate in it.
        fs::write(&certificate, "-----BEGIN").unwrap();
        loaded.current.write().unwrap().changed = (None, None);
        loaded.reload_if_changed();
        assert_eq!(loaded.fingerprint(), first);
        assert!(loaded.public_pem().contains("BEGIN CERTIFICATE"));

        fs::copy(&new_certificate, &certificate).unwrap();
        fs::copy(&new_key, &key).unwrap();
        loaded.current.write().unwrap().changed = (None, None);
        loaded.reload_if_changed();
        assert_ne!(loaded.fingerprint(), first);
        assert_eq!(
            loaded.fingerprint(),
            read_facts(&new_certificate).key_fingerprint
        );
    }

    #[test]
    fn a_key_sharing_a_file_is_never_made_public() {
        let folder = TestFolder::new();
        let (certificate, key) = own_files(folder.0.as_path(), "own");
        let combined = folder.0.as_path().join("combined.pem");
        let key_text = fs::read_to_string(&key).unwrap();
        let certificate_text = fs::read_to_string(&certificate).unwrap();
        fs::write(&combined, format!("{key_text}{certificate_text}")).unwrap();

        let loaded = use_files(&combined, &combined).unwrap();

        assert!(loaded.public_pem().contains("BEGIN CERTIFICATE"));
        assert!(!loaded.public_pem().contains("PRIVATE KEY"));
    }
}
