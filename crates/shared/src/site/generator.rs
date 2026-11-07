//! Which generator a site uses, at which version and how ruined.

use super::building::BuildingOptions;
use super::cave::CaveOptions;
use super::farm::FarmOptions;
use super::herbalist::HerbalistOptions;
use super::oasis::OasisOptions;
use super::town::TownOptions;
use super::{
    BUILDING_VERSION, CAVE_VERSION, DEV_AREA_VERSION, FARM_VERSION, HERBALIST_VERSION,
    OASIS_VERSION, STILTS_VERSION, TOWN_VERSION,
};
use crate::poi::PoiKind;
use crate::rng::Seed;

/// Of each 100 buildings that can be ruined, how many are, evenly from 1 to
/// `MOST_RUIN`.
const RUINED_PERCENT: u64 = 30;
/// The same in desert (owner: more ruins there).
const DESERT_RUINED_PERCENT: u64 = 60;
const MOST_RUIN: u64 = 5;

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
    /// Open ground with pools ringed by grass and trees.
    Oasis(OasisOptions),
    /// Developer tools: a walled area made to order (`DevArea`).
    DevArea(super::DevArea),
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
            Self::Oasis(_) => OASIS_VERSION,
            Self::DevArea(_) => DEV_AREA_VERSION,
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

    /// The generator of one poi's site: its kind's, ruined as its map seed
    /// says, more often in desert (`Poi::desert`).
    pub fn for_site(kind: PoiKind, seed: Seed, desert: bool) -> Self {
        if !Self::is_sturdy(kind) {
            return Self::for_poi(kind);
        }
        let percent = if desert {
            DESERT_RUINED_PERCENT
        } else {
            RUINED_PERCENT
        };
        let roll = seed.derive("how ruined").at(&[0]) % 100;
        let ruin = roll.saturating_sub(100 - percent) * MOST_RUIN / percent;
        let ruined = roll >= 100 - percent;
        Self::for_poi(kind).with_ruin(if ruined { ruin as u8 + 1 } else { 0 })
    }
}
