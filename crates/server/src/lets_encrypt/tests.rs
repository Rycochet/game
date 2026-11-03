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
