use super::*;

#[test]
fn a_newer_game_is_told_the_server_is_behind() {
    let refusal = other_protocol(PROTOCOL_VERSION + 1);
    assert!(refusal.starts_with("this server is older"), "{refusal}");
    assert!(!refusal.contains("reload"), "{refusal}");
}

#[test]
fn an_older_game_is_told_to_update() {
    let refusal = other_protocol(PROTOCOL_VERSION - 1);
    assert!(refusal.starts_with("your game is older"), "{refusal}");
    assert!(refusal.contains("reload the page"), "{refusal}");
}
