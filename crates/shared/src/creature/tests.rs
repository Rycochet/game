use super::*;
use crate::sheet::{Feeling, Reaction};

#[test]
fn every_kind_is_listed_alphabetically_and_no_location_twice() {
    let names: Vec<_> = CreatureKind::ALL.iter().map(|k| k.def().name).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    for kind in CreatureKind::ALL {
        let body = kind.def().body;
        for (index, part) in body.iter().enumerate() {
            assert!(!part.layers.is_empty(), "{:?}", part.location);
            assert!(part.places > 0);
            assert!(
                body[index + 1..]
                    .iter()
                    .all(|o| o.location != part.location)
            );
        }
    }
}

#[test]
fn every_kind_weighs_something() {
    for kind in CreatureKind::ALL {
        assert!(kind.def().weight > 0, "{:?}", kind);
    }
}

#[test]
fn sizes_step_and_stop_at_the_ends() {
    assert_eq!(Size::Medium.stepped(-1), Size::Small);
    assert_eq!(Size::Tiny.stepped(-3), Size::Tiny);
    assert_eq!(Size::Huge.stepped(5), Size::Giant);
    assert_eq!(Size::Tiny.apart(Size::Large), 3);
}

#[test]
fn kinds_like_their_own_and_react_as_they_are() {
    use CreatureKind::{Chicken, Human, Kobold};
    assert_eq!(Kobold.feeling_about(Kobold), Feeling::Friendly);
    assert_eq!(Kobold.feeling_about(Human), Feeling::Unfriendly);
    assert_eq!(Chicken.feeling_about(Human), Feeling::Neutral);
    assert_eq!(Kobold.reaction(Feeling::Unfriendly), Reaction::Attack);
    assert_eq!(Kobold.reaction(Feeling::Friendly), Reaction::Approach);
    assert_eq!(Chicken.reaction(Feeling::Neutral), Reaction::Flee);
    assert_eq!(Human.reaction(Feeling::Neutral), Reaction::Ignore);
}

#[test]
fn finger_counts_differ_by_kind() {
    assert_eq!(CreatureKind::Human.def().fingers(), Some(10));
    assert_eq!(CreatureKind::Kobold.def().fingers(), Some(8));
    assert_eq!(CreatureKind::Chicken.def().fingers(), None);
}
