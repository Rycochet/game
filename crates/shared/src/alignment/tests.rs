use super::deed::{BETRAYED_QUEST, KILLED_BEST, KILLED_NEUTRAL};
use super::kinds::ROLLED_SPREAD;
use super::*;
use crate::creature::CreatureKind;

#[test]
fn starts_neutral_and_stays_within_its_limits() {
    assert_eq!(Alignment::default(), Alignment::NEUTRAL);
    assert_eq!(Alignment::new(5000).value(), MOST_GOOD);
    assert_eq!(Alignment::new(-5000).value(), MOST_EVIL);
    assert_eq!(Alignment::new(990).shifted(50).value(), MOST_GOOD);
}

#[test]
fn bands_split_at_300_either_way() {
    assert_eq!(Alignment::new(-300).band(), Band::Evil);
    assert_eq!(Alignment::new(-299).band(), Band::Neutral);
    assert_eq!(Alignment::new(299).band(), Band::Neutral);
    assert_eq!(Alignment::new(300).band(), Band::Good);
}

#[test]
fn killing_costs_by_how_good_the_victim_was() {
    let killed = |value, provoked| {
        Deed::Killed {
            victim: Alignment::new(value),
            provoked,
        }
        .shift()
    };
    assert_eq!(killed(-800, false), 0);
    assert_eq!(killed(0, false), KILLED_NEUTRAL);
    assert_eq!(killed(300, false), KILLED_NEUTRAL);
    assert_eq!(killed(1000, false), KILLED_BEST);
    let between = killed(650, false);
    assert!(KILLED_BEST < between && between < KILLED_NEUTRAL);
    // Self-defence is free, whoever it was.
    assert_eq!(killed(1000, true), 0);
}

#[test]
fn only_helping_raises_it() {
    for deed in [Deed::Rescued, Deed::Escorted, Deed::Helped] {
        assert!(deed.shift() > 0, "{deed:?}");
    }
    assert_eq!(Deed::BetrayedQuest.shift(), BETRAYED_QUEST);
    let start = Alignment::NEUTRAL;
    assert!(start.after(Deed::Rescued) > start);
    assert!(start.after(Deed::BetrayedQuest) < start);
}

#[test]
fn animals_have_none_and_peoples_lean_their_way() {
    use CreatureKind::{Chicken, Halfling, Horse, Human, Orc, Wolf};
    for animal in [Chicken, Horse, Wolf, CreatureKind::Cattle] {
        assert_eq!(Alignment::rolled(animal, 7), None);
    }
    let average = |kind| {
        let total: i32 = (0..1000u64)
            .map(|hash| i32::from(Alignment::rolled(kind, hash).unwrap().value()))
            .sum();
        total / 1000
    };
    assert!(average(Orc) < -300);
    assert!(average(Halfling) > 100);
    assert!(average(Human).abs() < 20);
}

#[test]
fn rolls_stay_within_the_spread_of_a_leaning() {
    for kind in CreatureKind::ALL {
        let Some(leaning) = kind.alignment_leaning() else {
            continue;
        };
        for hash in 0..600u64 {
            let rolled = i32::from(Alignment::rolled(kind, hash).unwrap().value());
            assert!(
                (rolled - i32::from(leaning)).abs() <= ROLLED_SPREAD,
                "{kind:?}"
            );
        }
    }
}

#[test]
fn every_people_kind_has_one() {
    for kind in CreatureKind::ALL {
        if kind.def().playable {
            assert!(kind.alignment_leaning().is_some(), "{kind:?}");
        }
    }
}
