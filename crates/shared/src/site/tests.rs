use super::*;
use crate::dungeon::Cell;
use grid::Reach;

#[test]
fn only_sturdy_buildings_can_be_ruined() {
    assert!(Generator::can_be_ruined(PoiKind::Castle));
    assert!(!Generator::can_be_ruined(PoiKind::RockCave));
    assert!(!Generator::can_be_ruined(PoiKind::RedTent));
    assert!(!Generator::can_be_ruined(PoiKind::IceShelter));
}

#[test]
fn only_runnable_versions_make_sites() {
    let seed = Seed::new(3);
    assert!(Site::with_version(PoiKind::Castle, seed, BUILDING_VERSION).is_some());
    assert!(Site::with_version(PoiKind::Castle, seed, BUILDING_VERSION + 1).is_none());
    assert!(Site::with_version(PoiKind::RockCave, seed, CAVE_VERSION).is_some());
    assert!(Site::with_version(PoiKind::RockCave, seed, 0).is_none());
}

#[test]
fn only_a_ruined_generator_makes_a_ruin() {
    let seed = Seed::new(3);
    let ruined = Generator::for_poi(PoiKind::Castle).with_ruin(5);
    let new = Site::with_generator(PoiKind::Castle, seed, BUILDING_VERSION, ruined).unwrap();
    let plain = Site::with_version(PoiKind::Castle, seed, BUILDING_VERSION).unwrap();
    assert_ne!(new.level(0), plain.level(0));
    assert!(new.is_ruined() && !plain.is_ruined());
}

/// Skylights: only on the top level of natural caves, walkable, in the
/// middle of the cave, and they change nothing else.
#[test]
fn skylights_open_a_few_floor_cells_of_natural_caves() {
    let mut lights = 0;
    for n in 0..150 {
        let seed = Seed::new(n);
        for kind in [PoiKind::RockCave, PoiKind::GrassyCave, PoiKind::Boulders] {
            let Generator::Cave(options) = Generator::for_poi(kind) else {
                panic!("{kind:?} is a cave");
            };
            let (new, old) = (Site::new(kind, seed), cave::Plan::new(&options, seed));
            assert_eq!(new.levels(), old.levels());
            for index in 0..new.levels() {
                let (now, then) = (new.level(index), old.without_skylights(index));
                let (min_x, min_y, width, height) = now.level.bounds();
                assert_eq!(now.stairs, then.stairs);
                for y in min_y..min_y + height {
                    for x in min_x..min_x + width {
                        let cell = now.level.cell(x, y);
                        if cell == then.level.cell(x, y) {
                            continue;
                        }
                        lights += 1;
                        assert_eq!((index, cell), (0, Cell::Ground));
                        assert_eq!(then.level.cell(x, y), Cell::Floor);
                        assert_eq!(now.level.room(x, y), None);
                        assert!(!leads_out(&now.level, x, y));
                        for (dx, dy) in (-2..=2).flat_map(|d| (-2..=2).map(move |e| (d, e))) {
                            let near = now.level.cell(x + dx, y + dy);
                            assert!(!near.is_liquid());
                            assert!(!now.stairs.iter().any(|s| (s.x, s.y) == (x + dx, y + dy)));
                        }
                    }
                }
            }
        }
    }
    // About one per 1500 floor cells, in a few hundred caves.
    assert!((10..400).contains(&lights), "{lights}");
    // Dug dungeons have none.
    for n in 0..100 {
        let level = Site::new(PoiKind::Crypt, Seed::new(n)).level(0).level;
        let (min_x, min_y, width, height) = level.bounds();
        assert!(
            (min_y..min_y + height)
                .all(|y| { (min_x..min_x + width).all(|x| level.cell(x, y) != Cell::Ground) })
        );
    }
}

