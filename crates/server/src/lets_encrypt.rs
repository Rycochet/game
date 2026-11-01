//! Getting and renewing a certificate from Let's Encrypt.
//!
//! Let's Encrypt checks that a server answers for its domain by connecting to
//! the domain's port 443 and asking, during the TLS handshake, for a special
//! certificate made for this order (TLS-ALPN-01), so no other port is needed.
//! Until it has given a certificate, and if that one ever runs out, `tls` uses
//! the self-signed one.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{fs, io};

use bevy::prelude::*;
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, NewAccount,
    NewOrder, OrderStatus, RetryPolicy,
};
use rcgen::{CertificateParams, CustomExtension, KeyPair};
use rustls::crypto::ring::default_provider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::sign::CertifiedKey;
use time::OffsetDateTime;

use crate::tls::{Current, create_private_folder, read_pair, write_private};

/// What a TLS client offers when it is checking a domain.
pub(crate) const DOMAIN_CHECK_PROTOCOL: &[u8] = b"acme-tls/1";

/// How long to wait after failing before trying again, at first, and at most.
/// Let's Encrypt allows only a few failures an hour.
const FIRST_RETRY: Duration = Duration::from_secs(60 * 60);
const LAST_RETRY: Duration = Duration::from_secs(24 * 60 * 60);

/// The most to sleep with a certificate that needs nothing yet.
const CHECK_AT_LEAST_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// How long Let's Encrypt gets to check the domain and to issue the certificate.
const ORDER_POLICY: RetryPolicy = RetryPolicy::new().timeout(Duration::from_secs(120));

/// Whether a finished handshake was only a domain being checked, with nothing
/// more to come.
pub(crate) fn is_domain_check(protocol: Option<&[u8]>) -> bool {
    protocol == Some(DOMAIN_CHECK_PROTOCOL)
}

/// The state of getting a certificate, shared with the TLS handshakes.
#[derive(Debug)]
pub(crate) struct Issuing {
    domain: String,
    email: Option<String>,
    staging: bool,
    folder: PathBuf,
    /// What to answer a domain check with, during an order.
    check: RwLock<Option<Arc<CertifiedKey>>>,
    issued: RwLock<Option<Current>>,
}

impl Issuing {
    /// Sets up in `folder`, picking up any certificate saved there by an
    /// earlier run. Nothing is asked of Let's Encrypt until `run`.
    pub fn new(
        domain: String,
        email: Option<String>,
        staging: bool,
        folder: PathBuf,
    ) -> Result<Self, String> {
        create_private_folder(&folder)
            .map_err(|error| format!("Can't set up {}: {error}", folder.display()))?;
        let issuing = Self {
            domain,
            email,
            staging,
            folder,
            check: RwLock::default(),
            issued: RwLock::default(),
        };
        if issuing.certificate_path().exists() {
            match read_pair(&issuing.certificate_path(), &issuing.key_path()) {
                Ok(saved) => {
                    info!("Using the saved Let's Encrypt certificate");
                    *issuing.issued.write().expect("never poisoned") = Some(saved);
                }
                Err(error) => warn!("Asking for a new certificate: {error}"),
            }
        }
        Ok(issuing)
    }

    fn certificate_path(&self) -> PathBuf {
        self.folder.join("certificate.pem")
    }

    fn key_path(&self) -> PathBuf {
        self.folder.join("key.pem")
    }

    /// The answer to a domain check for this name, during an order.
    pub fn check_answer(&self, name: Option<&str>) -> Option<Arc<CertifiedKey>> {
        if name != Some(&self.domain) {
            return None;
        }
        self.check.read().expect("never poisoned").clone()
    }

    /// Does something with the issued certificate, if it can be given out now.
    pub fn with_issued<T>(&self, now: i64, using: impl FnOnce(&Current) -> T) -> Option<T> {
        let issued = self.issued.read().expect("never poisoned");
        issued
            .as_ref()
            .filter(|issued| (issued.not_before..=issued.not_after).contains(&now))
            .map(using)
    }

