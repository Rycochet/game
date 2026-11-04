use super::*;

#[test]
fn preferences_read_back_as_saved() {
    let preferences = Preferences {
        item_pictures: Some("Rings".into()),
        interface_size: InterfaceSize::Percent(150),
        touch: true,
        servers: vec![SavedServer {
            address: "play.example.com:9000".into(),
            key: Some("ab12".into()),
        }],
        last_server: Some("play.example.com:9000".into()),
        online: true,
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
