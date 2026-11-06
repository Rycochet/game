//! What a chunk of the wilderness holds besides its terrain: its poi.

use super::{Cutout, Wilderness};
use crate::poi::{ChunkPos, Poi, PoiMap};
use crate::site::{Generator, Site};

/// One chunk's plan: what its blocks are made from besides the terrain.
#[derive(Clone, Debug)]
pub struct ChunkPlan {
    chunk: ChunkPos,
    cutout: Option<Cutout>,
}

impl ChunkPlan {
    /// The plan of a chunk whose poi's site is `site` (the server's, made
    /// with the version it saved), or which has none.
    pub fn new(wild: &Wilderness, chunk: ChunkPos, site: Option<(Poi, &Site)>) -> Self {
        let chunk = ChunkPos {
            x: chunk.x.rem_euclid(wild.columns()),
            y: chunk.y,
        };
        let cutout = site.map(|(poi, site)| Cutout::new(wild, poi, &site.level(0)));
        Self { chunk, cutout }
    }

    /// The plan with the chunk's poi's site made as new, by its generator's
    /// current version.
    pub fn with_new_site(wild: &Wilderness, pois: &PoiMap, chunk: ChunkPos) -> Self {
        let poi = pois.in_chunk(chunk);
        let site = poi.map(|poi| {
            let generator = Generator::for_site(poi.kind, poi.seed, poi.desert);
            Site::with_generator(poi.kind, poi.seed, generator.version(), generator)
                .expect("the current version runs")
        });
        Self::new(wild, chunk, poi.zip(site.as_ref()))
    }

    pub fn chunk(&self) -> ChunkPos {
        self.chunk
    }

    pub fn cutout(&self) -> Option<&Cutout> {
        self.cutout.as_ref()
    }
}
