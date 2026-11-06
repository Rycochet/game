//! Where the pois of a world are: which chunk is where.

use super::{CHUNK_SIZE, ChunkPos};
use crate::climate::Climate;
use crate::movement::WorldGround;
use crate::rng::Seed;
use crate::surface::Surface;
use crate::world::WorldParams;
use crate::world_gate::WorldGates;

/// Finds the pois of one world.
pub struct PoiMap {
    pub(super) seed: Seed,
    /// The world's width in grid points (`Surface::width`): `x` wraps at this.
    pub(super) width: i64,
    /// Its own copy of the surface, to keep pois on land.
    pub(super) surface: Surface,
    /// What grows where, as farms only stand where crops would.
    pub(super) climate: Climate,
    /// Pois stand only where the ground is above this: high tide.
    pub(super) land_above: f64,
    /// The world's World Gates: their chunks have no other poi.
    gates: WorldGates,
    /// For where the ice can't be crossed, which has no pois.
    pub(super) ground: WorldGround,
}

impl PoiMap {
    /// `seed` is the world's secret pois seed (`WorldSeed::pois`):
    /// the terrain is public, but where pois are and what is inside
    /// them isn't.
    pub fn new(world: &WorldParams, seed: Seed) -> Self {
        Self::with_water(world, seed, world.high_water())
    }

    /// As `new`, with pois only where the ground is above `land_above`.
    pub(super) fn with_water(world: &WorldParams, seed: Seed, land_above: f64) -> Self {
        let surface = Surface::new(world);
        Self {
            gates: WorldGates::find(world, &surface),
            seed,
            width: surface.width(),
            surface,
            climate: Climate::new(world),
            land_above,
            ground: WorldGround::new(world),
        }
    }

    /// The world's World Gates.
    pub fn world_gates(&self) -> &WorldGates {
        &self.gates
    }

    /// Whether a chunk (`x` wrapped) has a World Gate, and so no other poi.
    pub(super) fn has_gate(&self, chunk: ChunkPos) -> bool {
        self.gates.gates.iter().any(|gate| gate.chunk == chunk)
    }

    /// The world's width in grid points: `x` wraps at this.
    pub fn width(&self) -> i64 {
        self.width
    }

    /// Chunk columns round the planet. The width is rarely a whole number of
    /// chunks, so the last column is wider, taking up the remainder.
    pub fn columns(&self) -> i64 {
        (self.width / CHUNK_SIZE).max(1)
    }

    /// The chunk containing world grid point `(x, y)`.
    pub fn chunk_at(&self, x: i64, y: i64) -> ChunkPos {
        ChunkPos {
            x: (x.rem_euclid(self.width) / CHUNK_SIZE).min(self.columns() - 1),
            y: y.div_euclid(CHUNK_SIZE),
        }
    }
}
