//! Dev Areas grown as biomes and round buildings.

use super::biome::typical;
use super::tests::{area, level_of, reachable};
use super::*;
use crate::climate::Biome;
use crate::dungeon::Cell;
use crate::site::Exit;

/// How many of the area's cells are this.
fn count(area: DevArea, cell: Cell) -> usize {
    let level = level_of(area);
    let size = i64::from(area.size);
    (0..size)
        .flat_map(|y| (0..size).map(move |x| (x, y)))
        .filter(|&(x, y)| level.level.cell(x, y) == cell)
        .count()
}

#[test]
fn each_biome_is_grown_from_a_place_of_that_biome() {
    for biome in Biome::ALL {
        assert_eq!(typical(biome).biome(), biome);
    }
}

#[test]
fn rainforest_grows_trees_and_desert_none() {
    let forest = area(
        64,
        DevLayout::Biome(Biome::TropicalRainforest),
        DevMobs::None,
    );
    let trees = count(forest, Cell::Tree);
    assert!(trees > 64 * 64 / 10, "{trees} trees");
    let desert = area(64, DevLayout::Biome(Biome::Desert), DevMobs::None);
    assert_eq!(count(desert, Cell::Tree), 0);
    assert_eq!(level_of(forest), level_of(forest), "the same every time");
}

#[test]
fn a_biome_area_keeps_its_stairs_clear() {
    let forest = area(
        32,
        DevLayout::Biome(Biome::TemperateRainforest),
        DevMobs::None,
    );
    let level = level_of(forest);
    let stairs = level.stairs[0];
    for dy in -1..=1 {
        for dx in -1..=1 {
            assert_eq!(level.level.cell(stairs.x + dx, stairs.y + dy), Cell::Ground);
        }
    }
}

#[test]
fn every_kind_of_site_makes_an_area_with_a_way_out() {
    for kind in PoiKind::ALL {
        for size in [16, 128] {
            let made = area(size, DevLayout::Building(kind), DevMobs::Default);
            let level = level_of(made);
            assert_eq!(level.stairs.len(), 1, "{kind:?}");
            let stairs = level.stairs[0];
            assert_eq!(stairs.to, Exit::WorldMap);
            assert!(
                level.level.is_walkable(stairs.x, stairs.y),
                "{kind:?} at {size}"
            );
            assert!(
                reachable(&level, (stairs.x, stairs.y)).len() > 1,
                "{kind:?} at {size}"
            );
            let (_, _, width, height) = level.level.bounds();
            assert_eq!((width, height), (i64::from(size) + 2, i64::from(size) + 2));
            made.spawns(&level);
        }
    }
}

#[test]
fn a_town_keeps_the_buildings_wholly_inside() {
    let town = area(128, DevLayout::Building(PoiKind::Town), DevMobs::None);
    let level = level_of(town);
    assert!(!level.buildings.is_empty());
    let (min_x, min_y, width, height) = level.level.bounds();
    for building in &level.buildings {
        for rect in &building.look.rects {
            assert!(rect.x > min_x && rect.y > min_y);
            assert!(rect.x + rect.width < min_x + width && rect.y + rect.height < min_y + height);
        }
    }
}

#[test]
fn a_castle_has_its_walls_and_its_own_people() {
    let castle = area(64, DevLayout::Building(PoiKind::Castle), DevMobs::Default);
    let level = level_of(castle);
    let (min_x, min_y, width, height) = level.level.bounds();
    let floors = (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .filter(|&(x, y)| level.level.cell(x, y) == Cell::Floor)
        .count();
    assert!(floors > 100, "{floors} floor cells");
    assert!(!castle.spawns(&level).is_empty());
    assert_eq!(castle.kind(), PoiKind::Castle);
}

#[test]
fn woods_have_their_own_creatures_by_default() {
    let found = (0..20).any(|seed| {
        let forest = DevArea {
            seed,
            ..area(128, DevLayout::Biome(Biome::BorealForest), DevMobs::Default)
        };
        !forest.spawns(&level_of(forest)).is_empty()
    });
    assert!(found);
}
