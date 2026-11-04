use super::*;

#[test]
fn a_version_reads_back() {
    let this = ServerVersion::this();
    let json = this.to_json();
    assert!(json.contains("\"game\":\"Hall of Worlds\""), "{json}");
    assert_eq!(ServerVersion::from_json(&json), Some(this.clone()));
    assert!(this.compatible());
}

#[test]
fn other_web_servers_are_not_taken_for_one() {
    for answer in [
        "",
        "<!doctype html><title>Welcome to nginx!</title>",
        "{}",
        r#"{"status":"ok"}"#,
        r#"{"game":"Another Game","version":"1.0.0","protocol":1}"#,
        r#"{"game":"Hall of Worlds","version":"0.1.0"}"#,
    ] {
        assert_eq!(ServerVersion::from_json(answer), None, "{answer}");
    }
}

#[test]
fn another_protocol_is_recognised_but_not_compatible() {
    let answer = format!(
        r#"{{"game":"Hall of Worlds","version":"9.9.9","protocol":{}}}"#,
        PROTOCOL_VERSION + 1
    );
    let version = ServerVersion::from_json(&answer).unwrap();
    assert_eq!(version.version, "9.9.9");
    assert!(!version.compatible());
}
