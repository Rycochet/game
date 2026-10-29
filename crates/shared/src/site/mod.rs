//! Sites: the maps that world map features lead to. A site is one or more
//! levels (the floors of a building, the depths of a cave), each a
//! `dungeon::Level` with stairs between them.
//!
//! Which generator makes a site, and its options, comes from the feature's
//! kind (`Generator::for_feature`); everything else comes from the feature's
//! map seed. Each level is generated on its own from the seed, so the client
//! and the server agree on it whichever levels were visited first.

mod building;
mod cave;
mod grid;

use serde::{Deserialize, Serialize};

use crate::dungeon::Level;
use crate::feature::FeatureKind;
use crate::rng::Seed;

pub use building::{BuildingOptions, Outline};
pub use cave::{CaveEntrance, CaveOptions};

/// Where a way out of a level leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Exit {
    /// Back out to the world map, at the feature.
    WorldMap,
    /// Another level of the same site, by index.
    Level(u32),
}

/// Which way a staircase goes, for its picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StairsDirection {
    Up,
    Down,
}

/// A staircase on a walkable cell. Using it takes you to `to`, where you
/// arrive on the matching staircase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stairs {
    pub x: i64,
    pub y: i64,
    pub direction: StairsDirection,
    pub to: Exit,
}

/// One generated level of a site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiteLevel {
    pub level: Level,
    pub stairs: Vec<Stairs>,
    /// Where you arrive from the world map, on the level you enter by.
    pub entrance: Option<(i64, i64)>,
}

/// Whether walking to `(x, y)` leaves for the world map: outside ground on
/// the very edge of the level (beside the edge, not just any ground).
pub fn leads_out(level: &Level, x: i64, y: i64) -> bool {
    level.cell(x, y) == crate::dungeon::Cell::Ground
        && grid::STEPS
            .iter()
            .any(|(dx, dy)| !level.contains(x + dx, y + dy))
}

impl SiteLevel {
    /// Where you arrive when you come in from `from`: the entrance from the
    /// world map, or the staircase that leads back to the level you came
    /// from.
    pub fn arrival(&self, from: Exit) -> Option<(i64, i64)> {
        self.stairs
            .iter()
            .find(|stairs| stairs.to == from)
            .map(|stairs| (stairs.x, stairs.y))
            .or(match from {
                Exit::WorldMap => self.entrance,
                Exit::Level(_) => None,
            })
    }

    /// As `arrival`, but somewhere even if nothing leads there (a jump
    /// between levels, from dev tools): the first stairs or the entrance.
    pub fn arrive(&self, from: Exit) -> (i64, i64) {
        self.arrival(from)
            .or_else(|| self.stairs.first().map(|s| (s.x, s.y)))
            .or(self.entrance)
            .unwrap_or((0, 0))
    }
}

/// How a site is generated, with the options that tune it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Generator {
    /// A building outline filled with rooms and corridors (BSP), possibly
    /// several floors high.
    Building(BuildingOptions),
    /// Irregular caves joined by winding passages, possibly several levels
    /// deep. Also used for dug dungeons such as crypts (with doors).
    Cave(CaveOptions),
}

