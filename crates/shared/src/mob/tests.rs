use super::*;
use crate::item::{Inventory, Item, Made};

/// A room big enough for some mobs.
fn room() -> Level {
    let mut level = Level::new(0, 0, 42, 22);
    for y in 1..21 {
        for x in 1..41 {
            level.set(x, y, Cell::Floor);
        }
    }
    level
}

#[test]
fn the_same_seed_gives_the_same_mobs() {
    let level = room();
    let first = spawns(Seed::new(5), 0, PoiKind::Castle, &level, &[]);
    assert!(!first.is_empty());
    assert_eq!(first, spawns(Seed::new(5), 0, PoiKind::Castle, &level, &[]));
    assert_ne!(first, spawns(Seed::new(6), 0, PoiKind::Castle, &level, &[]));
    assert_ne!(first, spawns(Seed::new(5), 1, PoiKind::Castle, &level, &[]));
}

#[test]
fn mobs_stand_apart_on_walkable_cells_with_looks_that_suit_them() {
    let level = room();
    for seed in 0..20 {
        let mobs = spawns(Seed::new(seed), 0, PoiKind::Crypt, &level, &[]);
        for (index, mob) in mobs.iter().enumerate() {
            assert!(level.is_walkable(mob.position.0, mob.position.1));
            assert!(mob.appearance.suits(mob.kind, mob.gender));
            assert!(mobs[index + 1..].iter().all(|o| o.position != mob.position));
        }
    }
}

#[test]
fn nobody_starts_beside_the_stairs() {
    let level = room();
    let stairs = [(10, 10), (30, 12)];
    for seed in 0..20 {
        for mob in spawns(Seed::new(seed), 0, PoiKind::Crypt, &level, &stairs) {
            for s in stairs {
                let gap = (s.0 - mob.position.0)
                    .abs()
                    .max((s.1 - mob.position.1).abs());
                assert!(gap >= FROM_STAIRS);
            }
        }
    }
}

#[test]
fn mobs_start_with_what_their_kind_can_have_and_animals_with_nothing() {
    let (mut kobolds_armed, mut qualities) = (0, std::collections::BTreeSet::new());
    for seed in 0..300 {
        let roll = |what: i64| Seed::new(seed).at(&[PURPOSE, what]);
        let gear = roll_gear(&roll, CreatureKind::Kobold);
        kobolds_armed += gear
            .iter()
            .any(|&(kind, _)| kind == ItemKind::Boomerang || kind.blade_family().is_some())
            as u32;
        for (kind, quality) in gear {
            assert_eq!(quality.is_some(), kind.def().has_quality, "{kind:?}");
            qualities.extend(quality);
        }
        assert!(roll_gear(&roll, CreatureKind::Chicken).is_empty());
        assert!(roll_gear(&roll, CreatureKind::Horse).is_empty());
        let beggar = roll_gear(&roll, CreatureKind::Beggar);
        assert_eq!(beggar[0].0, ItemKind::Shirt);
    }
    assert!((150..270).contains(&kobolds_armed), "{kobolds_armed}");
    assert!(qualities.contains(&Quality::Poor) && qualities.contains(&Quality::Rare));
}

/// Blades are left out: they are rolled within what the mob can carry. Anything else
/// must go on it, made for it, or else in its inventory, at the size it is made.
#[test]
fn mobs_can_wear_or_carry_all_they_can_start_with() {
    for creature in CreatureKind::ALL {
        let body = creature.def();
        let (width, height) = body.inventory;
        let bare = Inventory {
            creature,
            ..Inventory::default()
        };
        let places = gear_of(creature).iter().flat_map(|(_, kinds)| kinds.iter());
        for &(kind, _) in places.filter(|(kind, _)| kind.blade_family().is_none()) {
            let item = Item {
                made: Made::for_body(kind, body.size, body.fingers(), false),
                ..Item::new(Id::for_test(1), kind)
            };
            let shape = kind.def().shape;
            let (short, long) = (shape.width.min(shape.height), shape.width.max(shape.height));
            assert!(
                bare.check_wear_new(&item).is_ok()
                    || (!kind.comes_in_sizes()
                        && short <= width.min(height)
                        && long <= width.max(height)),
                "{creature:?} can't have {kind:?}"
            );
        }
    }
}

