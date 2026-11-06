use super::*;
use crate::dungeon::Level;

/// A block of open ground.
fn open_block() -> Level {
    let mut level = Level::new(0, 0, 64, 64);
    for y in 0..64 {
        for x in 0..64 {
            level.set(x, y, Cell::Grass);
        }
    }
    level
}

#[test]
fn blocks_have_packs_or_herds_by_biome_and_the_same_every_time() {
    let seed = Seed::new(7);
    let level = open_block();
    let blocks = (0..60).map(|n| BlockPos { x: n, y: 3 });
    let mut lived = 0;
    for block in blocks {
        let wolves = wild_spawns(seed, block, &level, Biome::BorealForest);
        assert_eq!(
            wolves,
            wild_spawns(seed, block, &level, Biome::BorealForest)
        );
        assert!(wolves.len() <= MOST_TOGETHER as usize);
        assert!(wolves.iter().all(|mob| mob.kind == CreatureKind::Wolf));
        assert!(wild_spawns(seed, block, &level, Biome::Desert).is_empty());
        lived += usize::from(!wolves.is_empty());
    }
    assert!((20..=45).contains(&lived), "{lived} of 60 blocks");
}

#[test]
fn wild_ids_are_the_same_for_the_same_number_only() {
    let seed = Seed::new(7);
    assert_eq!(wild_mob_id(seed, 5), wild_mob_id(seed, 5));
    assert_ne!(wild_mob_id(seed, 5), wild_mob_id(seed, 6));
    assert_ne!(wild_mob_id(seed, 5), wild_mob_id(Seed::new(8), 5));
}
