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
pub use ruin::MAX_RUIN;
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
            StiltHut | GreenStiltHut => &[Ladder, LogLadder],
            ThatchedHut | RedTent | BlueTent | GreenTent | RedRoofCottage | GreyRoofCottage
            | SnowyCottage | WoodenTower => &[Turning, Ladder, LogLadder],
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
            Self::Building(options) if options.stilts => STILTS_VERSION,
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

    /// Whether this kind of poi can be a ruin: buildings, except those too flimsy
    /// to leave one.
    pub fn can_be_ruined(kind: PoiKind) -> bool {
        Self::is_sturdy(kind) && Self::for_poi(kind).with_ruin(1).ruin() > 0
    }

    fn is_sturdy(kind: PoiKind) -> bool {
        !matches!(
            kind,
            PoiKind::RedTent | PoiKind::BlueTent | PoiKind::GreenTent | PoiKind::IceShelter
        )
    }

    /// The generator of one poi's site: its kind's, ruined as its map seed says.
    pub fn for_site(kind: PoiKind, seed: Seed) -> Self {
        if !Self::is_sturdy(kind) {
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
            Castle | Fort | StoneTower | WoodenTower | RedTent | BlueTent | GreenTent | IceShelter
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
                stilts: false,
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
                    stilts: false,
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
                    stilts: false,
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
            ThatchedHut | RedTent | BlueTent | GreenTent | IceShelter => {
                building((9, 13), (9, 13), &[Circle], 1, 10)
            }
            // The same, up on pillars: the ground, and the hut over it.
            StiltHut | GreenStiltHut => match building((9, 13), (9, 13), &[Circle], 2, 10) {
                Self::Building(options) => Self::Building(BuildingOptions {
                    min_floors: 2,
                    same_size: true,
                    stilts: true,
                    ..options
                }),
                other => other,
            },
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
                windows_percent: 0,
                ground_windows,
                stilts: false,
            }),
        }
    }
}

/// The building generator's version: bump it when its output changes in a
/// way that matters (not for looks), and keep the old one runnable while
/// saved sites use it. Bumping `FORGET_BEFORE` ends that: no old version is left
/// in use, so all the code for them can go.
pub const BUILDING_VERSION: u32 = 1 + FORGET_BEFORE;

/// The version of buildings on stilts (as `BUILDING_VERSION`). A stilt hut saved
/// under `BUILDING_VERSION` is still the plain hut it was first made as.
pub const STILTS_VERSION: u32 = 1 + BUILDING_VERSION;

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
            Generator::Building(options) if version == generator.version() => {
                Plan::Building(building::Plan::new(&options, seed))
            }
            Generator::Building(options) if options.stilts && version == BUILDING_VERSION => {
                let options = BuildingOptions {
                    min_floors: 1,
                    max_floors: 1,
                    same_size: false,
                    stilts: false,
                    ..options
                };
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
mod tests;

#[cfg(test)]
mod picture_tests;
