use super::*;

#[test]
fn guest_tokens_read_back_and_never_show_in_logs() {
    let token = GuestToken([0xab; 32]);
    assert_eq!(GuestToken::from_hex(&token.to_hex()), Some(token));
    assert_eq!(GuestToken::from_hex("abcd"), None);
    assert_eq!(GuestToken::from_hex(&"é".repeat(32)), None);
    let shown = format!("{:?}", Credentials::Guest(token));
    assert!(!shown.contains("ab"), "{shown}");
}