/// Stilt huts: the first floor's outline over covered ground, lone pillars
/// and one ladder. Ruined, ground is covered only under what is left above.
#[test]
fn stilt_huts_stand_on_pillars_over_covered_ground() {
    for seed in 0..100 {
        for ruin in 0..=5 {
            let kind = PoiKind::StiltHut;
            let generator = Generator::for_poi(kind).with_ruin(ruin);
            let site = Site::with_generator(kind, Seed::new(seed), generator.version(), generator)
                .expect("the current version runs");
            assert_eq!(site.levels(), 2);
            let (ground, above) = (site.level(0), site.level(1));
            assert_eq!(ground.stairs.len(), 1);
            assert!(leads_out(&ground.level, 0, 0));
            let (min_x, min_y, width, height) = ground.level.bounds();
            let cells =
                (min_y..min_y + height).flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)));
            let mut pillars = 0;
            for (x, y) in cells {
                let up = above.level.cell(x, y);
                match ground.level.cell(x, y) {
                    Cell::CoveredGround => {
                        assert!(up != Cell::Hole, "open above ({x}, {y})");
                    }
                    Cell::Wall => {
                        pillars += 1;
                        for (dx, dy) in (-1..=1).flat_map(|d| (-1..=1).map(move |e| (d, e))) {
                            assert!(
                                (dx, dy) == (0, 0)
                                    || ground.level.cell(x + dx, y + dy) != Cell::Wall
                            );
                        }
                    }
                    Cell::Ground if ruin == 0 => {
                        assert_eq!(up, Cell::Wall, "floor over open ground");
                    }
                    cell => assert_eq!(cell, Cell::Ground),
                }
            }
            assert!(ruin > 0 || pillars >= 4, "{pillars} pillars");
        }
    }
}

/// A stilt hut saved before they had stilts stays as it was.
#[test]
fn saved_stilt_huts_keep_their_floors() {
    let seed = Seed::new(3);
    let old = Site::with_version(PoiKind::StiltHut, seed, BUILDING_VERSION).unwrap();
    assert_eq!(old.levels(), 1);
    assert_eq!(old.level(0), Site::new(PoiKind::ThatchedHut, seed).level(0));
    assert_eq!(Site::new(PoiKind::StiltHut, seed).levels(), 2);
}

/// Both ends of a staircase are built the same, as something that suits
/// the place, and sites differ.
#[test]
fn stairs_are_the_same_style_at_both_ends() {
    let mut seen = std::collections::HashSet::new();
    for seed in 0..30 {
        for kind in PoiKind::ALL {
            let site = Site::new(kind, Seed::new(seed));
            for index in 0..site.levels() {
                for stairs in site.level(index).stairs {
                    let style = site.stairs_style(index, stairs);
                    assert!(StairsStyle::choices(kind).contains(&style));
                    seen.insert(style);
                    if let Exit::Level(to) = stairs.to {
                        let back = site.level(to).stairs;
                        let back = back.iter().find(|s| s.to == Exit::Level(index));
                        assert_eq!(site.stairs_style(to, *back.unwrap()), style);
                    }
                }
            }
        }
    }
    assert_eq!(seen.len(), StairsStyle::ALL.len());
}

#[test]
fn stairs_go_down_the_same_way_on_both_levels() {
    let level = Level::test_room();
    // Down stairs alone, against the north wall, open to the south:
    // you step on from the south, so they go down northwards.
    assert_eq!(descent_way(Some((&level, (0, -3))), None), (0, -1));
    // Up stairs alone against the east wall climb east, so they go down
    // westwards.
    assert_eq!(descent_way(None, Some((&level, (7, 0)))), (-1, 0));
    // Together, both levels count: there is floor to step on at the top
    // and off at the bottom.
    let (dx, dy) = descent_way(Some((&level, (0, -3))), Some((&level, (7, 0))));
    assert!(level.is_walkable(-dx, -3 - dy));
    assert!(level.is_walkable(7 + dx, dy));
}

/// A number summing up a level (cells, rooms, stairs), for golden tests.
fn fingerprint(site_level: &SiteLevel) -> u64 {
    let level = &site_level.level;
    let (min_x, min_y, width, height) = level.bounds();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut add = |value: u64| {
        hash = (hash ^ value).wrapping_mul(0x0000_0100_0000_01b3);
    };
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            add(level.cell(x, y) as u64);
            add(level.room(x, y).map_or(u64::MAX, |r| r.0 as u64));
        }
    }
    for stairs in &site_level.stairs {
        add(stairs.x as u64);
        add(stairs.y as u64);
    }
    hash
}

fn every_site(count: u64) -> impl Iterator<Item = Site> {
    PoiKind::ALL
        .into_iter()
        .flat_map(move |kind| (0..count).map(move |n| Site::new(kind, Seed::new(n * 7919 + 13))))
}

