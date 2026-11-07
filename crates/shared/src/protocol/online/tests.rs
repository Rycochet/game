use super::*;

#[test]
fn the_old_default_port_moves_to_the_new_one() {
    let mut settings = OnlineSettings {
        port: OLD_DEFAULT_PORT,
        ..Default::default()
    };
    settings.move_off_old_default();
    assert_eq!(settings.port, DEFAULT_PORT);
}

#[test]
fn a_chosen_port_is_kept() {
    let mut settings = OnlineSettings {
        port: 9000,
        ..Default::default()
    };
    settings.move_off_old_default();
    assert_eq!(settings.port, 9000);
}

#[test]
fn settings_saved_before_upnp_leave_it_off() {
    let settings: OnlineSettings = ron::from_str("(port: 9000)").unwrap();
    assert!(!settings.upnp);
    assert_eq!(settings.port, 9000);
}

#[test]
fn online_state_goes_through_postcard() {
    let state = OnlineState::Online {
        certificate: "pem".into(),
        fingerprint: "ab".into(),
        router: RouterPort::Open {
            external: "203.0.113.5:24691".into(),
            shared: false,
        },
    };
    let bytes = postcard::to_allocvec(&state).unwrap();
    assert_eq!(postcard::from_bytes::<OnlineState>(&bytes).unwrap(), state);
}
