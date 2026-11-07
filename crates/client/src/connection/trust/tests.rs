use super::*;

use std::time::Duration;

const ADDRESS: &str = "game.example:24691";

fn trust(key: Option<(String, KeyFrom)>) -> ServerTrust {
    ServerTrust {
        address: ADDRESS.into(),
        key,
        refusal: Refusal::default(),
        provider: Arc::new(default_provider()),
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
fn with_no_key_any_certificate_will_do() {
    let trust = trust(None);
    assert!(check(&trust, &certificate(30)));
    assert!(check(&trust, &certificate(30)));
    assert!(check(&trust, &certificate(-1)));
    assert_eq!(trust.refusal.take(), None);
}

#[test]
fn a_pasted_key_is_all_that_is_accepted() {
    let certificate = certificate(30);
    let key = Some((fingerprint(&certificate), KeyFrom::Pasted));
    assert!(check(&trust(key.clone()), &certificate));
    let trust = trust(key);
    assert!(!check(&trust, &self::certificate(30)));
    let reason = trust.refusal.take().unwrap();
    assert!(reason.contains("doesn't match the one saved"));
}

#[test]
fn a_named_key_says_where_it_came_from() {
    let trust = trust(Some(("0".repeat(64), KeyFrom::Environment)));
    assert!(!check(&trust, &certificate(30)));
    assert!(trust.refusal.take().unwrap().contains("HOW_SERVER_KEY"));
}

#[test]
fn a_keys_certificate_must_be_in_date() {
    let expired = certificate(-1);
    let trust = trust(Some((fingerprint(&expired), KeyFrom::Pasted)));
    assert!(!check(&trust, &expired));
    assert!(trust.refusal.take().unwrap().contains("expired"));
}
