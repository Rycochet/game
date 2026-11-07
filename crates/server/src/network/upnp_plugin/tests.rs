use super::*;
use std::path::PathBuf;

fn tls(source: CertificateSource, plain_too: bool) -> Security {
    Security::Tls {
        address: "0.0.0.0:24691".parse().unwrap(),
        source,
        plain_too,
    }
}

fn generated() -> CertificateSource {
    CertificateSource::Generated {
        folder: PathBuf::from("tls"),
        names: Vec::new(),
    }
}

#[test]
fn off_unless_asked() {
    assert!(matches!(
        choose(false, &tls(generated(), false)),
        UpnpChoice::Off
    ));
}

#[test]
fn the_secure_port_is_forwarded_as_itself() {
    let UpnpChoice::Forward(plugin) = choose(true, &tls(generated(), false)) else {
        panic!("not forwarded");
    };
    assert_eq!(plugin.external, 24691);
    assert_eq!(plugin.local.port(), 24691);
}

#[test]
fn lets_encrypt_gets_port_443() {
    let source = CertificateSource::LetsEncrypt {
        domain: "game.example.com".into(),
        email: None,
        staging: false,
        folder: PathBuf::from("tls"),
        names: Vec::new(),
    };
    let UpnpChoice::Forward(plugin) = choose(true, &tls(source, false)) else {
        panic!("not forwarded");
    };
    assert_eq!(plugin.external, 443);
    assert_eq!(plugin.local.port(), 24691);
}

#[test]
fn never_behind_a_proxy() {
    assert!(matches!(
        choose(true, &tls(generated(), true)),
        UpnpChoice::BehindProxy
    ));
    assert!(matches!(
        choose(true, &Security::PlainOnly),
        UpnpChoice::BehindProxy
    ));
}
