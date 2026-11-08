//! Terrain that slows walking off the roads, and the Pioneering skill under
//! Survival with a child for each (owner, 2026-11-08): grass and scrub 1.2
//! times as long as open ground, forest 1.3, jungle 1.5, snow 1.8 and desert
//! sand 2 (rough figures from walking-effort research), from the biome, taken
//! in squares of `TERRAIN_STEP` points. 100% in a terrain's skill takes off two
//! thirds of its extra.

use crate::climate::Biome;
use crate::road::RoadKind;
use crate::skill::SkillId;

use super::{WorldConditions, WorldGround};

/// What 100% in a terrain's Pioneering skill takes off its extra cost, per
/// mille (owner: two thirds).
pub const PIONEER_SAVING_PER_MILLE: u32 = 667;

/// Ground that slows walking, each with its own Pioneering skill.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Terrain {
    Grassland,
    Forest,
    Jungle,
    Snow,
    Desert,
}

impl Terrain {
    pub const ALL: [Terrain; 5] = [
        Self::Grassland,
        Self::Forest,
        Self::Jungle,
        Self::Snow,
        Self::Desert,
    ];

    /// How long a step over it takes against open ground, per mille.
    pub const fn per_mille(self) -> u32 {
        match self {
            Self::Grassland => 1200,
            Self::Forest => 1300,
            Self::Jungle => 1500,
            Self::Snow => 1800,
            Self::Desert => 2000,
        }
    }

    /// The terrain of a biome: every land biome has one. (No marshes yet: the
    /// climate has no wetlands.)
    pub const fn of_biome(biome: Biome) -> Terrain {
        match biome {
            Biome::Grassland | Biome::Savanna => Self::Grassland,
            Biome::BorealForest
            | Biome::Woodland
            | Biome::SeasonalForest
            | Biome::TemperateRainforest => Self::Forest,
            Biome::TropicalRainforest => Self::Jungle,
            Biome::Tundra => Self::Snow,
            Biome::Desert => Self::Desert,
        }
    }

    /// Its place in the skill tree, under `survival/pioneering`.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Grassland => "grassland",
            Self::Forest => "forest",
            Self::Jungle => "jungle",
            Self::Snow => "snow",
            Self::Desert => "desert",
        }
    }

    /// Its Pioneering skill.
    pub fn skill(self) -> SkillId {
        let path = format!("survival/pioneering/{}", self.key());
        SkillId::find(&path).expect("every terrain has a Pioneering skill")
    }
}

/// What a step lands on, for its cost: a road, water (a river or the sea, or
/// shallows on a level) or a terrain, which count in that order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Footing {
    pub road: Option<RoadKind>,
    pub wet: bool,
    pub terrain: Option<Terrain>,
}

impl Footing {
    /// Whether it is waded or swum (water off any road).
    pub fn wades(self) -> bool {
        self.wet && self.road.is_none()
    }

    /// The terrain that counts: none on a road or in water.
    pub fn slowed_by(self) -> Option<Terrain> {
        if self.road.is_some() || self.wet {
            None
        } else {
            self.terrain
        }
    }
}

impl WorldGround {
    /// What a world map step landing on `to` stands on: `road` if it runs along
    /// one; water in the sea or on a river (`on_river`, from
    /// `river::RiverPoints`); otherwise the point's terrain.
    pub fn footing(
        &self,
        to: (i64, i64),
        conditions: WorldConditions,
        road: Option<RoadKind>,
        on_river: bool,
    ) -> Footing {
        let land = self.elevation(to.0, to.1, conditions).is_some();
        let wet = !land || on_river;
        let terrain = (land && !wet && road.is_none()).then(|| self.land_terrain(to.0, to.1));
        Footing { road, wet, terrain }
    }

    /// A world map point's terrain: its biome's on land, none in open water.
    pub fn terrain(&self, x: i64, y: i64, conditions: WorldConditions) -> Option<Terrain> {
        self.elevation(x, y, conditions)?;
        Some(self.land_terrain(x, y))
    }

    /// `terrain` at a point already known to be walkable (`elevation` is
    /// `Some`), for searches that have looked: the biome at the corner of its
    /// `TERRAIN_STEP` square, as biomes change slowly and the climate is slow
    /// to sample.
    pub(crate) fn land_terrain(&self, x: i64, y: i64) -> Terrain {
        let (sx, sy) = self.terrain_square(x, y);
        let height = self.surface.sample(sx, sy).height;
        Terrain::of_biome(self.climate.sample(sx, sy, height).biome())
    }
}

impl WorldGround {
    /// The corner of the `TERRAIN_STEP` square a point is in, round the
    /// planet: points in the same square have the same terrain.
    pub(crate) fn terrain_square(&self, x: i64, y: i64) -> (i64, i64) {
        terrain_corner(x.rem_euclid(self.width.max(1)), y)
    }
}

/// Points along each side of the squares terrain is sampled in.
pub const TERRAIN_STEP: i64 = 4;

/// The corner of the `TERRAIN_STEP` square a point (wrapped) is in, whose biome
/// it takes.
fn terrain_corner(x: i64, y: i64) -> (i64, i64) {
    (
        x.div_euclid(TERRAIN_STEP) * TERRAIN_STEP,
        y.div_euclid(TERRAIN_STEP) * TERRAIN_STEP,
    )
}

#[cfg(test)]
mod tests;
