use super::*;

#[test]
fn what_is_kept_shows_again() {
    let settings = OnlineSettings {
        name: "Ryc's world".into(),
        public_address: "play.example.com:9000".into(),
        bind_address: "192.168.0.2".into(),
        port: 9000,
        player_limit: 6,
        // Not a box: the switch is kept by `upnp_row`.
        upnp: false,
    };
    let mut back = OnlineSettings::default();
    for (which, ..) in BOXES {
        assert!(
            take(which, &shown(which, &settings), &mut back),
            "{which:?}"
        );
    }
    assert_eq!(back, settings);
}

#[test]
fn what_isnt_right_changes_nothing() {
    let mut settings = OnlineSettings::default();
    let before = settings.clone();
    for (which, text) in [
        (OptionBox::Name, "  "),
        (OptionBox::PublicAddress, "wss://nope"),
        (OptionBox::BindAddress, "everywhere"),
        (OptionBox::Port, "0"),
        (OptionBox::Port, "70000"),
        (OptionBox::PlayerLimit, ""),
    ] {
        assert!(!take(which, text, &mut settings), "{which:?} {text:?}");
    }
    assert_eq!(settings, before);
    // No public address at all is fine.
    assert!(take(OptionBox::PublicAddress, "", &mut settings));
}
