use super::*;
use crate::blade::Family;
use crate::creature::Size;
use crate::dungeon::Level;
use crate::item::Quality;
use crate::rng::Seed;
use crate::sheet::{Attribute, Attributes};
use crate::spot::Spot;

fn blade(family: Family, grams: u32, length_mm: u32) -> Weapon {
    Weapon {
        family: Some(family),
        grams,
        length_mm,
        quality: Quality::Common,
        metal_tier: 2,
        skill: None,
    }
}

fn sword() -> Weapon {
    blade(Family::Sword, 1200, 1000)
}

#[test]
fn bare_hands_attack_twice_a_second_and_a_sword_once() {
    assert_eq!(attack_cost(&Weapon::UNARMED, 100, 0, None), 2100);
    assert_eq!(attack_cost(&sword(), 100, 0, None), 4200);
    let dagger = attack_cost(&blade(Family::Dagger, 300, 350), 100, 0, None);
    assert!((2300..2700).contains(&dagger), "{dagger}");
    let polearm = attack_cost(&blade(Family::Polearm, 2600, 2200), 100, 0, None);
    assert!((7500..9000).contains(&polearm), "{polearm}");
}

#[test]
fn agility_quickens_only_the_weapon_and_only_so_far() {
    assert_eq!(attack_cost(&sword(), 200, 0, None), 2100 + 1050);
    assert_eq!(attack_cost(&sword(), 1000, 0, None), 2100 + 1050);
    assert_eq!(attack_cost(&sword(), 25, 0, None), 2100 + 4200);
    assert_eq!(attack_cost(&Weapon::UNARMED, 1000, 0, None), 2100);
}

#[test]
fn skill_and_dual_wielding_take_some_off() {
    assert_eq!(attack_cost(&sword(), 100, 100, None), 4200 * 85 / 100);
    assert_eq!(attack_cost(&sword(), 100, 0, Some(100)), 4200 * 80 / 100);
    assert_eq!(attack_cost(&sword(), 100, 0, Some(0)), 4200);
    assert_eq!(skill_percent(5_000), 50);
    assert_eq!(skill_percent(50_000), 100);
}

#[test]
fn a_quarter_of_an_attack_is_its_wind_up() {
    assert_eq!(wind_up(4200), 1050);
}

#[test]
fn a_median_fist_does_one_health_and_heavier_weapons_more() {
    assert_eq!(blow(&Weapon::UNARMED, 100, 1000), 1000);
    assert_eq!(blow(&sword(), 100, 1000), 2400);
    assert_eq!(blow(&sword(), 400, 1000), 4800);
    // Damage per energy is the same for any weight: weight sets the rhythm.
    let per_energy = |weapon: &Weapon| blow(weapon, 100, 1000) * 1000 / weapon_cost(weapon);
    let polearm = blade(Family::Polearm, 2600, 2200);
    assert_eq!(per_energy(&sword()), per_energy(&polearm));
}

#[test]
fn quality_counts_most_then_metal() {
    let mut artifact = sword();
    artifact.quality = Quality::Artifact;
    assert_eq!(blow(&artifact, 100, 1000), 4800);
    let mut adamantine = sword();
    adamantine.metal_tier = 4;
    assert_eq!(blow(&adamantine, 100, 1000), 3000);
}

#[test]
fn blows_land_between_half_and_half_again_mostly_near_the_middle() {
    let mut rng = Seed::new(7).rng();
    let rolls: Vec<u64> = (0..10_000).map(|_| roll_blow(&mut rng)).collect();
    assert!(rolls.iter().all(|r| (500..=1500).contains(r)));
    let middle = rolls.iter().filter(|r| (750..=1250).contains(*r)).count();
    assert!(middle > 7_000, "{middle}");
}

#[test]
fn equals_take_half_and_armour_halves_at_its_half_point() {
    assert_eq!(past_defence(1000, 50, 50), 500);
    assert_eq!(past_defence(1000, 10, 90), 100);
    assert_eq!(through_armour(1000, 0), 1000);
    assert_eq!(through_armour(1000, ARMOUR_HALF), 500);
    assert!(through_armour(1000, 10_000) > 0);
}

#[test]
fn the_off_hand_does_half_until_mastered() {
    assert_eq!(off_hand_share(0), 500);
    assert_eq!(off_hand_share(50), 750);
    assert_eq!(off_hand_share(100), 1000);
    assert_eq!(off_hand_share(500), 1000);
}

#[test]
fn a_whole_blow_between_equals() {
    let blow = Blow {
        weapon: Weapon::UNARMED,
        strength: 100,
        luck: 1000,
        hand: 1000,
        attack: 7,
        defence: 7,
        armour: 0,
    };
    assert_eq!(blow.harm(), crate::vitals::Points(500));
}

#[test]
fn power_doubles_with_either_attribute_or_skill() {
    let median = Attributes::all(100);
    let leans = [Attribute::Strength, Attribute::Agility];
    let base = power(&median, &leans, 2000);
    assert_eq!(power(&Attributes::all(200), &leans, 2000), base * 2);
    assert_eq!(power(&median, &leans, 4000), base * 2);
    // Untrained still counts for something.
    assert_eq!(power(&median, &leans, 0), power(&median, &leans, 1));
    assert!(power(&median, &leans, 0) > 0);
}