    /// Runs for as long as the server does: asks for a certificate when one is
    /// due, waiting longer after each failure.
    pub async fn run(self: Arc<Self>) {
        let mut failures = 0;
        loop {
            let now = OffsetDateTime::now_utc().unix_timestamp();
            let dates = self
                .issued
                .read()
                .expect("never poisoned")
                .as_ref()
                .map(|issued| (issued.not_before, issued.not_after));
            let wait = match due_in(dates, now) {
                Duration::ZERO => match self.order().await {
                    Ok(issued) => {
                        failures = 0;
                        info!(
                            "Let's Encrypt gave a new certificate: key fingerprint {}",
                            issued.fingerprint
                        );
                        *self.issued.write().expect("never poisoned") = Some(issued);
                        Duration::ZERO
                    }
                    Err(error) => {
                        failures += 1;
                        let wait = retry_after(failures);
                        warn!(
                            "No certificate from Let's Encrypt, trying again in {} minutes (does {}'s port 443 reach this server's secure port?): {error}",
                            wait.as_secs() / 60,
                            self.domain,
                        );
                        wait
                    }
                },
                due => due.min(CHECK_AT_LEAST_EVERY),
            };
            *self.check.write().expect("never poisoned") = None;
            tokio::time::sleep(wait).await;
        }
    }

