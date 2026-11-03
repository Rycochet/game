use super::*;
use crate::lets_encrypt::{self, DOMAIN_CHECK_PROTOCOL};
use rustls::pki_types::CertificateDer;
use rustls::pki_types::pem::PemObject;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

use crate::storage::tests::TestFolder;

fn settings() -> Settings {
    Settings {
        behind_proxy: false,
        disable_ssl: false,
        address: None,
        certificate: None,
        key: None,
        names: None,
        domain: None,
        email: None,
        staging: false,
    }
}

fn lets_encrypt_settings() -> Settings {
    Settings {
        domain: Some("game.example.com".into()),
        ..settings()
    }
}

fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

fn read_facts(path: &Path) -> CertificateFacts {
    certificate_facts(&CertificateDer::from_pem_file(path).unwrap()).unwrap()
}

/// A key and certificate of their own, as an admin might have.
pub(crate) fn own_files(folder: &Path, name: &str) -> (PathBuf, PathBuf) {
    generated(&folder.join(name), &[], now()).unwrap()
}

fn use_files(certificate: &Path, key: &Path) -> Result<Certificates, String> {
    Certificates::load(&CertificateSource::Files {
        certificate: certificate.into(),
        key: key.into(),
    })
}

#[test]
fn tls_stays_on_behind_a_proxy_which_only_opens_the_plain_port() {
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
            plain_too: false,
        }
    );
    let proxied = Settings {
        behind_proxy: true,
        ..settings()
    };
    assert_eq!(
        Security::from_settings(proxied, data).unwrap(),
        Security::Tls {
            address: DEFAULT_TLS_ADDRESS.parse().unwrap(),
            source: CertificateSource::Generated {
                folder: data.join("tls"),
                names: Vec::new(),
            },
            plain_too: true,
        }
    );
}

#[test]
fn any_certificate_can_be_used_behind_a_proxy() {
    let data = Path::new("data");
    let with_files = Settings {
        behind_proxy: true,
        certificate: Some("c.pem".into()),
        key: Some("k.pem".into()),
        ..settings()
    };
    let Security::Tls {
        source: CertificateSource::Files { .. },
        plain_too: true,
        ..
    } = Security::from_settings(with_files, data).unwrap()
    else {
        panic!("own files, with the plain port open");
    };
    let with_a_domain = Settings {
        behind_proxy: true,
        ..lets_encrypt_settings()
    };
    let Security::Tls {
        source: CertificateSource::LetsEncrypt { .. },
        plain_too: true,
        ..
    } = Security::from_settings(with_a_domain, data).unwrap()
    else {
        panic!("Let's Encrypt, with the plain port open");
    };
}

#[test]
fn ssl_can_only_be_disabled_behind_a_proxy() {
    let data = Path::new("data");
    let alone = Settings {
        disable_ssl: true,
        ..settings()
    };
    let refusal = Security::from_settings(alone, data).unwrap_err();
    assert!(refusal.contains("HOW_BEHIND_PROXY"), "{refusal}");
    let proxied = Settings {
        disable_ssl: true,
        behind_proxy: true,
        ..settings()
    };
    assert_eq!(
        Security::from_settings(proxied, data).unwrap(),
        Security::PlainOnly
    );
}

#[test]
fn disabling_ssl_refuses_certificates_it_would_ignore() {
    let data = Path::new("data");
    let with_files = Settings {
        disable_ssl: true,
        behind_proxy: true,
        certificate: Some("c.pem".into()),
        key: Some("k.pem".into()),
        ..settings()
    };
    assert!(Security::from_settings(with_files, data).is_err());
    let with_a_domain = Settings {
        disable_ssl: true,
        behind_proxy: true,
        ..lets_encrypt_settings()
    };
    assert!(Security::from_settings(with_a_domain, data).is_err());
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
    let bad_address = Settings {
        address: Some("nowhere".into()),
        ..settings()
    };
    assert!(Security::from_settings(bad_address, data).is_err());
}

#[test]
fn lets_encrypt_is_chosen_by_naming_a_domain() {
    let data = Path::new("data");
    let chosen = Settings {
        email: Some("admin@example.com".into()),
        names: Some("other.example.com".into()),
        ..lets_encrypt_settings()
    };
    let Security::Tls { source, .. } = Security::from_settings(chosen, data).unwrap() else {
        panic!("TLS is on");
    };
    assert_eq!(
        source,
        CertificateSource::LetsEncrypt {
            domain: "game.example.com".into(),
            email: Some("admin@example.com".into()),
            staging: false,
            folder: data.join("tls"),
            names: vec!["other.example.com".into()],
        }
    );
}

#[test]
fn lets_encrypt_settings_that_disagree_are_refused() {
    let data = Path::new("data");
    let with_files = Settings {
        certificate: Some("c.pem".into()),
        key: Some("k.pem".into()),
        ..lets_encrypt_settings()
    };
    assert!(Security::from_settings(with_files, data).is_err());
    let with_half_the_files = Settings {
        key: Some("k.pem".into()),
        ..lets_encrypt_settings()
    };
    assert!(Security::from_settings(with_half_the_files, data).is_err());
    let email_alone = Settings {
        email: Some("admin@example.com".into()),
        ..settings()
    };
    assert!(Security::from_settings(email_alone, data).is_err());
}

#[test]
fn only_real_domains_can_be_asked_for() {
    assert!(is_domain("game.example.com"));
    for name in [
        "localhost",
        "127.0.0.1",
        "::1",
        "a..b",
        "a b.com",
        "https://a.com",
    ] {
        assert!(!is_domain(name), "{name}");
    }
}

/// Certificates using Let's Encrypt, with nothing asked of it.
pub(crate) fn lets_encrypt_certificates(folder: &Path) -> Certificates {
    Certificates::load(&CertificateSource::LetsEncrypt {
        domain: "game.example.com".into(),
        email: None,
        staging: true,
        folder: folder.join("tls"),
        names: Vec::new(),
    })
    .unwrap()
}

#[test]
fn until_lets_encrypt_answers_the_self_signed_certificate_is_used() {
    let folder = TestFolder::new();
    let loaded = Arc::new(lets_encrypt_certificates(&folder.0));

    assert!(folder.0.join("tls/acme").is_dir());
    assert!(
        loaded
            .server_config()
            .alpn_protocols
            .contains(&DOMAIN_CHECK_PROTOCOL.to_vec())
    );
    assert_eq!(
        loaded.fingerprint(),
        read_facts(&folder.0.join("tls/certificate.pem")).key_fingerprint
    );
}

#[test]
fn domain_checks_are_only_offered_when_lets_encrypt_is_used() {
    let folder = TestFolder::new();
    let (certificate, key) = own_files(folder.0.as_path(), "own");
    let loaded = Arc::new(use_files(&certificate, &key).unwrap());
    assert_eq!(
        loaded.server_config().alpn_protocols,
        [b"http/1.1".to_vec()]
    );
}

#[test]
fn the_issued_certificate_is_the_one_made_public_while_it_lasts() {
    let folder = TestFolder::new();
    let loaded = lets_encrypt_certificates(&folder.0);
    let own_pem = loaded.public_pem();
    let (certificate, key) = own_files(folder.0.as_path(), "issued");

    lets_encrypt::tests::hold_issued(&loaded.issuing().unwrap(), &certificate, &key);

    assert_ne!(loaded.public_pem(), own_pem);
    assert_eq!(
        loaded.fingerprint(),
        read_facts(&certificate).key_fingerprint
    );
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
