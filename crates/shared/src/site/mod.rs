//! Sites: the maps that world map pois lead to. A site is one or more
//! levels (the floors of a building, the depths of a cave), each a
//! `dungeon::Level` with stairs between them.
//!
//! Which generator makes a site, and its options, comes from the poi's
//! kind (`Generator::for_poi`); everything else comes from the poi's
//! map seed. Each level is generated on its own from the seed, so the client
//! and the server agree on it whichever levels were visited first.
//!
//! Each generator has a version (`Generator::version`), bumped whenever what
//! it makes changes in a way that matters. A saved site keeps the version it
//! was first made with, so every version still in use must stay runnable
//! (`Site::with_version`).

mod building;
mod cave;
mod farm;
mod grid;
mod herbalist;
mod ruin;
mod town;

use serde::{Deserialize, Serialize};

use crate::dungeon::Level;
use crate::poi::PoiKind;
use crate::rng::Seed;

pub use building::{BuildingOptions, Outline};
pub use cave::{CaveEntrance, CaveOptions};
pub use farm::{FarmOptions, FieldLook};
pub use herbalist::HerbalistOptions;
pub use town::{BuildingKind, BuildingLook, Rect, RoofKind, TownBuilding, TownOptions, WallKind};

/// What a site looks like from outside, for choosing a poi's model. Worked
/// out without generating any level, and holding nothing a look from outside
/// wouldn't show, so the server can send it to whoever knows of the poi.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoiLook {
    Building {
        outline: Outline,
        /// Its ground floor, in cells.
        width: u16,
        height: u16,
        floors: u8,
        /// How ruined it is (`BuildingOptions::ruin`).
        ruin: u8,
    },
    Cave {
        entrance: CaveEntrance,
    },
    Town {
        buildings: u16,
    },
    Farm {
        /// The farmhouse's ground floor, in cells.
        width: u16,
        height: u16,
        ruin: u8,
        fields: u8,
    },
}

impl PoiLook {
    /// Whether it is a ruin, and is named as one.
    pub fn is_ruined(self) -> bool {
        matches!(self, Self::Building { ruin, .. } | Self::Farm { ruin, .. } if ruin > 0)
    }

    /// The look of the site a poi of this kind and map seed leads to.
    pub fn of(kind: PoiKind, seed: Seed) -> Self {
        match Generator::for_site(kind, seed) {
            Generator::Building(options) => {
                let plan = building::Plan::new(&options, seed);
                let (width, height) = plan.footprint();
                Self::Building {
                    outline: plan.outline(),
                    width: width as u16,
                    height: height as u16,
                    floors: plan.floors() as u8,
                    ruin: options.ruin,
                }
            }
            Generator::Cave(options) => Self::Cave {
                entrance: options.entrance,
            },
            Generator::Town(options) => Self::Town {
                buildings: town::Plan::new(&options, seed).target() as u16,
            },
            Generator::Herbalist(options) => {
                let plan = building::Plan::new(&options.house, seed);
                let (width, height) = plan.footprint();
                Self::Building {
                    outline: plan.outline(),
                    width: width as u16,
                    height: height as u16,
                    floors: 1,
                    ruin: options.house.ruin,
                }
            }
            Generator::Farm(options) => {
                let plan = farm::Plan::new(&options, seed);
                let (width, height) = plan.footprint();
                Self::Farm {
                    width: width as u16,
                    height: height as u16,
                    ruin: options.house.ruin,
                    fields: plan.fields() as u8,
                }
            }
        }
    }
}

/// Development tool: bump this to start clean. Every server forgets all it has
/// saved from before (characters and what they know, sites and players' changes
/// to them), so anyone connecting starts again, as nothing is kept on a client.
/// Every generator version goes up with it, so sites are made as new whatever
/// else has changed. The world itself stays: it comes from its seed.
pub const FORGET_BEFORE: u32 = 2;

/// Of each 100 buildings that can be ruined, how many are, evenly from 1 to
/// `MOST_RUIN`.
const RUINED_PERCENT: u64 = 30;
const MOST_RUIN: u64 = 5;

