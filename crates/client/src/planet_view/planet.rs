//! The planet on show: the current world, or what another seed or generator
//! would make. Only for looking at: the game itself always uses the world's
//! own surface.

use shared::celestial::Celestial;
use shared::climate::{Climate, ClimateSample};
use shared::rng::Seed;
use shared::surface::{Surface, SurfaceGenerator, SurfaceSample};
use shared::vegetation::Vegetation;
use shared::world::WorldParams;
use shared::world_gate::WorldGates;

use crate::common::CurrentWorld;

/// The deepest ocean shown by its colour, on the height scale (8 km down).
const DEEPEST_SHOWN: f64 = -80.0;

pub(super) struct Planet {
    pub(super) params: WorldParams,
    pub(super) surface: Surface,
    pub(super) climate: Climate,
    pub(super) vegetation: Vegetation,
    pub(super) gates: WorldGates,
    /// Its suns and moons when it isn't the current world; the current world's
    /// are taken from it, as the debug tools may have swapped them.
    own_sky: Option<Celestial>,
}

impl Planet {
    pub(super) fn new(world: &CurrentWorld, seed: Seed, generator: SurfaceGenerator) -> Self {
        let params = WorldParams::from_seed(seed);
        let surface = Surface::with_generator(&params, generator);
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

    /// The sample as the world map's colours expect it: they shade water by
    /// a depth no deeper than `Classic`'s seas, so deeper oceans are squeezed
    /// into that, or they would all be black.
    pub(super) fn shown(&self, sample: SurfaceSample) -> SurfaceSample {
        let sea = self.params.sea_level;
        if self.surface.generator() == SurfaceGenerator::Classic || sample.height >= sea {
            return sample;
        }
        let depth = (sea - sample.height) / (sea - DEEPEST_SHOWN);
        SurfaceSample {
            height: sea - sea * depth.min(1.0),
            ..sample
        }
    }

    /// Whether this is sea frozen over, for surfaces in metres: `Classic`
    /// freezes by the snow line instead.
    pub(super) fn has_sea_ice(&self, sample: SurfaceSample, climate: ClimateSample) -> bool {
        self.surface.generator() != SurfaceGenerator::Classic
            && sample.height < self.params.sea_level
            && climate.celsius() < shared::climate::SEA_ICE
    }

    /// The climate over ground of this sample: cooling with height as the
    /// world's own does for `Classic`, at Earth's rate from the sea for a
    /// surface worked out in metres.
    pub(super) fn climate_at(&self, x: i64, y: i64, sample: SurfaceSample) -> ClimateSample {
        match self.surface.generator() {
            SurfaceGenerator::Classic => self.climate.sample(x, y, sample.height),
            SurfaceGenerator::Continents | SurfaceGenerator::Plates => {
                self.climate
                    .sample_metres(x, y, self.surface.metres(sample.height))
            }
        }
    }
}
