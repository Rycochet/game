use super::name::{Found, name_of};
use crate::common::HoverName;

#[test]
fn names_a_creature_by_its_hover_name() {
    let wolf = HoverName("Wolf".into());
    let found = Found {
        token: Some((None, Some(&wolf))),
        place: Some("Stairs down"),
        ..Found::default()
    };
    assert_eq!(name_of(found).as_deref(), Some("Wolf"));
}

#[test]
fn the_sky_comes_before_the_map() {
    let found = Found {
        sky: Some("Sol"),
        bag: true,
        place: Some("Inn"),
        ..Found::default()
    };
    assert_eq!(name_of(found).as_deref(), Some("Sol"));
}

#[test]
fn a_bag_comes_before_the_place_under_it() {
    let found = Found {
        bag: true,
        place: Some("Stairs up"),
        ..Found::default()
    };
    assert_eq!(name_of(found).as_deref(), Some("Bag"));
    let place = Found {
        place: Some("Stairs up"),
        ..Found::default()
    };
    assert_eq!(name_of(place).as_deref(), Some("Stairs up"));
}

#[test]
fn nothing_hovered_shows_nothing() {
    assert_eq!(name_of(Found::default()), None);
}
