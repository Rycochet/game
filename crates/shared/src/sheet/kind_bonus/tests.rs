use super::*;
use crate::creature::CreatureKind;

fn total(bonus: &Attributes) -> u16 {
    Attribute::ALL.iter().map(|&which| bonus.get(which)).sum()
}

#[test]
fn nothing_set_shares_them_all_evenly() {
    assert_eq!(bonus_from(&[]), Attributes::all(BONUS_POINTS / 9));
}

#[test]
fn what_is_set_comes_first_and_the_rest_is_shared() {
    let bonus = bonus_from(&[(Attribute::Strength, 10), (Attribute::Toughness, 20)]);
    assert_eq!((bonus.strength, bonus.toughness), (10, 20));
    // 24 left among seven: 3 each, and the 3 over to the first three.
    assert_eq!(
        (bonus.agility, bonus.intellect, bonus.perception),
        (4, 4, 4)
    );
    assert_eq!((bonus.will, bonus.aura), (3, 3));
    assert_eq!(total(&bonus), BONUS_POINTS);
}

#[test]
fn a_zero_set_is_kept_out_of_the_sharing() {
    let bonus = bonus_from(&[(Attribute::Intellect, 0)]);
    assert_eq!(bonus.intellect, 0);
    assert_eq!(total(&bonus), BONUS_POINTS);
}

#[test]
fn every_kind_has_the_same_points() {
    for kind in CreatureKind::ALL {
        let def = kind.def();
        let set: u16 = def.bonus_set.iter().map(|&(_, points)| points).sum();
        assert!(set <= BONUS_POINTS, "{} sets {set}", def.name);
        assert_eq!(total(&def.bonus()), BONUS_POINTS, "{}", def.name);
    }
}
