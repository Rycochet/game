use super::*;

fn parsed(text: &str) -> Option<(String, u16)> {
    ServerAddress::parse(text).map(|address| (address.host, address.port))
}

#[test]
fn hosts_with_and_without_ports() {
    assert_eq!(
        parsed("play.example.com"),
        Some(("play.example.com".into(), DEFAULT_PORT))
    );
    assert_eq!(
        parsed(" Play.Example.com:9000 "),
        Some(("play.example.com".into(), 9000))
    );
    assert_eq!(
        parsed("192.168.0.2:8081"),
        Some(("192.168.0.2".into(), 8081))
    );
    assert_eq!(
        parsed("localhost"),
        Some(("localhost".into(), DEFAULT_PORT))
    );
    assert_eq!(parsed("[::1]:9000"), Some(("::1".into(), 9000)));
    assert_eq!(parsed("[fe80::1]"), Some(("fe80::1".into(), DEFAULT_PORT)));
}

#[test]
fn not_addresses() {
    for text in [
        "",
        "wss://play.example.com",
        "play.example.com/ws",
        "play.example.com:",
        "play.example.com:0",
        "play.example.com:70000",
        "play example.com",
        "-bad.example.com",
        "a..b",
        "[::1",
        "[::1]9000",
        "[nothost]",
    ] {
        assert_eq!(parsed(text), None, "{text:?}");
    }
}

#[test]
fn shown_as_typed_and_its_socket() {
    let address = ServerAddress::parse("play.example.com").unwrap();
    assert_eq!(address.to_string(), "play.example.com");
    assert_eq!(address.url(), "wss://play.example.com:8081/ws");
    let address = ServerAddress::parse("[::1]:9000").unwrap();
    assert_eq!(address.to_string(), "[::1]:9000");
    assert_eq!(address.url(), "wss://[::1]:9000/ws");
}
