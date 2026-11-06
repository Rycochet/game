//! The planet on show: the current world, or what another seed or generator
//! would make. Only for looking at: the game itself always uses the world's
//! own surface.

use shared::celestial::Celestial;
use shared::climate::Climate;
use shared::rng::Seed;
use shared::surface::{Surface, SurfaceGenerator};
use shared::vegetation::Vegetation;
use shared::world::WorldParams;
use shared::world_gate::WorldGates;

use crate::common::CurrentWorld;

pub(super) struct Planet {
    pub(super) params: WorldParams,
    pub(super) surface: Surface,
    /// By the chosen generator, as the world's own climate is by its own.
    pub(super) climate: Climate,
    pub(super) vegetation: Vegetation,
    pub(super) gates: WorldGates,
    /// Its suns and moons when it isn't the current world; the current world's
    /// are taken from it, as the debug tools may have swapped them.
    own_sky: Option<Celestial>,
}

impl Planet {
    pub(super) fn new(world: &CurrentWorld, seed: Seed, generator: SurfaceGenerator) -> Self {
        let params = WorldParams::with_generator(seed, generator);
        let surface = Surface::new(&params);
        Self {
            gates: WorldGates::find(&params, &surface),
            surface,
            climate: Climate::new(&params),
            vegetation: Vegetation::new(&params),
            own_sky: (seed != world.params.seed).then(|| Celestial::from_seed(seed)),
            params,
        }
    }

    pub(super) fn celestial<'a>(&'a self, world: &'a CurrentWorld) -> &'a Celestial {
        self.own_sky.as_ref().unwrap_or(&world.celestial)
    }
}
