use super::*;

fn made() -> (String, String) {
    let key = rcgen::KeyPair::generate().unwrap();
    let params = rcgen::CertificateParams::new(vec!["localhost".to_owned()]).unwrap();
    let certificate = params.self_signed(&key).unwrap();
    let fingerprint = server::certificate_facts(certificate.der())
        .unwrap()
        .key_fingerprint;
    (certificate.pem(), fingerprint)
}

#[test]
fn a_certificate_gives_its_keys_fingerprint() {
    let (pem, fingerprint) = made();
    assert_eq!(server_key(&pem), Some(fingerprint.clone()));
    // A text box keeps no line breaks.
    assert_eq!(server_key(&pem.replace('\n', "")), Some(fingerprint));
}

#[test]
fn a_fingerprint_is_kept_as_it_is() {
    let (_, fingerprint) = made();
    assert_eq!(
        server_key(&fingerprint.to_uppercase()),
        Some(fingerprint.clone())
    );
    let with_colons = fingerprint
        .as_bytes()
        .chunks(2)
        .map(|pair| std::str::from_utf8(pair).unwrap())
        .collect::<Vec<_>>()
        .join(":");
    assert_eq!(server_key(&with_colons), Some(fingerprint));
}

#[test]
fn anything_else_is_nothing() {
    assert_eq!(server_key(""), None);
    assert_eq!(server_key("hello"), None);
    let not_one = "-----BEGIN CERTIFICATE-----bm90IGEgY2VydA==-----END CERTIFICATE-----";
    assert_eq!(server_key(not_one), None);
    assert_eq!(server_key(&"g".repeat(64)), None);
}

#[test]
fn base64_reads_back() {
    assert_eq!(from_base64("aGVsbG8="), Some(b"hello".to_vec()));
    assert_eq!(from_base64("aGVsbG8"), Some(b"hello".to_vec()));
    assert_eq!(from_base64("a!"), None);
}
