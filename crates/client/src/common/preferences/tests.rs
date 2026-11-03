use super::*;

#[test]
fn preferences_read_back_as_saved() {
    let preferences = Preferences {
        item_pictures: Some("Rings".into()),
        interface_size: InterfaceSize::Percent(150),
        touch: true,
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