impl Generator {
    /// The generator for each kind of world map feature.
    pub fn for_feature(kind: FeatureKind) -> Self {
        use FeatureKind::*;
        use Outline::*;
        let building = |width, height, outlines, max_floors, size_per_floor| {
            Self::Building(BuildingOptions {
                width,
                height,
                outlines,
                square: false,
                max_floors,
                size_per_floor,
            })
        };
        let cave = |size, levels, entrance, doors, water_percent, lava_percent| {
            Self::Cave(CaveOptions {
                size,
                levels,
                entrance,
                doors,
                water_percent,
                lava_percent,
            })
        };
        match kind {
            // Natural caves: walk in from the side.
            GrassyCave | EarthCave => cave((40, 70), (1, 3), CaveEntrance::Opening, false, 30, 0),
            RockCave => cave((45, 80), (1, 4), CaveEntrance::Opening, false, 25, 15),
            IceCave => cave((35, 60), (1, 3), CaveEntrance::Opening, false, 60, 0),
            // A hole among the rocks.
            Boulders => cave((25, 40), (1, 2), CaveEntrance::Hole, false, 20, 0),
            // Dug dungeons below: stairs down, doors.
            Ruins => cave((35, 60), (1, 3), CaveEntrance::Hole, true, 15, 5),
            Crypt | MossyCrypt => cave((30, 55), (1, 3), CaveEntrance::Hole, true, 10, 0),
            BloodyCrypt => cave((30, 55), (2, 4), CaveEntrance::Hole, true, 5, 30),
            Mausoleum => cave((35, 60), (2, 4), CaveEntrance::Hole, true, 5, 10),
            Grave => cave((18, 28), (1, 1), CaveEntrance::Hole, true, 0, 0),
            Barrel | Barrels => cave((20, 35), (1, 2), CaveEntrance::Hole, true, 10, 0),
            // Round, one room or a few: tents, huts, an igloo.
            ThatchedHut | StiltHut | GreenStiltHut | RedTent | BlueTent | GreenTent
            | IceShelter => building((9, 13), (9, 13), &[Circle], 1, 10),
            AdobeHouse | AdobeStore | RedRoofCottage | GreyRoofCottage | StoneCottage
            | SnowyCottage => {
                building((11, 19), (9, 15), &[Rectangle, Rectangle, CutCorners], 2, 5)
            }
            Warehouse => building((20, 32), (14, 22), &[Rectangle], 1, 10),
            ArcheryRange => building((16, 26), (10, 14), &[Rectangle, RoundCorners], 1, 10),
            // Towers are narrow but tall.
            StoneTower | WoodenTower => building((13, 19), (13, 19), &[Circle], 6, 3),
            Castle => building(
                (40, 70),
                (36, 60),
                &[Rectangle, CutCorners, RoundCorners],
                4,
                12,
            ),
            Fort => building((30, 50), (26, 44), &[Rectangle, CutCorners], 3, 12),
            StonePyramid | SandPyramid => Self::Building(BuildingOptions {
                width: (25, 45),
                height: (25, 45),
                outlines: &[Rectangle],
                square: true,
                max_floors: 5,
                size_per_floor: 6,
            }),
        }
    }
}

/// A feature's map: its plan, from which any level can be generated.
#[derive(Clone, Debug)]
pub struct Site {
    kind: FeatureKind,
    plan: Plan,
}

#[derive(Clone, Debug)]
enum Plan {
    Building(building::Plan),
    Cave(cave::Plan),
}

impl Site {
    /// The site a feature leads to, from its kind and map seed.
    pub fn new(kind: FeatureKind, seed: Seed) -> Self {
        let plan = match Generator::for_feature(kind) {
            Generator::Building(options) => Plan::Building(building::Plan::new(&options, seed)),
            Generator::Cave(options) => Plan::Cave(cave::Plan::new(&options, seed)),
        };
        Self { kind, plan }
    }

    pub fn kind(&self) -> FeatureKind {
        self.kind
    }

    /// Whether its levels are a building's floors (going up from the ground)
    /// rather than a cave's depths (going down).
    pub fn is_building(&self) -> bool {
        matches!(self.plan, Plan::Building(_))
    }

    /// How many levels it has.
    pub fn levels(&self) -> u32 {
        match &self.plan {
            Plan::Building(plan) => plan.floors(),
            Plan::Cave(plan) => plan.levels(),
        }
    }

    /// Generates level `index` (`0` is the one entered from the world map:
    /// a building's ground floor, a cave's first level).
    pub fn level(&self, index: u32) -> SiteLevel {
        assert!(index < self.levels(), "no level {index}");
        match &self.plan {
            Plan::Building(plan) => plan.level(index),
            Plan::Cave(plan) => plan.level(index),
        }
    }

    /// The way `stairs` on level `index` (`level`) go down, as a grid step
    /// from their top end to their bottom end, agreeing with the matching
    /// stairs on the level they lead to (generated again to look). Only for
    /// looks: both ends of a staircase are drawn running the same way.
    pub fn stairs_descend(&self, index: u32, level: &Level, stairs: Stairs) -> (i64, i64) {
        let other = match stairs.to {
            Exit::Level(to) => {
                let other = self.level(to);
                let back = other
                    .stairs
                    .iter()
                    .find(|s| s.to == Exit::Level(index))
                    .map(|s| (s.x, s.y));
                back.map(|cell| (other.level, cell))
            }
            Exit::WorldMap => None,
        };
        let here = Some((level, (stairs.x, stairs.y)));
        let other = other.as_ref().map(|(level, cell)| (level, *cell));
        match stairs.direction {
            StairsDirection::Down => descent_way(here, other),
            StairsDirection::Up => descent_way(other, here),
        }
    }
}

