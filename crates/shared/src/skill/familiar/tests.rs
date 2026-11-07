use super::*;
use crate::skill::{SkillId, Skills};

#[test]
fn familiarity_is_the_square_root_of_blocks_fought() {
    assert_eq!(familiarity(0), 0);
    assert_eq!(familiarity(100), 10 * VALUE_SCALE);
    assert_eq!(familiarity(10_000), 100 * VALUE_SCALE);
}

#[test]
fn practice_halves_every_ten_points_over() {
    let ten = FAMILIAR_HALVING;
    assert_eq!(taught(8, 0, 0), 8);
    assert_eq!(taught(8, 500, 900), 8, "below the skill: all of it");
    assert_eq!(taught(8, 901, 900), 8, "just over still rounds to all");
    assert_eq!(taught(8, ten / 2, 0), 6);
    assert_eq!(taught(8, ten, 0), 4);
    assert_eq!(taught(8, 2 * ten, 0), 2);
    assert_eq!(taught(8, 3 * ten, 0), 1);
    assert_eq!(taught(8, 5 * ten, 0), 0);
    assert_eq!(taught(8, u32::MAX, 0), 0);
}

/// Farming one kind, with every fight 20 seconds (four blocks of 8 practice), as
/// the owner asked to see: `cargo test -p shared farming -- --nocapture` prints it.
#[test]
fn farming_one_kind_stalls() {
    let skill = SkillId::find("combat/melee/unarmed/human").unwrap();
    let (mut skills, mut fought) = (Skills::default(), 0);
    let mut seen = Vec::new();
    for kill in 1..=1000 {
        let before = skills.practice(skill);
        for _ in 0..4 {
            let amount = taught(8, familiarity(fought), skills.value(skill));
            skills.practise(skill, amount);
            fought += 1;
        }
        if [1, 10, 100, 1000].contains(&kill) {
            let gained = skills.practice(skill) - before;
            let value = skills.value(skill);
            println!(
                "{kill:>5} kills: {gained:>2} a kill, {} practice, {}.{:02}%, familiarity {}%",
                skills.practice(skill),
                value / 100,
                value % 100,
                familiarity(fought) / 100,
            );
            seen.push(value);
        }
    }
    // As planned: full until familiarity passes the skill, then far slower.
    assert!((680..=690).contains(&seen[1]), "{seen:?}");
    assert!((1300..=1450).contains(&seen[2]), "{seen:?}");
    assert!((1900..=2200).contains(&seen[3]), "{seen:?}");
}
