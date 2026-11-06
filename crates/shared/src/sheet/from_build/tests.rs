use super::*;
use crate::sheet::{Attribute, Dot};

#[test]
fn a_centred_human_has_the_medians_with_body_raised_for_its_point() {
    let human = Attributes::for_kind(CreatureKind::Human, &Build::CENTRED);
    // Body has 2 points at the top: 25% more. Mind and Spirit are at the median.
    assert_eq!(human.strength, 125);
    assert_eq!(human.agility, 125);
    assert_eq!(human.intellect, 100);
    assert_eq!(human.aura, 100);
}

#[test]
fn a_corner_gives_its_kinds_most_and_the_others_least() {
    let build = Build {
        // Turned so Mind is at the bottom and Body and Spirit share the point.
        turn: 60,
        dots: [Dot([1000, 0, 0]), Dot([0, 1000, 0]), Dot([0, 0, 1000])],
    };
    let human = Attributes::for_kind(CreatureKind::Human, &build);
    // Body has 1.5 points: 12.5% more than 160 and 70.
    assert_eq!(human.strength, 180);
    assert_eq!(human.agility, 79);
    assert_eq!(human.perception, 160);
    assert_eq!(human.intellect, 70);
    assert_eq!(human.aura, 180);
}

#[test]
fn a_kinds_bonus_goes_on_top() {
    let dwarf = Attributes::for_kind(CreatureKind::Dwarf, &Build::CENTRED);
    let human = Attributes::for_kind(CreatureKind::Human, &Build::CENTRED);
    let bonus = CreatureKind::Dwarf.def().bonus;
    for which in Attribute::ALL {
        assert_eq!(
            dwarf.get(which),
            human.get(which) + bonus.get(which),
            "{which:?}"
        );
    }
    assert!(bonus.toughness > 0);
}

#[test]
fn every_kind_has_something_of_every_attribute_whatever_is_chosen() {
    let extremes = [Dot([1000, 0, 0]), Dot([0, 1000, 0]), Dot([0, 0, 1000])];
    for kind in CreatureKind::ALL {
        for dot in extremes {
            for turn in [0, 90, 200] {
                let build = Build {
                    dots: [dot; 3],
                    turn,
                };
                let attributes = Attributes::for_kind(kind, &build);
                for which in Attribute::ALL {
                    assert!(attributes.get(which) >= 1, "{kind:?} {which:?}");
                }
            }
        }
    }
}