/// Every walkable cell can be reached from wherever you arrive, and
/// every staircase leads to a level with a staircase back on the same
/// spot (buildings) or at least one back (caves).
#[test]
fn every_level_is_connected_and_stairs_match() {
    for site in every_site(6) {
        let levels: Vec<SiteLevel> = (0..site.levels()).map(|i| site.level(i)).collect();
        for (index, site_level) in levels.iter().enumerate() {
            let level = &site_level.level;
            let start = if index == 0 {
                site_level.arrival(Exit::WorldMap)
            } else if site.has_basement() && index + 1 == levels.len() {
                site_level.arrival(Exit::Level(0))
            } else {
                site_level.arrival(Exit::Level(index as u32 - 1))
            }
            .unwrap_or_else(|| panic!("{:?} level {index}: no arrival", site.kind()));
            let mut reach = Reach::new(level);
            reach.spread(level, &[start]);
            let (min_x, min_y, width, height) = level.bounds();
            for y in min_y..min_y + height {
                for x in min_x..min_x + width {
                    assert!(
                        !level.is_walkable(x, y) || reach.is(x, y),
                        "{:?} level {index}: ({x}, {y}) can't be reached",
                        site.kind()
                    );
                }
            }
            if index == 0 {
                let ground = (min_y..min_y + height)
                    .any(|y| (min_x..min_x + width).any(|x| leads_out(level, x, y)));
                let stairs_out = site_level.stairs.iter().any(|s| s.to == Exit::WorldMap);
                assert!(ground || stairs_out, "{:?}: no way out", site.kind());
            }
            for stairs in &site_level.stairs {
                assert!(matches!(
                    level.cell(stairs.x, stairs.y),
                    Cell::Floor | Cell::CoveredGround
                ));
                let Exit::Level(to) = stairs.to else {
                    continue;
                };
                let back = levels[to as usize]
                    .arrival(Exit::Level(index as u32))
                    .expect("a way back");
                if site.is_building() {
                    let there = &levels[to as usize].stairs;
                    let spot = (stairs.x, stairs.y);
                    assert!(
                        there
                            .iter()
                            .any(|s| (s.x, s.y) == spot && s.to == Exit::Level(index as u32)),
                        "{:?}",
                        site.kind()
                    );
                } else {
                    assert!(levels[to as usize].level.is_walkable(back.0, back.1));
                }
            }
        }
    }
}

/// Buildings: at least one door from the outside on the ground floor,
/// and no wall inside the outline thicker than 3 cells.
#[test]
fn buildings_have_doors_and_thin_walls() {
    for site in every_site(6).filter(Site::is_building) {
        for index in 0..site.levels() {
            let site_level = site.level(index);
            let level = &site_level.level;
            let (min_x, min_y, width, height) = level.bounds();
            let cells = || {
                (min_y..min_y + height)
                    .flat_map(move |y| (min_x..min_x + width).map(move |x| (x, y)))
            };
            if index == 0 && !matches!(site.kind(), PoiKind::StiltHut | PoiKind::GreenStiltHut) {
                let entrances = cells()
                    .filter(|&(x, y)| {
                        level.cell(x, y) == Cell::Door
                            && grid::STEPS
                                .iter()
                                .any(|(dx, dy)| level.cell(x + dx, y + dy).is_outside())
                    })
                    .count();
                assert!(entrances >= 1, "{:?}: no way in", site.kind());
            }
            // Inside the building: the cells with walkable cells
            // somewhere on both sides along a row or column.
            for (x, y) in cells() {
                if level.cell(x, y) != Cell::Wall {
                    continue;
                }
                let inside = [(1, 0), (0, 1)].iter().any(|&(dx, dy)| {
                    let found = |sign: i64| {
                        (1..width.max(height))
                            .any(|d| level.is_walkable(x + dx * d * sign, y + dy * d * sign))
                    };
                    found(1) && found(-1)
                });
                if !inside {
                    continue;
                }
                let near_open = (-2..=2i64).any(|dy| {
                    (-2..=2i64).any(|dx| {
                        let cell = level.cell(x + dx, y + dy);
                        cell.is_walkable() || cell == Cell::SecretDoor
                    })
                });
                assert!(
                    near_open,
                    "{:?} floor {index}: thick wall at ({x}, {y})",
                    site.kind()
                );
            }
        }
    }
}

