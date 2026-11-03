//! Surface samples, climate and cover kept by grid position.

use bevy::prelude::*;
use shared::climate::{Climate, ClimateSample};
use shared::surface::{Surface, SurfaceSample};
use shared::vegetation::{Cover, Vegetation};

use super::CACHE_SIDE;

/// Surface samples by grid position, stored in a wrapping 2D grid. Moving one
/// step only samples the new edge; everything else is reused.
#[derive(Resource)]
pub struct HeightCache {
    keys: Vec<Option<(i64, i64)>>,
    samples: Vec<SurfaceSample>,
    climate_keys: Vec<Option<(i64, i64)>>,
    climates: Vec<ClimateSample>,
    cover_keys: Vec<Option<(i64, i64)>>,
    covers: Vec<Cover>,
}

impl Default for HeightCache {
    fn default() -> Self {
        let len = (CACHE_SIDE * CACHE_SIDE) as usize;
        Self {
            keys: vec![None; len],
            samples: vec![
                SurfaceSample {
                    height: 0.0,
                    jitter: 0.0
                };
                len
            ],
            climate_keys: vec![None; len],
            climates: vec![ClimateSample::default(); len],
            cover_keys: vec![None; len],
            covers: vec![Cover::default(); len],
        }
    }
}

impl HeightCache {
    pub fn get(&mut self, surface: &Surface, x: i64, y: i64) -> SurfaceSample {
        let index = (y.rem_euclid(CACHE_SIDE) * CACHE_SIDE + x.rem_euclid(CACHE_SIDE)) as usize;
        if self.keys[index] != Some((x, y)) {
            self.keys[index] = Some((x, y));
            self.samples[index] = surface.sample(x, y);
        }
        self.samples[index]
    }

    /// The climate at a point, kept so a new hour needn't work it out again.
    pub fn climate(
        &mut self,
        climate: &Climate,
        surface: &Surface,
        x: i64,
        y: i64,
    ) -> ClimateSample {
        let index = (y.rem_euclid(CACHE_SIDE) * CACHE_SIDE + x.rem_euclid(CACHE_SIDE)) as usize;
        if self.climate_keys[index] != Some((x, y)) {
            let height = self.get(surface, x, y).height;
            self.climate_keys[index] = Some((x, y));
            self.climates[index] = climate.sample(x, y, height);
        }
        self.climates[index]
    }

    /// What grows at a point, kept like the climate.
    pub fn cover(
        &mut self,
        vegetation: &Vegetation,
        climate: &Climate,
        surface: &Surface,
        x: i64,
        y: i64,
    ) -> Cover {
        let index = (y.rem_euclid(CACHE_SIDE) * CACHE_SIDE + x.rem_euclid(CACHE_SIDE)) as usize;
        if self.cover_keys[index] != Some((x, y)) {
            let place = self.climate(climate, surface, x, y);
            let height = self.get(surface, x, y).height;
            self.cover_keys[index] = Some((x, y));
            self.covers[index] = vegetation.cover(x, y, height, place);
        }
        self.covers[index]
    }
}