fn body(x: i64, y: i64, size: Size) -> Body {
    Body {
        at: Spot::new(x, y),
        height: 0,
        size,
    }
}

#[test]
fn weapons_reach_by_their_length_but_not_through_walls() {
    let level = Level::test_room();
    let me = body(0, 0, Size::Medium);
    let dagger = blade(Family::Dagger, 300, 400).reach();
    let spear = blade(Family::Spear, 1500, 2000).reach();
    assert_eq!(
        (Weapon::UNARMED.reach(), dagger, sword().reach()),
        (0, 50, 125)
    );
    assert_eq!(reach_between(Size::Medium, 0, Size::Medium), 340);
    // Bare hands reach a Medium neighbour in the next cell, even diagonally...
    assert!(in_reach(&level, &me, 0, &body(240, 240, Size::Medium)));
    // ...a dagger hardly further, and a spear two cells away.
    assert!(!in_reach(&level, &me, dagger, &body(400, 0, Size::Medium)));
    assert!(in_reach(&level, &me, spear, &body(480, 0, Size::Medium)));
    assert!(!in_reach(&level, &me, spear, &body(480, 480, Size::Medium)));
    // Smaller creatures are reached less far, bigger ones further.
    assert!(!in_reach(&level, &me, 0, &body(300, 0, Size::Tiny)));
    assert!(in_reach(&level, &me, 0, &body(300, 0, Size::Large)));
    // The wall at x = -2 stands between.
    assert!(!in_reach(
        &level,
        &body(-240, 0, Size::Medium),
        spear,
        &body(-720, 0, Size::Medium)
    ));
}

#[test]
fn reach_is_measured_up_and_down_slopes_too() {
    let level = Level::test_room();
    let me = body(0, 0, Size::Medium);
    let level_with_me = body(320, 0, Size::Medium);
    assert!(in_reach(&level, &me, 0, &level_with_me));
    let above = Body {
        height: 240,
        ..level_with_me
    };
    assert_eq!(distance(&me, &above), 400);
    assert!(!in_reach(&level, &me, 0, &above));
}

#[test]
fn a_blade_item_is_a_weapon_and_other_things_are_not() {
    use crate::creature::CreatureKind;
    use crate::id::Id;
    use crate::item::{Item, ItemKind};
    use crate::item_form::{ItemOrder, make_item};
    let order = ItemOrder::plain(ItemKind::Longsword, Quality::Rare);
    let human = CreatureKind::Human;
    let mut new_id = || Id::for_test(2);
    let item = make_item(
        &order,
        human,
        Id::for_test(1),
        Seed::new(3),
        None,
        &mut new_id,
    );
    let weapon = Weapon::of(&item).expect("a longsword is a weapon");
    assert_eq!(weapon.family, Some(Family::LongSword));
    assert_eq!(weapon.quality, Quality::Rare);
    assert_eq!(weapon.grams, item.total_weight());
    assert!(weapon.length_mm > 500);
    assert!(Weapon::of(&Item::new(Id::for_test(3), ItemKind::Hat)).is_none());
}

#[test]
fn a_fighter_swings_and_practises_its_own_skills() {
    use crate::creature::CreatureKind;
    use crate::skill::{SkillId, Skills};
    let attributes = Attributes::all(100);
    let skills = Skills::default();
    let unarmed = Fighter {
        kind: CreatureKind::Human,
        attributes: &attributes,
        skills: Some(&skills),
        wielded: Wielded {
            main: Weapon::UNARMED,
            off: None,
        },
    };
    let swing = unarmed.swing(0);
    assert_eq!(swing.cost, UNARMED_COST);
    assert_eq!(swing.wind_up, wind_up(UNARMED_COST));
    assert_eq!(
        unarmed.practised_by(&swing),
        vec![SkillId::find("combat/melee/unarmed/human").unwrap()]
    );
    assert_eq!(
        unarmed.practised_defending(),
        SkillId::find("combat/melee/defence/unarmed")
    );
    // Two hands: turns, the off hand with dual wielding practised too.
    let dual = Fighter {
        wielded: Wielded {
            main: sword(),
            off: Some(blade(Family::Dagger, 300, 350)),
        },
        ..unarmed
    };
    let (first, second) = (dual.swing(0), dual.swing(1));
    assert!(!first.off_hand && second.off_hand);
    assert!(second.cost < first.cost);
    let practised = dual.practised_by(&second);
    assert!(practised.contains(&SkillId::find("combat/melee/dual").unwrap()));
    assert!(practised.contains(&SkillId::find("combat/melee/dagger").unwrap()));
}

#[test]
fn every_blade_finds_a_skill_to_practise() {
    use crate::creature::CreatureKind;
    for family in Family::ALL {
        let weapon = blade(family, 1000, 1000);
        assert!(
            weapon.attack_skill(CreatureKind::Human).is_some(),
            "{family:?}"
        );
        assert!(weapon.defence_skill().is_some(), "{family:?}");
    }
    for kind in CreatureKind::ALL {
        assert!(Weapon::UNARMED.attack_skill(kind).is_some(), "{kind:?}");
    }
}
