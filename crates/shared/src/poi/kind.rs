//! The kinds of poi.

use serde::{Deserialize, Serialize};

/// What a poi is. The order matches the Dungeon Odyssey terrain tiles
/// `l2_terrain066` to `l2_terrain099` the client shows them with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PoiKind {
    Boulders,
    Ruins,
    GrassyCave,
    EarthCave,
    RockCave,
    IceCave,
    ThatchedHut,
    AdobeHouse,
    AdobeStore,
    StiltHut,
    RedRoofCottage,
    GreyRoofCottage,
    StoneCottage,
    Warehouse,
    GreenStiltHut,
    StoneTower,
    WoodenTower,
    RedTent,
    BlueTent,
    GreenTent,
    Castle,
    Fort,
    IceShelter,
    SnowyCottage,
    StonePyramid,
    SandPyramid,
    ArcheryRange,
    Barrel,
    Barrels,
    Crypt,
    MossyCrypt,
    BloodyCrypt,
    Grave,
    Mausoleum,
    Hamlet,
    Village,
    Town,
    City,
    Farm,
    HerbalistHut,
}

impl PoiKind {
    pub const ALL: [Self; 40] = [
        Self::Boulders,
        Self::Ruins,
        Self::GrassyCave,
        Self::EarthCave,
        Self::RockCave,
        Self::IceCave,
        Self::ThatchedHut,
        Self::AdobeHouse,
        Self::AdobeStore,
        Self::StiltHut,
        Self::RedRoofCottage,
        Self::GreyRoofCottage,
        Self::StoneCottage,
        Self::Warehouse,
        Self::GreenStiltHut,
        Self::StoneTower,
        Self::WoodenTower,
        Self::RedTent,
        Self::BlueTent,
        Self::GreenTent,
        Self::Castle,
        Self::Fort,
        Self::IceShelter,
        Self::SnowyCottage,
        Self::StonePyramid,
        Self::SandPyramid,
        Self::ArcheryRange,
        Self::Barrel,
        Self::Barrels,
        Self::Crypt,
        Self::MossyCrypt,
        Self::BloodyCrypt,
        Self::Grave,
        Self::Mausoleum,
        Self::Hamlet,
        Self::Village,
        Self::Town,
        Self::City,
        Self::Farm,
        Self::HerbalistHut,
    ];

    /// How many of `ALL` come from the chunk's plain roll: the rest have their
    /// own (`PoiMap::town_at`, `PoiMap::kept_to_land`), so they can be rarer, or
    /// keep to the land that suits them, and every other chunk keeps its poi.
    pub(super) const ROLLED: u64 = 34;

    /// Whether it is a town of any size.
    pub fn is_town(self) -> bool {
        matches!(self, Self::Hamlet | Self::Village | Self::Town | Self::City)
    }
}
