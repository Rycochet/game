use super::*;

#[test]
fn only_secure_web_addresses_are_opened() {
    assert!(safe_to_open(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id=a&state=b%20c"
    ));
    assert!(safe_to_open("http://localhost:8080/authorize?x=1"));
    assert!(safe_to_open("http://127.0.0.1/authorize"));
    assert!(!safe_to_open("http://example.com/authorize"));
    assert!(!safe_to_open("http://localhost.example.com/"));
    assert!(!safe_to_open("file:///C:/Windows/System32/calc.exe"));
    assert!(!safe_to_open("https://example.com/a b"));
    assert!(!safe_to_open("https://example.com/\"&calc"));
    assert!(!safe_to_open("javascript:alert(1)"));
}