/// Windows are only in outside walls, with wall on either side of them, the
/// inside on one side and nothing of the building's at a diagonal; basements
/// have none, and forts and the like none on the ground floor.
#[test]
fn windows_are_set_in_outside_walls() {
    let windows = |level: &Level| {
        let (min_x, min_y, width, height) = level.bounds();
        (min_y..min_y + height)
            .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
            .filter(|&(x, y)| level.cell(x, y) == Cell::Window)
            .collect::<Vec<_>>()
    };
    let mut found = 0;
    for ruin in [0, 3, 5] {
        for n in 0..40 {
            for kind in [PoiKind::RedRoofCottage, PoiKind::Warehouse, PoiKind::Farm] {
                let generator = Generator::for_poi(kind).with_ruin(ruin);
                let site = Site::with_generator(kind, Seed::new(n), generator.version(), generator)
                    .expect("the current version runs");
                let storeys = site.levels() - u32::from(site.has_basement());
                for index in 0..site.levels() {
                    let level = site.level(index).level;
                    let spots = windows(&level);
                    if index >= storeys {
                        assert!(spots.is_empty());
                    }
                    found += spots.len();
                    for (x, y) in spots {
                        let straight = [(1, 0), (0, 1)].iter().any(|&(dx, dy)| {
                            level.cell(x + dx, y + dy) == Cell::Wall
                                && level.cell(x - dx, y - dy) == Cell::Wall
                                && (level.is_walkable(x + dy, y + dx)
                                    || level.is_walkable(x - dy, y - dx))
                        });
                        assert!(straight, "{kind:?} floor {index}: ({x}, {y})");
                        if index == 0 {
                            for (dx, dy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                                assert!(!level.cell(x + dx, y + dy).is_wall());
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(found > 100, "{found}");
    let mut above = 0;
    for kind in [
        PoiKind::Castle,
        PoiKind::Fort,
        PoiKind::StoneTower,
        PoiKind::WoodenTower,
    ] {
        for n in 0..5 {
            let site = Site::new(kind, Seed::new(n));
            assert!(windows(&site.level(0).level).is_empty());
            above += (1..site.levels())
                .map(|i| windows(&site.level(i).level).len())
                .sum::<usize>();
        }
    }
    assert!(above > 0);
}

#[test]
fn pyramids_never_have_windows() {
    for kind in [PoiKind::StonePyramid, PoiKind::SandPyramid] {
        for n in 0..10 {
            let site = Site::new(kind, Seed::new(n));
            for index in 0..site.levels() {
                let level = site.level(index).level;
                let (min_x, min_y, width, height) = level.bounds();
                assert!(
                    (min_y..min_y + height).all(|y| {
                        (min_x..min_x + width).all(|x| level.cell(x, y) != Cell::Window)
                    })
                );
            }
        }
    }
}

/// Caves: no room is a plain rectangle.
#[test]
fn cave_rooms_are_not_rectangles() {
    for site in every_site(4).filter(|site| !site.is_building() && !site.kind().is_town()) {
        for index in 0..site.levels() {
            let level = site.level(index).level;
            let (min_x, min_y, width, height) = level.bounds();
            let mut boxes: std::collections::BTreeMap<u16, (i64, i64, i64, i64, i64)> =
                Default::default();
            for y in min_y..min_y + height {
                for x in min_x..min_x + width {
                    if let Some(room) = level.room(x, y) {
                        let b = boxes.entry(room.0).or_insert((x, y, x, y, 0));
                        *b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y), b.4 + 1);
                    }
                }
            }
            for (room, (x0, y0, x1, y1, count)) in boxes {
                assert!(
                    count < (x1 - x0 + 1) * (y1 - y0 + 1),
                    "{:?} level {index}: room {room} is a rectangle",
                    site.kind()
                );
            }
        }
    }
}

/// Rooms are only on floor-like cells, and some exist.
#[test]
fn rooms_are_marked() {
    // (In towns, rooms are buildings: tested in `town`.)
    for site in every_site(2).filter(|site| !site.kind().is_town()) {
        // (A stilt hut's rooms are upstairs.)
        let on_stilts = matches!(site.kind(), PoiKind::StiltHut | PoiKind::GreenStiltHut);
        let level = site.level(u32::from(on_stilts)).level;
        let (min_x, min_y, width, height) = level.bounds();
        let mut any = false;
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if level.room(x, y).is_some() {
                    any = true;
                    assert!(
                        matches!(level.cell(x, y), Cell::Floor | Cell::Water | Cell::Lava),
                        "{:?}: room on {:?}",
                        site.kind(),
                        level.cell(x, y)
                    );
                }
            }
        }
        assert!(any, "{:?}: no rooms", site.kind());
    }
}

/// Golden test: the same seed always makes the same levels. If this
/// changes, every site in every world changes.
#[test]
fn golden_levels() {
    let seed = Seed::new(12345);
    let values: Vec<u64> = [PoiKind::Castle, PoiKind::RockCave, PoiKind::Crypt]
        .into_iter()
        .map(|kind| {
            let site = Site::new(kind, seed);
            fingerprint(&site.level(site.levels() - 1)) ^ site.levels() as u64
        })
        .collect();
    assert_eq!(values, GOLDEN_LEVELS);
}

const GOLDEN_LEVELS: [u64; 3] = [
    18086710199301873350,
    4540973389226848185,
    16348176066830501790,
];
