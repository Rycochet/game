use super::*;
use crate::climate::Biome;
use crate::dungeon::Cell;
use crate::site::{Exit, Generator, Site, SiteLevel};
use std::collections::BTreeSet;

pub(super) fn area(size: u16, layout: DevLayout, mobs: DevMobs) -> DevArea {
    DevArea {
        size,
        layout,
        mobs,
        seed: 7,
    }
}

pub(super) fn level_of(area: DevArea) -> SiteLevel {
    let generator = Generator::DevArea(area);
    let site = Site::with_generator(
        DEV_AREA_KIND,
        Seed::new(area.seed),
        generator.version(),
        generator,
    )
    .expect("the current version runs");
    assert_eq!(site.levels(), 1);
    site.level(0)
}

/// Every cell that can be walked to from `from`, a step at a time (no corners).
pub(super) fn reachable(level: &SiteLevel, from: (i64, i64)) -> BTreeSet<(i64, i64)> {
    let mut seen = BTreeSet::from([from]);
    let mut todo = vec![from];
    while let Some((x, y)) = todo.pop() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next = (x + dx, y + dy);
            if level.level.is_walkable(next.0, next.1) && seen.insert(next) {
                todo.push(next);
            }
        }
    }
    seen
}

#[test]
fn checked_keeps_to_what_can_be_made() {
    let mobs = DevMobs::Kind {
        kind: CreatureKind::Wolf,
        count: 0,
    };
    let small = area(0, DevLayout::Empty, mobs).checked();
    assert_eq!(small.size, 16);
    assert_eq!(
        small.mobs,
        DevMobs::Kind {
            kind: CreatureKind::Wolf,
            count: 1
        }
    );
    assert_eq!(area(20, DevLayout::Empty, mobs).checked().size, 32);
    let many = DevMobs::Kind {
        kind: CreatureKind::Wolf,
        count: 200,
    };
    let big = area(1000, DevLayout::Empty, many).checked();
    assert_eq!(big.size, 128);
    assert_eq!(
        big.mobs,
        DevMobs::Kind {
            kind: CreatureKind::Wolf,
            count: MOST_DEV_MOBS
        }
    );
}

#[test]
fn seeds_from_numbers_and_words() {
    assert_eq!(DevArea::seed_from_text(" 42 "), 42);
    let word = DevArea::seed_from_text("test1");
    assert_eq!(word, DevArea::seed_from_text("test1"));
    assert_ne!(word, DevArea::seed_from_text("test2"));
}

#[test]
fn layouts_are_named() {
    let all = DevLayout::all();
    assert_eq!(all.len(), 3 + 9 + PoiKind::ALL.len());
    for layout in all {
        let name = layout.name();
        assert_eq!(
            DevLayout::named(&name.to_uppercase()),
            Some(layout),
            "{name}"
        );
    }
    assert_eq!(
        DevLayout::named("boreal_forest"),
        Some(DevLayout::Biome(Biome::BorealForest))
    );
    assert_eq!(
        DevLayout::named("Stone Tower"),
        Some(DevLayout::Building(PoiKind::StoneTower))
    );
}

#[test]
fn an_empty_area_is_open_ground_walled_round() {
    let level = level_of(area(16, DevLayout::Empty, DevMobs::Default));
    assert_eq!(level.level.bounds(), (-1, -1, 18, 18));
    for y in -1..=16 {
        for x in -1..=16 {
            let inside = (0..16).contains(&x) && (0..16).contains(&y);
            let want = if inside { Cell::Ground } else { Cell::Wall };
            assert_eq!(level.level.cell(x, y), want, "at {x}, {y}");
        }
    }
    let stairs = level.stairs[0];
    assert_eq!(stairs.to, Exit::WorldMap);
    assert_eq!(level.arrive(Exit::WorldMap), (stairs.x, stairs.y));
    assert_eq!(reachable(&level, (stairs.x, stairs.y)).len(), 16 * 16);
}

#[test]
fn moat_and_cover_split_the_middle_but_leave_a_way_round() {
    for (layout, cell) in [
        (DevLayout::Moat, Cell::Water),
        (DevLayout::Cover, Cell::Wall),
    ] {
        let level = level_of(area(32, layout, DevMobs::Default));
        for y in 4..28 {
            assert_eq!(level.level.cell(15, y), cell, "{layout:?} at {y}");
            assert_eq!(level.level.cell(16, y), cell, "{layout:?} at {y}");
        }
        assert_eq!(level.level.cell(15, 3), Cell::Ground);
        assert_eq!(level.level.cell(16, 28), Cell::Ground);
        let from = level.arrive(Exit::WorldMap);
        assert_eq!(
            reachable(&level, from).len(),
            32 * 32 - 2 * 24,
            "{layout:?}"
        );
    }
}

#[test]
fn mobs_of_a_kind_are_put_away_from_the_stairs() {
    let mobs = DevMobs::Kind {
        kind: CreatureKind::Goblin,
        count: 10,
    };
    let area = area(32, DevLayout::Cover, mobs);
    let level = level_of(area);
    let spawns = area.spawns(&level);
    assert_eq!(spawns.len(), 10);
    assert_eq!(spawns, area.spawns(&level), "the same every time");
    let stairs = level.stairs[0];
    let places: BTreeSet<_> = spawns.iter().map(|spawn| spawn.position).collect();
    assert_eq!(places.len(), 10, "one to a cell");
    for spawn in &spawns {
        assert_eq!(spawn.kind, CreatureKind::Goblin);
        let (x, y) = spawn.position;
        assert!(level.level.is_walkable(x, y));
        assert!((x - stairs.x).abs().max((y - stairs.y).abs()) >= 6);
    }
}

#[test]
fn default_and_none_put_nobody_in_plain_layouts() {
    for mobs in [DevMobs::Default, DevMobs::None] {
        let area = area(16, DevLayout::Empty, mobs);
        assert!(area.spawns(&level_of(area)).is_empty());
    }
}
