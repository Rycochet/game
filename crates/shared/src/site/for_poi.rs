//! The generator each kind of poi uses.

use super::Generator;
use super::building::{BuildingOptions, Outline};
use super::cave::{CaveEntrance, CaveOptions};
use super::farm::FarmOptions;
use super::herbalist::HerbalistOptions;
use super::town::TownOptions;
use crate::poi::PoiKind;

/// Of the places in outside walls a window could go, how many have one.
const WINDOWS_PERCENT: u32 = 30;

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
