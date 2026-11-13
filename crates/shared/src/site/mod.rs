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
mod dev_area;
mod farm;
mod grid;
mod herbalist;
mod oasis;
mod pond;
mod ruin;
pub use ruin::MAX_RUIN;
mod town;
mod wall;

pub use building::{
    BuildingOptions, CastleOptions, GATE_TOWER_SIZE, Outline, PoiCastle, TOWER_SIZE, TowerSpot,
    tower_spots,
};
pub use cave::{CaveEntrance, CaveOptions};
pub use dev_area::{DEV_AREA_KIND, DEV_AREA_SIZES, DevArea, DevLayout, DevMobs, MOST_DEV_MOBS};
pub use farm::{FarmOptions, FieldLook};
pub use herbalist::HerbalistOptions;
pub use oasis::OasisOptions;
pub use town::{BuildingKind, BuildingLook, Rect, RoofKind, TownBuilding, TownOptions, WallKind};
pub use wall::{Gate, PoiWall, WallLook, WallMaterial, WallOptions};

// Split into files (`cargo xtask lint` keeps files small).
mod descent;
mod for_poi;
mod generator;
mod level;
mod poi_look;
#[allow(clippy::module_inception)]
mod site;
mod stairs;
mod versions;

pub use generator::Generator;
pub use level::{Exit, SiteLevel, can_leave, leads_out};
pub use poi_look::PoiLook;
pub use site::Site;
pub use stairs::{Stairs, StairsDirection, StairsStyle};
pub use versions::{
    BOXED_CAVE_VERSION, BUILDING_VERSION, CAVE_VERSION, DEV_AREA_VERSION, FARM_VERSION,
    FORGET_BEFORE, HERBALIST_VERSION, OASIS_VERSION, STILTS_VERSION, TOWN_VERSION,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod picture_tests;
