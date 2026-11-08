use super::*;

#[test]
fn preferences_read_back_as_saved() {
    let preferences = Preferences {
        item_pictures: Some("Rings".into()),
        dev_area: Some(DevAreaPrefs {
            size: 64,
            generator: "Boreal Forest".into(),
            mobs: "Wolf".into(),
            count: 3,
            seed: "woods".into(),
        }),
        create_mob: Some(CreateMobPrefs {
            kind: "Orc".into(),
            attitude: "Friendly".into(),
            gear: true,
            still: false,
            seed: 9,
        }),
        interface_size: InterfaceSize::Percent(150),
        touch: true,
        servers: vec![SavedServer {
            address: "play.example.com:9000".into(),
            key: Some("ab12".into()),
        }],
        last_server: Some("play.example.com:9000".into()),
        online: true,
        server_difficulties: shared::difficulty::Difficulties::only(
            shared::difficulty::Difficulty::Hard,
        ),
        difficulty: shared::difficulty::Difficulty::Easy,
        online_settings: shared::protocol::OnlineSettings {
            name: "Ryc's world".into(),
            player_limit: 4,
            ..Default::default()
        },
        window: WindowPrefs {
            place: Some(crate::common::display::WindowPlace {
                x: -1200,
                y: 40,
                width: 1280.0,
                height: 720.0,
            }),
            maximised: false,
            monitor: Some(r"\\.\DISPLAY2".into()),
            ..Default::default()
        },
        video: crate::common::VideoPrefs {
            render_scale: 80,
            frame_rate: crate::common::FrameRate::Unlimited,
            ..Default::default()
        },
        pause_unfocused: true,
        controls: {
            let mut controls = crate::common::Controls::default();
            let shift_g = crate::common::Binding::new(KeyCode::KeyG, crate::common::Mods::SHIFT);
            controls.set(crate::common::Action::FullMap, 1, Some(shift_g));
            controls
        },
    };
    assert_eq!(Preferences::from_text(&preferences.to_text()), preferences);
}

#[test]
fn unreadable_or_unknown_preferences_are_the_defaults() {
    assert_eq!(Preferences::from_text("not ron"), Preferences::default());
    let later = "(item_pictures: Some(\"Gems\"), something_new: 3)";
    assert_eq!(
        Preferences::from_text(later).item_pictures.as_deref(),
        Some("Gems")
    );
}

#[test]
fn server_options_saved_on_the_old_default_port_take_the_new_one() {
    let preferences = Preferences::from_text("(online_settings: (port: 8081))");
    assert_eq!(
        preferences.online_settings.port,
        shared::protocol::DEFAULT_PORT
    );
}
