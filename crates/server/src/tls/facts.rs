//! `CertificateFacts`: what a certificate says, for showing to people.

use bevy::prelude::*;
use std::net::IpAddr;
use x509_parser::certificate::X509Certificate;
use x509_parser::extensions::GeneralName;
use x509_parser::prelude::FromDer;

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
pub(super) fn ip_address(bytes: &[u8]) -> Option<IpAddr> {
    if let Ok(bytes) = <[u8; 4]>::try_from(bytes) {
        Some(bytes.into())
    } else {
        <[u8; 16]>::try_from(bytes).ok().map(Into::into)
    }
}

/// The fingerprint of a public key (as DER).
pub(super) fn key_fingerprint(public_key: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, public_key)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