    /// The account: saved from an earlier run, or made (and saved) now.
    async fn account(&self) -> Result<Account, String> {
        let path = self.folder.join("account.ron");
        match fs::read_to_string(&path) {
            Ok(text) => {
                let credentials: AccountCredentials =
                    ron::from_str(&text).map_err(|error| error.to_string())?;
                return Account::builder()
                    .map_err(|error| error.to_string())?
                    .from_credentials(credentials)
                    .await
                    .map_err(|error| error.to_string());
            }
            Err(error) if error.kind() != io::ErrorKind::NotFound => {
                return Err(error.to_string());
            }
            Err(_) => {}
        }
        let contact = self.email.as_ref().map(|email| format!("mailto:{email}"));
        let contact: Vec<&str> = contact.iter().map(String::as_str).collect();
        let directory = if self.staging {
            instant_acme::LetsEncrypt::Staging
        } else {
            instant_acme::LetsEncrypt::Production
        };
        info!(
            "Accepting Let's Encrypt's terms, as HOW_LETS_ENCRYPT_DOMAIN asks: {}",
            directory.url()
        );
        let (account, credentials) = Account::builder()
            .map_err(|error| error.to_string())?
            .create(
                &NewAccount {
                    contact: &contact,
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                directory.url().to_owned(),
                None,
            )
            .await
            .map_err(|error| error.to_string())?;
        let text = ron::to_string(&credentials).map_err(|error| error.to_string())?;
        write_private(&path, &text).map_err(|error| error.to_string())?;
        Ok(account)
    }

    /// Asks for a certificate, answers the check, and saves what comes back.
    async fn order(&self) -> Result<Current, String> {
        let text = |error: instant_acme::Error| error.to_string();
        let account = self.account().await?;
        let identifiers = [Identifier::Dns(self.domain.clone())];
        let mut order = account
            .new_order(&NewOrder::new(&identifiers))
            .await
            .map_err(text)?;

        let mut authorizations = order.authorizations();
        while let Some(authorization) = authorizations.next().await {
            let mut authorization = authorization.map_err(text)?;
            match authorization.status {
                AuthorizationStatus::Pending => {}
                AuthorizationStatus::Valid => continue,
                status => return Err(format!("the domain's check is {status:?}")),
            }
            let mut challenge = authorization
                .challenge(ChallengeType::TlsAlpn01)
                .ok_or("Let's Encrypt offered no TLS check")?;
            let digest = challenge.key_authorization().digest();
            let answer = check_certificate(&self.domain, digest.as_ref())?;
            *self.check.write().expect("never poisoned") = Some(answer);
            challenge.set_ready().await.map_err(text)?;
        }

        let status = order.poll_ready(&ORDER_POLICY).await.map_err(text)?;
        if status != OrderStatus::Ready {
            let reason = order
                .authorizations()
                .next()
                .await
                .and_then(Result::ok)
                .map(|authorization| format!("{:?}", authorization.status));
            return Err(format!("the order is {status:?} ({reason:?})"));
        }
        let key = order.finalize().await.map_err(text)?;
        let chain = order.poll_certificate(&ORDER_POLICY).await.map_err(text)?;
        self.save(&chain, &key)
    }

    /// Saves a new certificate and reads it back, so a bad one isn't used.
    fn save(&self, chain: &str, key: &str) -> Result<Current, String> {
        write_private(&self.key_path(), key).map_err(|error| error.to_string())?;
        write_private(&self.certificate_path(), chain).map_err(|error| error.to_string())?;
        read_pair(&self.certificate_path(), &self.key_path())
    }
}

/// How long until a certificate should be renewed, with a third of its life left
/// (as Let's Encrypt suggests), or straight away if there is none or it is time.
fn due_in(dates: Option<(i64, i64)>, now: i64) -> Duration {
    let Some((not_before, not_after)) = dates else {
        return Duration::ZERO;
    };
    let renew_at = not_after - (not_after - not_before) / 3;
    Duration::from_secs(u64::try_from(renew_at - now).unwrap_or(0))
}

/// How long to wait after this many failures in a row.
fn retry_after(failures: u32) -> Duration {
    FIRST_RETRY
        .saturating_mul(2u32.saturating_pow(failures.saturating_sub(1)))
        .min(LAST_RETRY)
}

/// The special certificate proving this server is answering for `domain`.
fn check_certificate(domain: &str, digest: &[u8]) -> Result<Arc<CertifiedKey>, String> {
    let text = |error: rcgen::Error| error.to_string();
    let key = KeyPair::generate().map_err(text)?;
    let mut params = CertificateParams::new(vec![domain.to_owned()]).map_err(text)?;
    params.custom_extensions = vec![CustomExtension::new_acme_identifier(digest)];
    let certificate = params.self_signed(&key).map_err(text)?;
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.serialize_der()));
    let signing_key = default_provider()
        .key_provider
        .load_private_key(private_key)
        .map_err(|error| error.to_string())?;
    let chain = vec![CertificateDer::from(certificate.der().to_vec())];
    Ok(Arc::new(CertifiedKey::new(chain, signing_key)))
}

