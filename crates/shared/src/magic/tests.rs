use super::*;
use crate::skill::Skills;
use crate::vitals::Points;

fn healer(practice: u32) -> Skills {
    let mut skills = Skills::default();
    skills.practise(heal_skill(), practice);
    skills
}

#[test]
fn healing_is_in_the_tree() {
    assert_eq!(heal_skill().path(), HEAL_SKILL);
}

#[test]
fn six_mana_heals_twelve_at_first_and_a_hundred_and_twenty_at_best() {
    assert_eq!(heal_ratio(None), 2000);
    assert_eq!(heal_amount(None), Points::whole(12));
    assert_eq!(heal_amount(Some(&healer(0))), Points::whole(12));
    let best = healer(1_000_000);
    assert_eq!(heal_ratio(Some(&best)), 20_000);
    assert_eq!(heal_amount(Some(&best)), Points::whole(120));
}

#[test]
fn the_ratio_rises_with_skill() {
    let mut last = 0;
    for practice in [0, 1_000, 8_000, 125_000, 1_000_000, 9_000_000] {
        let ratio = heal_ratio(Some(&healer(practice)));
        assert!(ratio >= last, "{practice}: {ratio} after {last}");
        assert!(ratio <= RATIO_AT_BEST);
        last = ratio;
    }
}

#[test]
fn automatic_healing_waits_for_a_quarter() {
    assert!(needs_healing(Points::whole(5), 20));
    assert!(!needs_healing(Points(5_001), 20));
    assert!(needs_healing(Points::ZERO, 20));
}