/// The four sides of a cell, as grid steps (`y` south), in the order ties
/// are settled.
const SIDES: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// The way a staircase goes down between the cell `upper` (its top, on the
/// level above) and `lower` (its bottom, on the level below), either unknown
/// (the world map). Best is walkable floor to step on at the top end of the
/// upper level and off at the bottom end of the lower one; then a wall at
/// the far end of each (stairs run up against a wall). Ties go to the first
/// of `SIDES`.
fn descent_way(
    upper: Option<(&Level, (i64, i64))>,
    lower: Option<(&Level, (i64, i64))>,
) -> (i64, i64) {
    let score = |(dx, dy): (i64, i64)| {
        let mut score = 0;
        if let Some((level, (x, y))) = upper {
            score += 2 * level.is_walkable(x - dx, y - dy) as u32;
            score += !level.is_walkable(x + dx, y + dy) as u32;
        }
        if let Some((level, (x, y))) = lower {
            score += 2 * level.is_walkable(x + dx, y + dy) as u32;
            score += !level.is_walkable(x - dx, y - dy) as u32;
        }
        score
    };
    let mut best = SIDES[0];
    for side in SIDES {
        if score(side) > score(best) {
            best = side;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::Cell;
    use grid::Reach;

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
        FeatureKind::ALL.into_iter().flat_map(move |kind| {
            (0..count).map(move |n| Site::new(kind, Seed::new(n * 7919 + 13)))
        })
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
                    assert_eq!(level.cell(stairs.x, stairs.y), Cell::Floor);
                    let Exit::Level(to) = stairs.to else {
                        continue;
                    };
                    let back = levels[to as usize]
                        .arrival(Exit::Level(index as u32))
                        .expect("a way back");
                    if site.is_building() {
                        assert_eq!(back, (stairs.x, stairs.y), "{:?}", site.kind());
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
                if index == 0 {
                    let entrances = cells()
                        .filter(|&(x, y)| {
                            level.cell(x, y) == Cell::Door
                                && grid::STEPS
                                    .iter()
                                    .any(|(dx, dy)| level.cell(x + dx, y + dy) == Cell::Ground)
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

    /// Caves: no room is a plain rectangle.
    #[test]
    fn cave_rooms_are_not_rectangles() {
        for site in every_site(4).filter(|site| !site.is_building()) {
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
        for site in every_site(2) {
            let level = site.level(0).level;
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
        let values: Vec<u64> = [
            FeatureKind::Castle,
            FeatureKind::RockCave,
            FeatureKind::Crypt,
        ]
        .into_iter()
        .map(|kind| {
            let site = Site::new(kind, seed);
            fingerprint(&site.level(site.levels() - 1)) ^ site.levels() as u64
        })
        .collect();
        assert_eq!(values, GOLDEN_LEVELS);
    }

    const GOLDEN_LEVELS: [u64; 3] = [
        16250830485420571163,
        4540973389226848185,
        16348176066830501790,
    ];
}

#[cfg(test)]
mod pictures {
    use super::*;
    use crate::dungeon::Cell;

    /// Prints levels as text: `cargo test -p shared print_sites -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn print_sites() {
        let kinds = std::env::var("SITE_KINDS").unwrap_or("20,4,29".into());
        let seed: u64 = std::env::var("SITE_SEED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        for kind in kinds
            .split(',')
            .map(|k| FeatureKind::ALL[k.parse::<usize>().unwrap()])
        {
            let site = Site::new(kind, Seed::new(seed));
            for index in 0..site.levels() {
                let site_level = site.level(index);
                let level = &site_level.level;
                println!("{kind:?} level {index} of {}", site.levels());
                let (min_x, min_y, width, height) = level.bounds();
                for y in min_y..min_y + height {
                    let row: String = (min_x..min_x + width)
                        .map(|x| {
                            if let Some(s) = site_level.stairs.iter().find(|s| (s.x, s.y) == (x, y))
                            {
                                return if s.direction == StairsDirection::Up {
                                    '<'
                                } else {
                                    '>'
                                };
                            }
                            match level.cell(x, y) {
                                Cell::Floor => match level.room(x, y) {
                                    Some(r) => (b'a' + (r.0 % 26) as u8) as char,
                                    None => '.',
                                },
                                Cell::Wall => '#',
                                Cell::Door => '+',
                                Cell::SecretDoor => 'S',
                                Cell::Ground => ',',
                                Cell::Water => '~',
                                Cell::Lava => '^',
                            }
                        })
                        .collect();
                    println!("{row}");
                }
            }
        }
    }
}
