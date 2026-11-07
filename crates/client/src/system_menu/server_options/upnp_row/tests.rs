use super::*;

fn online(router: RouterPort) -> OnlineState {
    OnlineState::Online {
        certificate: String::new(),
        fingerprint: String::new(),
        router,
    }
}

#[test]
fn an_open_port_gives_the_address_to_copy() {
    let settings = OnlineSettings::default();
    let open = online(RouterPort::Open {
        external: "203.0.113.5:24691".into(),
        shared: false,
    });
    let (words, warning, copy) = router_words(&open, &settings);
    assert!(words.contains("203.0.113.5:24691"), "{words}");
    assert!(!warning);
    assert_eq!(copy.as_deref(), Some("203.0.113.5:24691"));
}

#[test]
fn a_shared_address_or_a_refusal_is_a_warning_with_nothing_to_copy() {
    let settings = OnlineSettings::default();
    let shared = online(RouterPort::Open {
        external: "100.64.0.9:24691".into(),
        shared: true,
    });
    let failed = online(RouterPort::Failed("no router answered".into()));
    for state in [shared, failed] {
        let (words, warning, copy) = router_words(&state, &settings);
        assert!(warning, "{words}");
        assert_eq!(copy, None);
    }
}

#[test]
fn offline_says_what_going_online_will_do() {
    let mut settings = OnlineSettings {
        upnp: true,
        ..Default::default()
    };
    let (words, ..) = router_words(&OnlineState::Offline, &settings);
    assert!(words.contains("asks the router"), "{words}");
    settings.upnp = false;
    let (words, ..) = router_words(&OnlineState::Offline, &settings);
    assert!(words.contains("forwarded"), "{words}");
}

#[test]
fn the_environment_wins_over_the_switch() {
    let settings = OnlineSettings::default();
    assert!(!in_use(&settings, None).upnp);
    assert!(in_use(&settings, Some(true)).upnp);
    let on = OnlineSettings {
        upnp: true,
        ..Default::default()
    };
    assert!(!in_use(&on, Some(false)).upnp);
    assert!(in_use(&on, None).upnp);
}