/// Of the places in outside walls a window could go, how many have one.
const WINDOWS_PERCENT: u32 = 30;

/// Where a way out of a level leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Exit {
    /// Back out to the world map, at the poi.
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

/// What a way between two levels is built as. Only for looks: all are used
/// the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StairsStyle {
    /// Winding round a central column.
    Spiral,
    /// Two flights side by side, turning back half way up.
    Turning,
    /// A wooden ladder leaning from the middle of the bottom.
    Ladder,
    /// A thick pole, leaning like a ladder, with rungs out of alternate sides.
    LogLadder,
    /// Hanging straight down, with sagging rope for rungs.
    RopeLadder,
    /// Hanging straight down, with wooden slats for rungs.
    SlatLadder,
    /// A single hanging rope with knots.
    KnottedRope,
}

impl StairsStyle {
    pub const ALL: [Self; 7] = [
        Self::Spiral,
        Self::Turning,
        Self::Ladder,
        Self::LogLadder,
        Self::RopeLadder,
        Self::SlatLadder,
        Self::KnottedRope,
    ];

    /// The next one, going round `ALL` (dev tools, to show them all).
    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|&style| style == self);
        Self::ALL[(index.unwrap_or(0) + 1) % Self::ALL.len()]
    }

    /// The styles that suit a kind of place.
    fn choices(kind: PoiKind) -> &'static [Self] {
        use PoiKind::*;
        use StairsStyle::*;
        match kind {
            Castle | Fort | StoneTower | StoneCottage | Warehouse | ArcheryRange | AdobeHouse
            | AdobeStore | StonePyramid | SandPyramid => &[Spiral, Turning],
            ThatchedHut | StiltHut | GreenStiltHut | RedTent | BlueTent | GreenTent
            | RedRoofCottage | GreyRoofCottage | SnowyCottage | WoodenTower => {
                &[Turning, Ladder, LogLadder]
            }
            Crypt | MossyCrypt | BloodyCrypt | Mausoleum | Grave | Ruins | Barrel | Barrels => {
                &[Spiral, Turning, Ladder]
            }
            GrassyCave | EarthCave | RockCave | Boulders | IceCave | IceShelter => {
                &[Ladder, LogLadder, RopeLadder, SlatLadder, KnottedRope]
            }
            Farm | HerbalistHut => &[Turning, Ladder],
            // Towns have no stairs yet.
            Hamlet | Village | Town | City => &[Turning],
        }
    }
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
    /// A town's buildings (no other level has any).
    pub buildings: Vec<TownBuilding>,
    /// A farm's fields, on its ground floor (no other level has any).
    pub fields: Vec<FieldLook>,
}

/// Whether walking to `(x, y)` leaves for the world map: outside ground or
/// road on the very edge of the level (beside the edge, not just any ground).
pub fn leads_out(level: &Level, x: i64, y: i64) -> bool {
    let cell = level.cell(x, y);
    (cell == crate::dungeon::Cell::Ground || cell.is_road())
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
    /// Roads with solid buildings along them.
    Town(TownOptions),
    /// A farmhouse in the middle of fenced fields.
    Farm(FarmOptions),
    /// A small hut over a basement, with a herb patch outside.
    Herbalist(HerbalistOptions),
}

impl Generator {
    /// Its current version (`BUILDING_VERSION`, `CAVE_VERSION`).
    pub fn version(&self) -> u32 {
        match self {
            Self::Building(_) => BUILDING_VERSION,
            Self::Cave(_) => CAVE_VERSION,
            Self::Town(_) => TOWN_VERSION,
            Self::Farm(_) => FARM_VERSION,
            Self::Herbalist(_) => HERBALIST_VERSION,
        }
    }

