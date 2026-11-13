//! The generator each kind of poi uses.

use super::Generator;
use super::building::{BuildingOptions, CastleOptions, Outline};
use super::cave::{CaveEntrance, CaveOptions};
use super::farm::FarmOptions;
use super::herbalist::HerbalistOptions;
use super::oasis::OasisOptions;
use super::town::{CITY_WALLS, HAMLET_WALLS, TOWN_WALLS, TownOptions, VILLAGE_WALLS};
use super::wall::{WallMaterial, WallOptions};
use crate::poi::PoiKind;

/// Of the places in outside walls a window could go, how many have one.
const WINDOWS_PERCENT: u32 = 30;

/// Half of forts have a wall round them (owner, 2026-11-13): 2 to 8 cells
/// out, with a pond of 2 × 2 to 3 × 3 inside, mostly of planks or stone,
/// sometimes logs.
const FORT_WALL: WallOptions = WallOptions {
    percent: 50,
    gap: (2, 8),
    pond: Some((2, 3)),
    materials: &[
        (WallMaterial::Plank, 40),
        (WallMaterial::Stone, 40),
        (WallMaterial::Log, 20),
    ],
};

/// Castles (owner, 2026-11-13): an outer wall of stone or dressed blocks
/// round 40–70 × 36–60 cells, 5 to 8 cells of courtyard to the keep, a pond of
/// 3 × 3 to 4 × 4, a moat of 2 to 3 (4 round the biggest) more often than not,
/// and towers of 2 to 3 floors.
const CASTLE: CastleOptions = CastleOptions {
    width: (40, 70),
    height: (36, 60),
    gap: (5, 8),
    pond: (3, 4),
    moat_percent: 60,
    moat: (2, 3),
    wide_moat_from: 110,
    tower_floors: (2, 3),
    materials: &[(WallMaterial::Stone, 50), (WallMaterial::Block, 50)],
};

impl Generator {
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
                wall: None,
                castle: None,
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
        // Owner, 2026-11-13: the smaller, the likelier a wall. Main roads
        // follow the road network's sides once `for_site` knows them.
        let town = |buildings, wall_percent, wall_materials| {
            Self::Town(TownOptions {
                buildings,
                sides: [false; 4],
                wall_percent,
                wall_materials,
            })
        };
        let at_least_three_floors = |generator| match generator {
            Self::Building(options) => Self::Building(BuildingOptions {
                min_floors: 3,
                ..options
            }),
            other => other,
        };
        match kind {
            Hamlet => town((3, 8), 60, HAMLET_WALLS),
            Village => town((9, 25), 45, VILLAGE_WALLS),
            Town => town((26, 60), 30, TOWN_WALLS),
            City => town((61, 150), 15, CITY_WALLS),
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
                    wall: None,
                    castle: None,
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
                    wall: None,
                    castle: None,
                },
                strips: (3, 5),
                length: (7, 12),
            }),
            Oasis => Self::Oasis(OasisOptions {
                size: (40, 56),
                pools: (1, 2),
                radius: (5, 9),
                grass: (2, 4),
                trees_percent: 30,
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
            // A keep of at least 3 floors inside the castle's wall.
            Castle => match at_least_three_floors(building(
                CASTLE.width,
                CASTLE.height,
                &[Rectangle, CutCorners, RoundCorners],
                4,
                12,
            )) {
                Self::Building(options) => Self::Building(BuildingOptions {
                    castle: Some(CASTLE),
                    ..options
                }),
                other => other,
            },
            // Always a basement (owner); half have a wall round them.
            Fort => match at_least_three_floors(building(
                (30, 50),
                (26, 44),
                &[Rectangle, CutCorners],
                3,
                12,
            )) {
                Self::Building(options) => Self::Building(BuildingOptions {
                    basement_percent: 100,
                    wall: Some(FORT_WALL),
                    ..options
                }),
                other => other,
            },
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
                wall: None,
                castle: None,
            }),
        }
    }
}