#[test]
fn towns_have_beggars_and_farms_have_animals() {
    let level = room();
    let town = spawns(Seed::new(1), 0, PoiKind::Town, &level, &[]);
    assert!(!town.is_empty() && town.len() <= MOST_BEGGARS);
    assert!(town.iter().all(|mob| mob.kind == CreatureKind::Beggar));
    let farm = spawns(Seed::new(1), 0, PoiKind::Farm, &level, &[]);
    assert!(!farm.is_empty());
    assert!(farm.iter().all(|mob| !mob.kind.def().playable));
}

#[test]
fn a_move_is_small_enough_to_send_often() {
    let moved = crate::protocol::ServerMessage::Moves(vec![crate::protocol::Stretch {
        id: Id::for_test(u128::MAX),
        from: (100 * 240, -100 * 240),
        to: (101 * 240, -100 * 240),
        ticks: 7,
    }]);
    // Its kind and length, 17 for the id, four numbers of 3 bytes and one.
    let bytes = postcard::to_allocvec(&moved).unwrap();
    assert!(bytes.len() <= 32, "{}", bytes.len());
}

#[test]
fn a_look_reads_back_from_a_message() {
    let look = EntityLook {
        id: Id::for_test(9),
        kind: CreatureKind::Kobold,
        gender: Gender::Female,
        name: None,
        appearance: Appearance::default(),
        feeling: Feeling::Unfriendly,
        x: -2,
        y: 3,
        worn: Vec::new(),
        mark: None,
        heading: crate::facing::Heading(12_345),
    };
    let bytes = postcard::to_allocvec(&look).unwrap();
    assert_eq!(postcard::from_bytes::<EntityLook>(&bytes).unwrap(), look);
}

#[test]
fn every_place_shares_out_exactly_100() {
    for site in PoiKind::ALL {
        let total: u64 = who_lives_in(site).iter().map(|&(_, share)| share).sum();
        assert_eq!(total, 100, "{site:?}");
    }
}

#[test]
fn keener_noses_and_stronger_trails_last_longer() {
    use crate::creature::Acuity;
    // A poor nose: 30 seconds on the weakest trail, a minute on the strongest.
    assert!(can_smell(Acuity::Poor, BASE_SCENT, 0, 30 * 20 - 1));
    assert!(!can_smell(Acuity::Poor, BASE_SCENT, 0, 30 * 20));
    assert!(can_smell(Acuity::Poor, MOST_SCENT, 0, 60 * 20 - 1));
    // A keen nose follows what a poor one has lost; none smells nothing.
    assert!(can_smell(Acuity::Keen, BASE_SCENT, 0, 60 * 20));
    assert!(!can_smell(Acuity::None, MOST_SCENT, 0, 0));
    // Nothing is kept past what the keenest nose could smell on the strongest.
    assert_eq!(SCENT_KEPT_FOR, smells_for(Acuity::Keen) * 2);
}

#[test]
fn carrying_nothing_leaves_the_weakest_trail() {
    let nothing = Inventory::default();
    assert_eq!(scent_strength(&nothing), BASE_SCENT);
}

#[test]
fn mobs_genders_spread_roughly_45_45_10() {
    let level = room();
    let mut counts = [0usize; 3];
    for seed in 0..200 {
        for mob in spawns(Seed::new(seed), 0, PoiKind::Crypt, &level, &[]) {
            counts[mob.gender as usize] += 1;
        }
    }
    let total: usize = counts.iter().sum();
    assert!(total > 1000, "{total}");
    let share = |count: usize| count * 100 / total;
    assert!((38..=52).contains(&share(counts[0])), "{counts:?}");
    assert!((38..=52).contains(&share(counts[1])), "{counts:?}");
    assert!((5..=15).contains(&share(counts[2])), "{counts:?}");
}

#[test]
fn peoples_have_an_alignment_and_animals_none() {
    let level = room();
    let (mut people, mut animals) = (0, 0);
    for seed in 0..100 {
        for mob in spawns(Seed::new(seed), 0, PoiKind::Crypt, &level, &[]) {
            let leans = mob.kind.alignment_leaning();
            assert_eq!(mob.alignment.is_some(), leans.is_some(), "{:?}", mob.kind);
            if leans.is_some() {
                people += 1;
            } else {
                animals += 1;
            }
        }
    }
    assert!(people > 0 && animals > 0, "{people} {animals}");
}
