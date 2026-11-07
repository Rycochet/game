//! The server's certificate: which one to use (`Security`), making and keeping
//! a self-signed one in the data folder, loading the admin's own files and
//! noticing when they are replaced, or one from Let's Encrypt (`lets_encrypt`).
//!
//! A certificate holds the public key and is given to anyone who connects; the
//! private key proves the server owns it and never leaves the server. Clients
//! remember the public key of a self-signed certificate, so a renewed
//! certificate keeps the same key.

use bevy::prelude::*;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use time::Duration;

use crate::environment::option_on;

// Split into files (`cargo xtask lint` keeps files small).
mod certificates;
mod facts;
mod files;

use certificates::modified;
pub(super) use certificates::{Certificates, Current};
use facts::key_fingerprint;
pub use facts::{CertificateFacts, certificate_facts};
use files::generated;
pub(super) use files::{create_private_folder, read_pair, write_private};

/// Where secure connections are taken, unless `HOW_TLS_ADDRESS` says.
const DEFAULT_TLS_ADDRESS: &str = "0.0.0.0:24691";

/// The names a generated certificate always covers (`HOW_TLS_NAMES` adds more).
const LOCAL_NAMES: [&str; 3] = ["localhost", "127.0.0.1", "::1"];

/// How long a generated certificate lasts.
const LASTS: Duration = Duration::days(3650);

/// A generated certificate with less than this left is renewed on starting.
const RENEW_WITHIN: Duration = Duration::days(365);

/// How the server secures its connections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Security {
    /// Plain HTTP and `ws://` only: TLS is turned off, and a reverse proxy in
    /// front takes care of it.
    PlainOnly,
    /// HTTPS and `wss://` on `address`. The plain port serves the game as well if
    /// `plain_too` (behind a reverse proxy), or else only answers with an error.
    Tls {
        address: SocketAddr,
        source: CertificateSource,
        plain_too: bool,
    },
}

/// Where the certificate comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateSource {
    /// Made here and kept in `folder`, covering `names` as well as this machine.
    Generated { folder: PathBuf, names: Vec<String> },
    /// The admin's own files.
    Files { certificate: PathBuf, key: PathBuf },
    /// From Let's Encrypt, with a certificate made here (kept in `folder`,
    /// covering `names`) until it comes and if it runs out.
    LetsEncrypt {
        domain: String,
        email: Option<String>,
        staging: bool,
        folder: PathBuf,
        names: Vec<String>,
    },
}

/// What the environment says, before it is checked.
struct Settings {
    behind_proxy: bool,
    disable_ssl: bool,
    address: Option<String>,
    certificate: Option<PathBuf>,
    key: Option<PathBuf>,
    names: Option<String>,
    domain: Option<String>,
    email: Option<String>,
    staging: bool,
}

impl Security {
    /// Works it out from `HOW_BEHIND_PROXY`, `HOW_DISABLE_SSL`, `HOW_TLS_ADDRESS`,
    /// `HOW_TLS_CERTIFICATE`, `HOW_TLS_KEY`, `HOW_TLS_NAMES`,
    /// `HOW_LETS_ENCRYPT_DOMAIN`, `HOW_LETS_ENCRYPT_EMAIL` and
    /// `HOW_DEBUG_LETS_ENCRYPT_STAGING`.
    pub fn from_environment(data: &Path) -> Result<Self, String> {
        Self::from_settings(
            Settings {
                behind_proxy: option_on("HOW_BEHIND_PROXY", false),
                disable_ssl: option_on("HOW_DISABLE_SSL", false),
                address: env::var("HOW_TLS_ADDRESS").ok(),
                certificate: env::var_os("HOW_TLS_CERTIFICATE").map(Into::into),
                key: env::var_os("HOW_TLS_KEY").map(Into::into),
                names: env::var("HOW_TLS_NAMES").ok(),
                domain: env::var("HOW_LETS_ENCRYPT_DOMAIN").ok(),
                email: env::var("HOW_LETS_ENCRYPT_EMAIL").ok(),
                staging: option_on("HOW_DEBUG_LETS_ENCRYPT_STAGING", false),
            },
            data,
        )
    }

    fn from_settings(settings: Settings, data: &Path) -> Result<Self, String> {
        if settings.disable_ssl {
            if !settings.behind_proxy {
                return Err(
                    "HOW_DISABLE_SSL=true would leave connections unprotected, so it also needs HOW_BEHIND_PROXY=true"
                        .into(),
                );
            }
            if settings.certificate.is_some() || settings.key.is_some() || settings.domain.is_some()
            {
                return Err(
                    "HOW_DISABLE_SSL=true uses no certificate, so it can't be set with HOW_TLS_CERTIFICATE, HOW_TLS_KEY or HOW_LETS_ENCRYPT_DOMAIN"
                        .into(),
                );
            }
            return Ok(Self::PlainOnly);
        }
        if settings.domain.is_some() && (settings.certificate.is_some() || settings.key.is_some()) {
            return Err(
                "Use either HOW_LETS_ENCRYPT_DOMAIN or your own HOW_TLS_CERTIFICATE and HOW_TLS_KEY, not both"
                    .into(),
            );
        }
        if settings.email.is_some() && settings.domain.is_none() {
            return Err("HOW_LETS_ENCRYPT_EMAIL needs HOW_LETS_ENCRYPT_DOMAIN".into());
        }
        let address = settings.address.as_deref().unwrap_or(DEFAULT_TLS_ADDRESS);
        let address = address.parse().map_err(|_| {
            format!("HOW_TLS_ADDRESS={address} is not an address and port, such as 0.0.0.0:24691")
        })?;
        let folder = data.join("tls");
        let names = settings
            .names
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(Into::into)
            .collect();
        let source = match (settings.certificate, settings.key, settings.domain) {
            (Some(certificate), Some(key), _) => CertificateSource::Files { certificate, key },
            (None, None, Some(domain)) => {
                let domain = domain.trim().to_owned();
                if !is_domain(&domain) {
                    return Err(format!(
                        "HOW_LETS_ENCRYPT_DOMAIN={domain} is not a domain name, such as game.example.com"
                    ));
                }
                CertificateSource::LetsEncrypt {
                    domain,
                    email: settings.email,
                    staging: settings.staging,
                    folder,
                    names,
                }
            }
            (None, None, None) => CertificateSource::Generated { folder, names },
            _ => return Err("HOW_TLS_CERTIFICATE and HOW_TLS_KEY must be set together".into()),
        };
        Ok(Self::Tls {
            address,
            source,
            plain_too: settings.behind_proxy,
        })
    }
}

/// Whether Let's Encrypt could be asked about this name: it needs a real domain,
/// which an address or a name like `localhost` isn't.
fn is_domain(name: &str) -> bool {
    name.contains('.')
        && name.parse::<IpAddr>().is_err()
        && name.split('.').all(|part| {
            !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
}

#[cfg(test)]
pub(crate) mod tests;
