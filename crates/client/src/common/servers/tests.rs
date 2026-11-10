use super::*;

fn server(address: &str, key: Option<&str>) -> SavedServer {
    SavedServer {
        address: address.into(),
        key: key.map(Into::into),
        name: None,
    }
}

#[test]
fn the_latest_goes_first_once() {
    let mut preferences = Preferences::default();
    preferences.joined(&server("a.example", None));
    preferences.joined(&server("b.example", None));
    preferences.joined(&server("a.example", Some("key")));
    assert_eq!(
        preferences.servers,
        [server("a.example", Some("key")), server("b.example", None)]
    );
    preferences.forget_server("a.example");
    assert_eq!(preferences.servers, [server("b.example", None)]);
}

#[test]
fn a_server_is_called_by_its_name_unless_it_is_the_default() {
    let mut saved = server("play.example.com:9000", None);
    assert_eq!(saved.label(), "play.example.com:9000");
    saved.name = Some(GAME_NAME.into());
    assert_eq!(saved.label(), "play.example.com:9000");
    saved.name = Some("  ".into());
    assert_eq!(saved.label(), "play.example.com:9000");
    saved.name = Some("Kim's Keep".into());
    assert_eq!(saved.label(), "Kim's Keep");
}

#[test]
fn an_older_saved_server_has_no_name() {
    let saved: SavedServer = ron::from_str(r#"(address: "a.example", key: None)"#).unwrap();
    assert_eq!(saved.name, None);
}
