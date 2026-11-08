use super::learned::NOT_SEEN;
use super::opposition::{LINKS, opposed_by};
use super::*;

fn find(path: &str) -> SkillId {
    SkillId::find(path).unwrap_or_else(|| panic!("no {path}"))
}

fn sword() -> SkillId {
    find("combat/melee/longsword/longsword")
}

/// A longsword practised to `practice` and used at playtime 0.
fn swordsman(practice: u32) -> Skills {
    let mut skills = Skills::default();
    skills.practise(sword(), practice);
    skills.used(sword(), 0);
    skills
}

#[test]
fn melee_and_everything_under_it_rusts_from_disuse_and_nothing_else_does() {
    for id in SkillId::all() {
        let under_melee = id.upwards().any(|node| node == find("combat/melee"));
        let expected = if under_melee {
            Rust::DISUSED
        } else {
            Rust::STEADY
        };
        assert_eq!(id.rust(), expected, "{}", id.path());
    }
}

#[test]
fn every_link_is_found_and_never_within_one_line_of_descent() {
    for &(from, to) in LINKS {
        let (from, to) = (find(from), find(to));
        assert!(
            !from.upwards().any(|node| node == to) && !to.upwards().any(|node| node == from),
            "{} and {} are in one line",
            from.path(),
            to.path()
        );
    }
    assert_eq!(
        opposed_by(sword()),
        [find("magic"), find("combat/ranged")],
        "a link from a node covers everything under it"
    );
    assert!(opposed_by(find("magic/divine/healing")).is_empty());
}

#[test]
fn disuse_starts_after_eight_hours_and_takes_a_step_an_hour() {
    let mut skills = swordsman(100_000);
    assert!(!skills.rust(8 * HOUR - 1));
    assert_eq!(skills.practice(sword()), 100_000);
    assert!(skills.rust(8 * HOUR));
    // A step is 1% of what is above the floor, half the best.
    assert_eq!(skills.practice(sword()), 99_500);
    assert!(!skills.rust(9 * HOUR - 1), "one step an hour");
    // Asked once at 10 hours or every minute, it comes to the same.
    let mut once = skills.clone();
    once.rust(10 * HOUR);
    for minute in 0..=120 {
        skills.rust(8 * HOUR + minute * HOUR / 60);
    }
    assert_eq!(once, skills);
    // Magic never rusts from disuse.
    let mut caster = Skills::default();
    caster.practise(find("magic/divine/healing"), 10_000);
    caster.used(find("magic/divine/healing"), 0);
    assert!(!caster.rust(1000 * HOUR));
}

#[test]
fn rust_stops_at_half_the_best_and_relearning_is_twice_as_fast() {
    let mut skills = swordsman(100_000);
    skills.rust(10_000 * HOUR);
    assert_eq!(skills.practice(sword()), 50_000);
    assert_eq!(skills.best(sword()), 100_000);
    // Doubled up to the best, then as usual.
    skills.practise(sword(), 20_000);
    assert_eq!(skills.practice(sword()), 90_000);
    skills.practise(sword(), 10_000);
    assert_eq!(skills.practice(sword()), 105_000);
    // A node with the least practice stays known.
    let mut barely = swordsman(1);
    barely.rust(10_000 * HOUR);
    assert!(barely.knows(sword()));
}

#[test]
fn using_a_node_wears_down_only_the_node_it_opposes_once_an_hour() {
    let healing = find("magic/divine/healing");
    let mut skills = swordsman(100_000);
    skills.practise(healing, 100_000);
    skills.used(healing, 0);
    let magic = skills.practice(find("magic"));
    // Cast within the hour: nothing.
    assert!(!skills.used(sword(), HOUR - 1));
    assert!(skills.used(sword(), HOUR));
    assert!(skills.practice(find("magic")) < magic);
    assert_eq!(
        skills.practice(healing),
        100_000,
        "never the spells under it"
    );
    assert!(!skills.used(sword(), 2 * HOUR - 1), "once an hour");
    // Casting keeps it.
    skills.used(healing, 2 * HOUR - 1);
    assert!(!skills.used(sword(), 2 * HOUR));
}

#[test]
fn dying_rusts_every_skill_by_its_hours_and_leaves_the_clocks() {
    let healing = find("magic/divine/healing");
    let mut skills = swordsman(100_000);
    skills.practise(healing, 100_000);
    let clocks = skills.clone();
    skills.rust_for_hours(crate::difficulty::Difficulty::Normal.rust_on_death());
    // 8 steps of 1% of the gap: 50,000 × 0.99⁸ is about 46,140, each step rounded up.
    assert_eq!(skills.practice(sword()), 96_134);
    assert_eq!(skills.practice(healing), 96_134);
    assert_eq!(
        skills.0.values().map(|l| l.clock).collect::<Vec<_>>(),
        clocks.0.values().map(|l| l.clock).collect::<Vec<_>>()
    );
}

#[test]
fn saves_keep_rust_and_old_ones_count_as_used_on_joining() {
    let skills = swordsman(1000);
    let written = ron::to_string(&skills).unwrap();
    assert!(written.contains("(\"combat/melee/longsword/longsword\",1000,1000,0,72000)"));
    assert_eq!(ron::from_str::<Skills>(&written).unwrap(), skills);

    let mut old: Skills = ron::from_str("[(\"combat/melee\", 400)]").unwrap();
    let melee = find("combat/melee");
    assert_eq!(old.best(melee), 400);
    assert_eq!(old.0[&melee].used, NOT_SEEN);
    assert!(!old.rust(1000 * HOUR), "not before joining");
    old.settle(1000 * HOUR);
    assert!(!old.rust(1008 * HOUR - 1));
    assert!(old.rust(1008 * HOUR));
}

/// The curve of a longsword set aside, and of dying: `cargo test -p shared
/// rust_curve -- --nocapture`.
#[test]
fn rust_curve() {
    use crate::difficulty::Difficulty;
    let start = 125_000; // 50%
    println!("A longsword at 50%, set aside:");
    for hours in [0, 8, 12, 24, 48, 96, 168, 336, 720] {
        let mut skills = swordsman(start);
        skills.rust(hours * HOUR);
        let value = skills.value(sword());
        println!("  {hours:>4} h: {}.{:02}%", value / 100, value % 100);
    }
    for difficulty in [Difficulty::Normal, Difficulty::Hard] {
        let mut skills = swordsman(start);
        skills.rust_for_hours(difficulty.rust_on_death());
        let value = skills.value(sword());
        println!(
            "  dying on {}: {}.{:02}%",
            difficulty.name(),
            value / 100,
            value % 100
        );
    }
}
