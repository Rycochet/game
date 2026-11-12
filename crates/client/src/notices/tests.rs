use super::*;
use shared::poi::PoiKind;

#[test]
fn a_found_poi_is_named_in_words() {
    assert_eq!(found(PoiKind::Village), "Found: Village");
    assert!(found(PoiKind::HerbalistHut).starts_with("Found: Herbalist"));
}

#[test]
fn the_notice_is_good_markup() {
    NOTICES.assert_good();
}

#[test]
fn a_locked_door_shows_a_notice_that_goes_by_itself() {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use crate::dom::{test_app, test_find};

    let mut app = test_app();
    app.add_message::<FromServer>().add_plugins(NoticesPlugin);
    // Each frame is a second long.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
    app.update();
    let row = test_find(&mut app, "notices/row");
    let hidden = |app: &App| app.world().get::<Node>(row).unwrap().display == Display::None;
    assert!(hidden(&app));

    let locked = ServerMessage::DoorRefused {
        x: 0,
        y: 0,
        locked: true,
    };
    app.world_mut().write_message(FromServer(locked));
    app.update();
    app.update();
    assert!(!hidden(&app));
    let words = test_find(&mut app, "notices/words");
    let said = app.world().get::<Children>(words).unwrap()[0];
    assert_eq!(app.world().get::<Text>(said).unwrap().0, "Locked");

    // (A frame's time is capped well under a second.)
    for _ in 0..40 {
        app.update();
    }
    assert!(hidden(&app));
}
