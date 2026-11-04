//! The world the player is in, and the world map chunks and pois known.

use bevy::prelude::*;
use shared::celestial::Celestial;
use shared::climate::Climate;
use shared::movement::{WorldConditions, WorldGround};
use shared::poi::{CHUNK_SIZE, ChunkPos};
use shared::protocol::{SeenPoi, WorldInfo};
use shared::river::Rivers;
use shared::surface::Surface;
use shared::vegetation::Vegetation;
use shared::weather::WeatherField;
use shared::world::WorldParams;
use std::collections::BTreeMap;

use super::{GameClock, KnownRoads, MOST_PLANETS_IN_AN_AREA, Weather};

/// The world the player is in, its surface generator and the pois the
/// server has told the client about. Only exists once the server has said
/// which world it is (`connection`).
#[derive(Resource)]
pub struct CurrentWorld {
    pub params: WorldParams,
    pub surface: Surface,
    pub climate: Climate,
    pub rivers: Rivers,
    pub vegetation: Vegetation,
    pub weather: WeatherField,
    pub celestial: Celestial,
    /// The ground as far as walking goes (the server's rules).
    pub ground: WorldGround,
    pub pois: KnownPois,
    /// The roads the server has told of.
    pub roads: KnownRoads,
}

impl CurrentWorld {
    /// The world a server has told the client about.
    pub fn new(info: &WorldInfo) -> Self {
        let params = WorldParams::from_seed(info.terrain_seed);
        let surface = Surface::new(&params);
        Self {
            climate: Climate::new(&params),
            rivers: Rivers::new(&params),
            vegetation: Vegetation::new(&params),
            weather: WeatherField::new(&params),
            celestial: Celestial::from_seed(params.seed),
            pois: KnownPois::new(surface.width()),
            roads: KnownRoads::default(),
            ground: WorldGround::new(&params),
            surface,
            params,
        }
    }

    /// What decides where can be walked now.
    pub fn conditions(&self, clock: &GameClock, weather: &Weather) -> WorldConditions {
        WorldConditions {
            water: self.params.water_level(clock.hour as f64),
            snow_line: weather.snow_line,
        }
    }
}

/// The world map chunks the server has sent, and their pois.
pub struct KnownPois {
    /// The world's width: `x` wraps at this.
    width: i64,
    /// By chunk, north-south first (so an area's rows can be looked up);
    /// `None` for a chunk known to have no poi.
    chunks: BTreeMap<(i64, i64), Option<SeenPoi>>,
}

impl KnownPois {
    pub fn new(width: i64) -> Self {
        Self {
            width,
            chunks: BTreeMap::new(),
        }
    }

    /// What the server says is in a chunk.
    pub fn forget_all(&mut self) {
        *self = Self::new(self.width);
    }

    pub fn insert(&mut self, chunk: ChunkPos, poi: Option<SeenPoi>) {
        self.chunks.insert((chunk.y, chunk.x), poi);
    }

    /// Whether the server has sent a chunk (numbered as it does, wrapped round
    /// the planet).
    pub fn is_known(&self, chunk: ChunkPos) -> bool {
        self.chunks.contains_key(&(chunk.y, chunk.x))
    }

    /// Every known poi whose position is within `min..=max` (world grid
    /// points, in any planet-width frame: the pois are given in the
    /// same one), like `shared::poi::PoiMap::in_area`.
    pub fn in_area(&self, min: (i64, i64), max: (i64, i64)) -> Vec<SeenPoi> {
        // Nothing asks for more than a few planets on purpose: far more means
        // a broken position, which would take for ever or overflow below.
        if max.0.saturating_sub(min.0) > MOST_PLANETS_IN_AN_AREA * self.width {
            warn!("Asked for the pois of an area far too wide: {min:?} to {max:?}");
            return Vec::new();
        }
        let rows =
            (min.1.div_euclid(CHUNK_SIZE), i64::MIN)..=(max.1.div_euclid(CHUNK_SIZE), i64::MAX);
        let mut found = Vec::new();
        for poi in self.chunks.range(rows).filter_map(|(_, poi)| *poi) {
            // Each copy of the planet the area overlaps.
            for copy in min.0.div_euclid(self.width)..=max.0.div_euclid(self.width) {
                let x = poi.x + copy * self.width;
                if (min.0..=max.0).contains(&x) && (min.1..=max.1).contains(&poi.y) {
                    found.push(SeenPoi { x, ..poi });
                }
            }
        }
        found.sort_by_key(|poi| (poi.y, poi.x));
        found
    }
}
