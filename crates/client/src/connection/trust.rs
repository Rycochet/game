//! Native clients' checks on a `wss://` server's certificate. (In a browser,
//! the browser checks, and a page can't add to that.)
//!
//! Owner's decision (2026-11-03): with no key given, any certificate is
//! accepted, so a server may change its certificate freely. A key pasted into
//! Connect (or `HOW_SERVER_KEY`) must be the certificate's, which closes that
//! for anyone who wants it. TLS itself proves the server holds the matching
//! private key.

use std::sync::Arc;

use bevy::log::info;
use rustls::DigitallySignedStruct;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::ring::default_provider;
use rustls::crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use tungstenite::Connector;

use super::socket::Refusal;

// Split into files (`cargo xtask lint` keeps files small).
mod key;

pub use key::server_key;

/// Where a key to check against came from, for saying why it failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KeyFrom {
    /// Pasted in Connect, and saved with the server.
    Pasted,
    /// `HOW_SERVER_KEY`.
    Environment,
}

/// The checks for connecting to `host:port`: `key` is the fingerprint the
/// certificate's public key must have, if any. An `Err` says why it can't be done.
pub(super) fn connector(
    host: &str,
    port: u16,
    key: Option<(String, KeyFrom)>,
    refusal: Refusal,
) -> Result<Connector, String> {
    let provider = Arc::new(default_provider());
    let trust = ServerTrust {
        address: format!("{host}:{port}"),
        key,
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
    key: Option<(String, KeyFrom)>,
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
}

impl ServerCertVerifier for ServerTrust {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let address = &self.address;
        let Some(facts) = server::certificate_facts(end_entity) else {
            return Err(self.refuse(format!("{address} sent something that isn't a certificate")));
        };
        let Some((expected, from)) = &self.key else {
            info!(
                "{address}'s key has the fingerprint {}",
                facts.key_fingerprint
            );
            return Ok(ServerCertVerified::assertion());
        };
        if facts.key_fingerprint != *expected {
            return Err(self.refuse(match from {
                KeyFrom::Pasted => format!(
                    "{address}'s certificate doesn't match the one saved for it, so this may not \
                     be the server you meant to reach. If its owner has changed it, ask them for \
                     the new one and paste it in with the padlock in Connect."
                ),
                KeyFrom::Environment => format!(
                    "{address}'s key isn't the one HOW_SERVER_KEY names: it has {}",
                    facts.key_fingerprint
                ),
            }));
        }
        let seconds = now.as_secs() as i64;
        if !(facts.not_before..=facts.not_after).contains(&seconds) {
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
mod tests;