    /// The same generator, with a building ruined this much (0 to 5). Other
    /// generators take no notice.
    pub fn with_ruin(self, ruin: u8) -> Self {
        match self {
            Self::Building(options) => Self::Building(BuildingOptions { ruin, ..options }),
            Self::Farm(options) => Self::Farm(FarmOptions {
                house: BuildingOptions {
                    ruin,
                    ..options.house
                },
                ..options
            }),
            Self::Herbalist(options) => Self::Herbalist(HerbalistOptions {
                house: BuildingOptions {
                    ruin,
                    ..options.house
                },
                ..options
            }),
            other => other,
        }
    }

    /// How ruined it is (0 for any that can't be).
    pub fn ruin(&self) -> u8 {
        match self {
            Self::Building(options) => options.ruin,
            Self::Farm(options) => options.house.ruin,
            Self::Herbalist(options) => options.house.ruin,
            _ => 0,
        }
    }

    /// The generator of one poi's site: its kind's, ruined as its map seed says.
    pub fn for_site(kind: PoiKind, seed: Seed) -> Self {
        let sturdy = !matches!(
            kind,
            PoiKind::RedTent | PoiKind::BlueTent | PoiKind::GreenTent | PoiKind::IceShelter
        );
        if !sturdy {
            return Self::for_poi(kind);
        }
        let roll = seed.derive("how ruined").at(&[0]) % 100;
        let ruin = roll.saturating_sub(100 - RUINED_PERCENT) * MOST_RUIN / RUINED_PERCENT;
        let ruined = roll >= 100 - RUINED_PERCENT;
        Self::for_poi(kind).with_ruin(if ruined { ruin as u8 + 1 } else { 0 })
    }