/// The folder Let's Encrypt's files go in, within the certificate folder.
pub(crate) fn folder_within(folder: &Path) -> PathBuf {
    folder.join("acme")
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use x509_parser::certificate::X509Certificate;
    use x509_parser::prelude::FromDer;

    use crate::storage::tests::TestFolder;
    use crate::tls::tests::own_files;

    /// Waiting for a domain check, with its answer ready.
    pub(crate) fn expect_check(issuing: &Issuing) -> Arc<CertifiedKey> {
        let answer = check_certificate(&issuing.domain, &[7; 32]).unwrap();
        *issuing.check.write().unwrap() = Some(answer.clone());
        answer
    }

    /// Holds a certificate as if it had been issued, valid as its files say.
    pub(crate) fn hold_issued(issuing: &Issuing, certificate: &Path, key: &Path) {
        *issuing.issued.write().unwrap() = Some(read_pair(certificate, key).unwrap());
    }

    fn issuing(folder: &TestFolder) -> Issuing {
        Issuing::new("game.example.com".into(), None, true, folder.0.join("acme")).unwrap()
    }

    #[test]
    fn a_certificate_is_due_with_a_third_of_its_life_left() {
        assert_eq!(due_in(None, 0), Duration::ZERO);
        let dates = Some((0, 90));
        assert_eq!(due_in(dates, 0), Duration::from_secs(60));
        assert_eq!(due_in(dates, 59), Duration::from_secs(1));
        assert_eq!(due_in(dates, 60), Duration::ZERO);
        assert_eq!(due_in(dates, 1000), Duration::ZERO);
    }

    #[test]
    fn failures_wait_longer_each_time_up_to_a_day() {
        assert_eq!(retry_after(1), FIRST_RETRY);
        assert_eq!(retry_after(2), FIRST_RETRY * 2);
        assert_eq!(retry_after(3), FIRST_RETRY * 4);
        assert_eq!(retry_after(10), LAST_RETRY);
        assert_eq!(retry_after(u32::MAX), LAST_RETRY);
    }

    #[test]
    fn the_check_certificate_proves_the_order() {
        let digest = [9; 32];
        let answer = check_certificate("game.example.com", &digest).unwrap();
        let (_, parsed) = X509Certificate::from_der(&answer.cert[0]).unwrap();

        // id-pe-acmeIdentifier, critical, holding the digest (RFC 8737).
        let extension = parsed
            .extensions()
            .iter()
            .find(|extension| extension.oid.to_id_string() == "1.3.6.1.5.5.7.1.31")
            .expect("the check's extension");
        assert!(extension.critical);
        assert!(extension.value.ends_with(&digest));
        let facts = crate::tls::certificate_facts(&answer.cert[0]).unwrap();
        assert_eq!(facts.names, ["game.example.com"]);
    }

    #[test]
    fn checks_are_only_answered_during_an_order_and_for_this_domain() {
        let folder = TestFolder::new();
        let issuing = issuing(&folder);
        assert!(issuing.check_answer(Some("game.example.com")).is_none());

        expect_check(&issuing);

        assert!(issuing.check_answer(Some("game.example.com")).is_some());
        assert!(issuing.check_answer(Some("other.example.com")).is_none());
        assert!(issuing.check_answer(None).is_none());
    }

    #[test]
    fn only_the_check_protocol_counts_as_a_check() {
        assert!(is_domain_check(Some(DOMAIN_CHECK_PROTOCOL)));
        assert!(!is_domain_check(Some(b"http/1.1")));
        assert!(!is_domain_check(None));
    }

    #[test]
    fn a_saved_certificate_is_used_on_starting() {
        let folder = TestFolder::new();
        let (certificate, key) = own_files(&folder.0, "own");
        let first = issuing(&folder);
        let saved = first
            .save(
                &fs::read_to_string(&certificate).unwrap(),
                &fs::read_to_string(&key).unwrap(),
            )
            .unwrap();

        let next = issuing(&folder);

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let fingerprint = next.with_issued(now, |issued| issued.fingerprint.clone());
        assert_eq!(fingerprint, Some(saved.fingerprint));
    }

    #[test]
    fn a_broken_saved_certificate_is_asked_for_again() {
        let folder = TestFolder::new();
        let first = issuing(&folder);
        fs::write(first.certificate_path(), "broken").unwrap();
        fs::write(first.key_path(), "broken").unwrap();

        let next = issuing(&folder);

        assert!(next.with_issued(0, |_| ()).is_none());
    }

    #[test]
    fn a_certificate_out_of_date_is_not_used() {
        let folder = TestFolder::new();
        let (certificate, key) = own_files(&folder.0, "own");
        let issuing = issuing(&folder);
        hold_issued(&issuing, &certificate, &key);

        let now = OffsetDateTime::now_utc().unix_timestamp();
        assert!(issuing.with_issued(now, |_| ()).is_some());
        assert!(
            issuing
                .with_issued(now + 11 * 365 * 86400, |_| ())
                .is_none()
        );
        assert!(issuing.with_issued(now - 30 * 86400, |_| ()).is_none());
    }
}
