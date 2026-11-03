use super::*;

fn server(address: &str, key: Option<&str>) -> SavedServer {
    SavedServer {
        address: address.into(),
        key: key.map(Into::into),
    }
}

#[test]
fn the_latest_goes_first_once() {
    let mut preferences = Preferences::default();
    preferences.connected_to(Some(&server("a.example", None)));
    preferences.connected_to(Some(&server("b.example", None)));
    preferences.connected_to(Some(&server("a.example", Some("key"))));
    assert_eq!(
        preferences.servers,
        [server("a.example", Some("key")), server("b.example", None)]
    );
    assert_eq!(preferences.last_server.as_deref(), Some("a.example"));
    assert_eq!(
        ChosenServer::from_saved(&preferences),
        ChosenServer(Some(server("a.example", Some("key"))))
    );
}

#[test]
fn the_usual_server_is_remembered_as_none() {
    let mut preferences = Preferences::default();
    preferences.connected_to(Some(&server("a.example", None)));
    preferences.connected_to(None);
    assert_eq!(preferences.last_server, None);
    assert_eq!(preferences.servers.len(), 1);
    assert_eq!(ChosenServer::from_saved(&preferences), ChosenServer(None));
}

#[test]
fn forgetting_the_last_one_plays_the_usual_one() {
    let mut preferences = Preferences::default();
    preferences.connected_to(Some(&server("a.example", None)));
    preferences.forget_server("a.example");
    assert!(preferences.servers.is_empty());
    assert_eq!(preferences.last_server, None);
}