    /// The generator for each kind of world map poi.
    pub fn for_poi(kind: PoiKind) -> Self {
        use Outline::*;
        use PoiKind::*;
        let ground_windows = !matches!(
            kind,
            Castle
                | Fort
                | StoneTower
                | WoodenTower
                | RedTent
                | BlueTent
                | GreenTent
                | IceShelter
                | StonePyramid
                | SandPyramid
        );
        let building = |width, height, outlines, max_floors, size_per_floor| {
            Self::Building(BuildingOptions {
                width,
                height,
                outlines,
                square: false,
                min_floors: 1,
                max_floors,
                same_size: false,
                taper: None,
                size_per_floor,
                ruin: 0,
                basement_percent: 0,
                basement_size: None,
                windows_percent: WINDOWS_PERCENT,
                ground_windows,
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
        let town = |buildings| Self::Town(TownOptions { buildings });
        let at_least_three_floors = |generator| match generator {
            Self::Building(options) => Self::Building(BuildingOptions {
                min_floors: 3,
                ..options
            }),
            other => other,
        };
        match kind {
            Hamlet => town((3, 8)),
            Village => town((9, 25)),
            Town => town((26, 60)),
            City => town((61, 150)),
            Farm => Self::Farm(FarmOptions {
                house: BuildingOptions {
                    width: (13, 19),
                    height: (13, 17),
                    outlines: &[Rectangle, Rectangle, CutCorners],
                    square: false,
                    min_floors: 2,
                    max_floors: 2,
                    same_size: true,
                    taper: None,
                    size_per_floor: 5,
                    ruin: 0,
                    basement_percent: 50,
                    basement_size: None,
                    windows_percent: WINDOWS_PERCENT,
                    ground_windows,
                },
                fields: (1, 8),
            }),
            HerbalistHut => Self::Herbalist(HerbalistOptions {
                house: BuildingOptions {
                    width: (5, 8),
                    height: (5, 8),
                    outlines: &[Rectangle],
                    square: false,
                    min_floors: 1,
                    max_floors: 1,
                    same_size: false,
                    taper: None,
                    size_per_floor: 5,
                    ruin: 0,
                    basement_percent: 100,
                    basement_size: Some(((8, 16), (8, 16))),
                    windows_percent: WINDOWS_PERCENT,
                    ground_windows,
                },
                strips: (3, 5),
                length: (7, 12),
            }),
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
            // Towers are narrow but tall, the same all the way up.
            StoneTower | WoodenTower => match building((13, 19), (13, 19), &[Circle], 8, 2) {
                Self::Building(options) => Self::Building(BuildingOptions {
                    min_floors: 5,
                    same_size: true,
                    ..options
                }),
                other => other,
            },
            // Always a keep above the walls.
            Castle => at_least_three_floors(building(
                (40, 70),
                (36, 60),
                &[Rectangle, CutCorners, RoundCorners],
                4,
                12,
            )),
            Fort => at_least_three_floors(building(
                (30, 50),
                (26, 44),
                &[Rectangle, CutCorners],
                3,
                12,
            )),
            StonePyramid | SandPyramid => Self::Building(BuildingOptions {
                width: (25, 45),
                height: (25, 45),
                outlines: &[Rectangle],
                square: true,
                min_floors: 1,
                max_floors: 1,
                same_size: false,
                // Stepping in until no room fits at the top.
                taper: Some((3, 4)),
                size_per_floor: 6,
                ruin: 0,
                basement_percent: 0,
                basement_size: None,
                windows_percent: WINDOWS_PERCENT,
                ground_windows,
            }),
        }
    }
}

/// The building generator's version: bump it when its output changes in a
/// way that matters (not for looks), and keep the old one runnable while
/// saved sites use it. Bumping `FORGET_BEFORE` ends that: no old version is left
/// in use, so all the code for them can go.
pub const BUILDING_VERSION: u32 = 1 + FORGET_BEFORE;

/// The cave generator's version (as `BUILDING_VERSION`).
pub const CAVE_VERSION: u32 = 1 + FORGET_BEFORE;

/// The town generator's version (as `BUILDING_VERSION`).
pub const TOWN_VERSION: u32 = 1 + FORGET_BEFORE;

/// The farm generator's version (as `BUILDING_VERSION`).
pub const FARM_VERSION: u32 = 1 + FORGET_BEFORE;

/// The herbalist's hut generator's version (as `BUILDING_VERSION`).
pub const HERBALIST_VERSION: u32 = 1 + FORGET_BEFORE;

/// A poi's map: its plan, from which any level can be generated.
#[derive(Clone, Debug)]
pub struct Site {
    kind: PoiKind,
    seed: Seed,
    plan: Plan,
    ruined: bool,
}

#[derive(Clone, Debug)]
enum Plan {
    Building(building::Plan),
    Cave(cave::Plan),
    Town(town::Plan),
    Farm(farm::Plan),
    Herbalist(herbalist::Plan),
}

impl Site {
    /// The site a poi leads to, from its kind and map seed, made with
    /// the current version of its generator.
    pub fn new(kind: PoiKind, seed: Seed) -> Self {
        let version = Generator::for_poi(kind).version();
        Self::with_version(kind, seed, version).expect("the current version runs")
    }

    /// The same, made with an older (or the current) version of its
    /// generator: `None` if that version can no longer be run.
    pub fn with_version(kind: PoiKind, seed: Seed, version: u32) -> Option<Self> {
        Self::with_generator(kind, seed, version, Generator::for_poi(kind))
    }

    /// The same, with options other than the kind's own (a ruined building).
    pub fn with_generator(
        kind: PoiKind,
        seed: Seed,
        version: u32,
        generator: Generator,
    ) -> Option<Self> {
        let ruined = match generator {
            Generator::Building(options) => options.ruin > 0,
            Generator::Farm(options) => options.house.ruin > 0,
            Generator::Herbalist(options) => options.house.ruin > 0,
            _ => false,
        };
        let plan = match generator {
            Generator::Building(options) if version == BUILDING_VERSION => {
                Plan::Building(building::Plan::new(&options, seed))
            }
            Generator::Cave(options) if version == CAVE_VERSION => {
                Plan::Cave(cave::Plan::new(&options, seed))
            }
            Generator::Town(options) if version == TOWN_VERSION => {
                Plan::Town(town::Plan::new(&options, seed))
            }
            Generator::Farm(options) if version == FARM_VERSION => {
                Plan::Farm(farm::Plan::new(&options, seed))
            }
            Generator::Herbalist(options) if version == HERBALIST_VERSION => {
                Plan::Herbalist(herbalist::Plan::new(&options, seed))
            }
            _ => return None,
        };
        Some(Self {
            kind,
            seed,
            plan,
            ruined,
        })
    }

    /// Whether it is a ruined building, and is named as one.
    pub fn is_ruined(&self) -> bool {
        self.ruined
    }

    pub fn kind(&self) -> PoiKind {
        self.kind
    }

    /// Whether its levels are a building's floors (going up from the ground)
    /// rather than a cave's depths (going down).
    pub fn is_building(&self) -> bool {
        matches!(
            self.plan,
            Plan::Building(_) | Plan::Farm(_) | Plan::Herbalist(_)
        )
    }

    /// Whether its last level is a basement: below the ground floor, whatever
    /// its index says.
    pub fn has_basement(&self) -> bool {
        match &self.plan {
            Plan::Building(plan) => plan.has_basement(),
            Plan::Farm(plan) => plan.has_basement(),
            Plan::Herbalist(plan) => plan.house().has_basement(),
            _ => false,
        }
    }

    /// How many levels it has.
    pub fn levels(&self) -> u32 {
        match &self.plan {
            Plan::Building(plan) => plan.floors(),
            Plan::Cave(plan) => plan.levels(),
            Plan::Town(_) => 1,
            Plan::Farm(plan) => plan.levels(),
            Plan::Herbalist(plan) => plan.house().floors(),
        }
    }

    /// Generates level `index` (`0` is the one entered from the world map:
    /// a building's ground floor, a cave's first level).
    pub fn level(&self, index: u32) -> SiteLevel {
        assert!(index < self.levels(), "no level {index}");
        match &self.plan {
            Plan::Building(plan) => plan.level(index),
            Plan::Cave(plan) => plan.level(index),
            Plan::Town(plan) => plan.level(),
            Plan::Farm(plan) => plan.level(index),
            Plan::Herbalist(plan) => plan.level(index),
        }
    }

    /// What `stairs` on level `index` are built as, the same at both ends:
    /// it depends on the site's seed and the upper of the two levels joined
    /// (the world map counts as above everything). Only for looks; it draws
    /// nothing from the generators' random sequences.
    pub fn stairs_style(&self, index: u32, stairs: Stairs) -> StairsStyle {
        let upper = match stairs.to {
            Exit::WorldMap => -1,
            Exit::Level(to) => i64::from(index.min(to)),
        };
        let choices = StairsStyle::choices(self.kind);
        let hash = self.seed.derive("stairs style").at(&[upper]);
        choices[(hash % choices.len() as u64) as usize]
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
        PoiKind::ALL.into_iter().flat_map(move |kind| {
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
                    assert_eq!(level.cell(stairs.x, stairs.y), Cell::Floor);
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
                if index == 0 {
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
                    let site =
                        Site::with_generator(kind, Seed::new(n), generator.version(), generator)
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
}

#[cfg(test)]
mod pictures {
    use super::*;
    use crate::dungeon::Cell;

    /// Prints levels as text: `cargo test -p shared print_sites -- --ignored --nocapture`
    /// (`SITE_KINDS`, `SITE_SEED`, `SITE_RUIN` choose what).
    #[test]
    #[ignore]
    fn print_sites() {
        let kinds = std::env::var("SITE_KINDS").unwrap_or("20,4,29".into());
        let seed: u64 = std::env::var("SITE_SEED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let ruin: u8 = std::env::var("SITE_RUIN")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        for kind in kinds
            .split(',')
            .map(|k| PoiKind::ALL[k.parse::<usize>().unwrap()])
        {
            let site = Site::with_generator(
                kind,
                Seed::new(seed),
                Generator::for_poi(kind).version(),
                Generator::for_poi(kind).with_ruin(ruin),
            )
            .expect("the current version runs");
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
                                Cell::Hole => ' ',
                                Cell::Window => 'W',
                                Cell::BlindWindow => 'w',
                                Cell::BuildingDoor => '+',
                                Cell::Garden => '"',
                                Cell::Field => ':',
                                Cell::Fence => '|',
                                Cell::FieldGate => '/',
                                Cell::HerbBed => ';',
                                cell if cell.is_road() => '=',
                                cell => panic!("no letter for {cell:?}"),
                            }
                        })
                        .collect();
                    println!("{row}");
                }
            }
        }
    }

    #[test]
    fn about_a_third_of_sturdy_buildings_are_ruined_at_every_level() {
        let mut counts = [0; 6];
        for seed in 0..3000 {
            let look = PoiLook::of(PoiKind::Castle, Seed::new(seed));
            let PoiLook::Building { ruin, .. } = look else {
                panic!("a castle is a building");
            };
            counts[ruin as usize] += 1;
        }
        assert!((1900..2300).contains(&counts[0]), "{counts:?}");
        for count in &counts[1..] {
            assert!((100..260).contains(count), "{counts:?}");
        }
    }

    #[test]
    fn tents_are_never_ruined() {
        assert!((0..300).all(|seed| !PoiLook::of(PoiKind::RedTent, Seed::new(seed)).is_ruined()));
    }

    fn floors_of(kind: PoiKind) -> impl Iterator<Item = Site> {
        (0..60).map(move |n| Site::new(kind, Seed::new(n * 31 + 7)))
    }

    #[test]
    fn castles_and_forts_have_at_least_three_floors() {
        for kind in [PoiKind::Castle, PoiKind::Fort] {
            assert!(floors_of(kind).all(|site| site.levels() >= 3), "{kind:?}");
        }
    }

    /// Each floor is 3 or 4 cells smaller per side, and the top leaves no room
    /// for another: its walls and a 3 by 3 room at most one step from too small.
    #[test]
    fn pyramids_step_in_until_no_room_fits() {
        for kind in [PoiKind::StonePyramid, PoiKind::SandPyramid] {
            for site in floors_of(kind) {
                assert!(site.levels() >= 3, "{}", site.levels());
                let plan = building_plan(&site);
                let sizes = plan.floor_widths();
                for pair in sizes.windows(2) {
                    let by = (pair[0] - pair[1]) / 2;
                    assert!((3..=4).contains(&by), "{sizes:?}");
                }
                let top = sizes[sizes.len() - 1];
                assert!((5..11).contains(&top), "{sizes:?}");
            }
        }
    }

    /// At least 5 floors, all alike, except that a tower with more may have a
    /// top about half the size.
    #[test]
    fn towers_are_tall_and_the_same_all_the_way_up() {
        for kind in [PoiKind::StoneTower, PoiKind::WoodenTower] {
            let mut small_tops = 0;
            for site in floors_of(kind) {
                let sizes = building_plan(&site).floor_widths();
                assert!(sizes.len() >= 5, "{sizes:?}");
                let (top, rest) = sizes.split_last().unwrap();
                assert!(rest.iter().all(|&size| size == sizes[0]), "{sizes:?}");
                if *top != sizes[0] {
                    small_tops += 1;
                    assert!(sizes.len() > 5, "{sizes:?}");
                    assert!((sizes[0] / 3..=sizes[0] * 2 / 3).contains(top), "{sizes:?}");
                }
            }
            assert!(small_tops > 0);
        }
    }

    fn building_plan(site: &Site) -> &building::Plan {
        match &site.plan {
            Plan::Building(plan) => plan,
            _ => panic!("a building"),
        }
    }

    #[test]
    fn only_big_buildings_may_have_extra_stairs() {
        let most_stairs_up = |kind| {
            floors_of(kind)
                .flat_map(|site| {
                    (0..site.levels()).map(move |index| {
                        let level = site.level(index);
                        let up = |s: &&Stairs| s.direction == StairsDirection::Up;
                        level.stairs.iter().filter(up).count()
                    })
                })
                .max()
                .unwrap()
        };
        assert_eq!(most_stairs_up(PoiKind::StoneCottage), 1);
        assert!(most_stairs_up(PoiKind::Castle) > 1);
    }
}
