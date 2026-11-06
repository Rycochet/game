//! What a chunk of the wilderness holds besides its terrain: its poi, and the
//! rivers and roads passing through.

use super::course::Segment;
use super::{Cutout, Wilderness};
use crate::poi::{CHUNK_SIZE, ChunkPos, Poi, PoiMap};
use crate::river::MAX_LENGTH;
use crate::road::{Road, RoadKind};
use crate::site::{Generator, Site};

/// How far outside its chunk a river or road still matters to it, in cells:
/// the widest river, the shallows round it and the trees thinning beyond.
const COURSE_MARGIN: f64 = 40.0;

/// One chunk's plan: what its blocks are made from besides the terrain.
#[derive(Clone, Debug)]
pub struct ChunkPlan {
    chunk: ChunkPos,
    cutout: Option<Cutout>,
    pub(super) rivers: Vec<Segment>,
    pub(super) roads: Vec<(Segment, RoadKind)>,
}

impl ChunkPlan {
    /// The plan of a chunk whose poi's site is `site` (the server's, made
    /// with the version it saved), or which has none, with the `roads` the
    /// server has worked out near it (more may come, and the plan be made
    /// again).
    pub fn new(
        wild: &Wilderness,
        chunk: ChunkPos,
        site: Option<(Poi, &Site)>,
        roads: &[&Road],
    ) -> Self {
        let chunk = ChunkPos {
            x: chunk.x.rem_euclid(wild.columns()),
            y: chunk.y,
        };
        let cutout = site.map(|(poi, site)| Cutout::new(wild, poi, &site.level(0)));
        let bounds = wild.chunk_bounds(chunk);
        let near = |segment: &Segment| segment.near(bounds, COURSE_MARGIN);
        let shifts = [-wild.points, 0, wild.points];
        // A river may start up to `MAX_LENGTH` points away.
        let reach = MAX_LENGTH.div_ceil(CHUNK_SIZE as usize) as i64;
        let mut rivers = Vec::new();
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let from = ChunkPos {
                    x: (chunk.x + dx).rem_euclid(wild.columns()),
                    y: chunk.y + dy,
                };
                let Some(river) = wild.rivers.in_chunk(from) else {
                    continue;
                };
                for shift in shifts {
                    rivers.extend(wild.river_segments(&river, shift).into_iter().filter(near));
                }
            }
        }
        let mut found = Vec::new();
        for road in roads {
            for shift in shifts {
                let segments = wild.road_segments(road, shift).into_iter().filter(near);
                found.extend(segments.map(|segment| (segment, road.kind)));
            }
        }
        Self {
            chunk,
            cutout,
            rivers,
            roads: found,
        }
    }

    /// The plan with the chunk's poi's site made as new, by its generator's
    /// current version, and no roads.
    pub fn with_new_site(wild: &Wilderness, pois: &PoiMap, chunk: ChunkPos) -> Self {
        let poi = pois.in_chunk(chunk);
        let site = poi.map(|poi| {
            let generator = Generator::for_site(poi.kind, poi.seed, poi.desert);
            Site::with_generator(poi.kind, poi.seed, generator.version(), generator)
                .expect("the current version runs")
        });
        Self::new(wild, chunk, poi.zip(site.as_ref()), &[])
    }

    pub fn chunk(&self) -> ChunkPos {
        self.chunk
    }

    pub fn cutout(&self) -> Option<&Cutout> {
        self.cutout.as_ref()
    }

    /// Whether any river runs through or near it.
    pub fn has_rivers(&self) -> bool {
        !self.rivers.is_empty()
    }

    /// Whether any road runs through or near it.
    pub fn has_roads(&self) -> bool {
        !self.roads.is_empty()
    }
}
