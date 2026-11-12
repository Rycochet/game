use super::*;
use crate::gender::{GENDER_BONUS, Gender};
use crate::sheet::{Attribute, Dot, Row};

#[test]
fn a_centred_human_has_the_medians_with_body_raised_for_its_point() {
    let human = Attributes::for_kind(CreatureKind::Human, Gender::Neutral, &Build::CENTRED);
    // Body has 2 points at the top: 25% more. Mind and Spirit are at the median.
    // A human sets no bonus, so has 6 on each, and Neutral 2 more on Spirit.
    assert_eq!(human.strength, 125 + 6);
    assert_eq!(human.agility, 125 + 6);
    assert_eq!(human.intellect, 100 + 6);
    assert_eq!(human.aura, 100 + 6 + 2);
}

#[test]
fn a_corner_gives_its_kinds_most_and_the_others_least() {
    let build = Build {
        // Turned so Mind is at the bottom and Body and Spirit share the point.
        turn: 60,
        dots: [Dot([1000, 0, 0]), Dot([0, 1000, 0]), Dot([0, 0, 1000])],
    };
    let human = Attributes::for_kind(CreatureKind::Human, Gender::Neutral, &build);
    // Body has 1.5 points: 12.5% more than 160 and 70.
    assert_eq!(human.strength, 180 + 6);
    assert_eq!(human.agility, 79 + 6);
    assert_eq!(human.perception, 160 + 6);
    assert_eq!(human.intellect, 70 + 6);
    assert_eq!(human.aura, 180 + 6 + 2);
}

#[test]
fn a_kinds_bonus_goes_on_top() {
    let dwarf = Attributes::for_kind(CreatureKind::Dwarf, Gender::Neutral, &Build::CENTRED);
    let human = Attributes::for_kind(CreatureKind::Human, Gender::Neutral, &Build::CENTRED);
    // The same medians: only their bonuses differ.
    let (theirs, ours) = (
        CreatureKind::Dwarf.def().bonus(),
        CreatureKind::Human.def().bonus(),
    );
    for which in Attribute::ALL {
        let without = human.get(which) - ours.get(which);
        assert_eq!(dwarf.get(which), without + theirs.get(which), "{which:?}");
    }
    assert_eq!(theirs.toughness, 20);
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
                let attributes = Attributes::for_kind(kind, Gender::Neutral, &build);
                for which in Attribute::ALL {
                    assert!(attributes.get(which) >= 1, "{kind:?} {which:?}");
                }
            }
        }
    }
}

#[test]
fn every_gender_adds_the_same_total_to_its_own_row() {
    let plain = |gender| Attributes::for_kind(CreatureKind::Human, gender, &Build::CENTRED);
    let neutral = plain(Gender::Neutral);
    let without: Vec<i64> = Attribute::ALL
        .into_iter()
        .map(|which| {
            let bonus = if which.row() == Row::Spirit {
                GENDER_BONUS
            } else {
                0
            };
            i64::from(neutral.get(which) - bonus)
        })
        .collect();
    for gender in Gender::ALL {
        let with = plain(gender);
        let mut total = 0;
        for (which, before) in Attribute::ALL.into_iter().zip(&without) {
            let added = i64::from(with.get(which)) - before;
            let expected = if which.row() == gender.row() {
                GENDER_BONUS
            } else {
                0
            };
            assert_eq!(added, i64::from(expected), "{gender:?} {which:?}");
            total += added;
        }
        assert_eq!(total, 3 * i64::from(GENDER_BONUS), "{gender:?}");
    }
}
