//! The wilderness as the server holds it (`shared::wilderness`): the world's
//! generator, and the chunk plans and blocks made lately, kept until there
//! are too many. Nothing here is saved: what players change of the wilderness
//! is kept with `storage::SiteId::Wilderness`, laid over the blocks as they
//! are put together for a visit (`sites`).

use shared::dungeon::Level;
use shared::poi::{CHUNK_SIZE, ChunkPos};
use shared::protocol::SeenStairs;
use shared::road::Road;
use shared::site::{BuildingLook, FieldLook, TownBuilding};
use shared::wilderness::{BlockPos, ChunkPlan, Wilderness};
use shared::world::WorldParams;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// How far in the wilderness can be seen, and others made out, in cells: it is
/// mostly open, as trees don't block sight.
pub(crate) const SIGHT: u32 = 40;

/// Blocks each side of the one the player is in that make up a visit's level.
/// Sight and what is sent ahead (`LEVEL_PRELOAD_STEPS`) reach less far than a
/// block, so the player never sees past its edge.
pub(crate) const WINDOW_BLOCKS: i64 = 1;

/// Blocks kept made before all are let go (about 4 MB).
const MOST_BLOCKS: usize = 256;

/// Chunk plans kept before all are let go.
const MOST_CHUNKS: usize = 64;

/// A chunk as the server has worked it out.
pub(crate) struct WildChunk {
    pub(crate) plan: ChunkPlan,
    /// The cutout's stairs as players see them, in wilderness cells.
    pub(crate) stairs: Vec<SeenStairs>,
    /// A town cutout's buildings and a farm's fields as players see them, in
    /// wilderness cells.
    pub(crate) buildings: Vec<BuildingLook>,
    pub(crate) fields: Vec<FieldLook>,
    /// A town cutout's buildings as its site has them, in the site's cells:
    /// what its shops are (`sites::shop_here`).
    pub(crate) town: Vec<TownBuilding>,
    /// Where exploring from the poi on the world map arrives, in wilderness
    /// cells: its site's entrance, or beside its way down (`sites::wild`).
    pub(crate) entrance: Option<(i64, i64)>,
}

/// The wilderness of one world.
pub(crate) struct WildernessMaps {
    pub(crate) wild: Wilderness,
    chunks: BTreeMap<ChunkPos, WildChunk>,
    blocks: HashMap<BlockPos, Level>,
    /// Every road worked out so far (`shared::road`), as they come.
    roads: Vec<Arc<Road>>,
    /// Who of the wilderness's mobs are about, and where.
    pub(crate) areas: crate::entities::WildAreas,
}

impl WildernessMaps {
    pub(crate) fn new(params: &WorldParams) -> Self {
        Self {
            wild: Wilderness::new(params),
            chunks: BTreeMap::new(),
            blocks: HashMap::new(),
            roads: Vec::new(),
            areas: Default::default(),
        }
    }

    /// A road has been worked out: the chunks it passes near are made again
    /// when next needed, with it.
    pub(crate) fn add_road(&mut self, road: Arc<Road>) {
        let near: Vec<ChunkPos> = self
            .chunks
            .keys()
            .copied()
            .filter(|&chunk| self.passes_near(&road, chunk))
            .collect();
        for chunk in near {
            self.forget(Some(chunk));
        }
        self.roads.push(road);
    }

    /// The roads that may pass through or near a chunk.
    pub(crate) fn roads_near(&self, chunk: ChunkPos) -> Vec<&Road> {
        let chunk = self.wrapped(chunk);
        let near = |road: &&Arc<Road>| self.passes_near(road, chunk);
        self.roads.iter().filter(near).map(|road| &**road).collect()
    }

    /// Whether a road's box comes within a world map point of a chunk, on any
    /// copy of the planet.
    fn passes_near(&self, road: &Road, chunk: ChunkPos) -> bool {
        let ((min_x, min_y), (max_x, max_y)) = road.bounds();
        let points = self.wild.points();
        let (west, north) = (chunk.x * CHUNK_SIZE - 1, chunk.y * CHUNK_SIZE - 1);
        let (east, south) = (west + CHUNK_SIZE * 2 + 2, north + CHUNK_SIZE + 2);
        [-points, 0, points].iter().any(|shift| {
            min_x + shift <= east && max_x + shift >= west && min_y <= south && max_y >= north
        })
    }

    /// A chunk's plan, if worked out already.
    pub(crate) fn chunk(&self, chunk: ChunkPos) -> Option<&WildChunk> {
        self.chunks.get(&self.wrapped(chunk))
    }

    /// Keeps a chunk's plan.
    pub(crate) fn keep_chunk(&mut self, chunk: WildChunk) {
        if self.chunks.len() >= MOST_CHUNKS {
            self.chunks.clear();
            self.blocks.clear();
        }
        self.chunks.insert(chunk.plan.chunk(), chunk);
    }

    /// A block's cells, made if they aren't kept. Its chunk's plan must be kept.
    pub(crate) fn block(&mut self, block: BlockPos) -> &Level {
        let block = BlockPos {
            x: block.x.rem_euclid(self.wild.block_columns()),
            y: block.y,
        };
        if !self.blocks.contains_key(&block) {
            if self.blocks.len() >= MOST_BLOCKS {
                self.blocks.clear();
            }
            let (x, y, _, _) = self.wild.block_bounds(block);
            let chunk = self.wild.chunk_of(x, y);
            let plan = &self.chunks[&chunk].plan;
            let level = self.wild.block(block, plan);
            self.blocks.insert(block, level);
        }
        &self.blocks[&block]
    }

    /// Forgets a chunk's plan and blocks (its site has been forgotten, so its
    /// cutout may be different), or all of them.
    pub(crate) fn forget(&mut self, chunk: Option<ChunkPos>) {
        match chunk {
            Some(chunk) => {
                let chunk = self.wrapped(chunk);
                self.chunks.remove(&chunk);
                let wild = &self.wild;
                self.blocks.retain(|&block, _| {
                    let (x, y, _, _) = wild.block_bounds(block);
                    wild.chunk_of(x, y) != chunk
                });
            }
            None => {
                self.chunks.clear();
                self.blocks.clear();
            }
        }
    }

    fn wrapped(&self, chunk: ChunkPos) -> ChunkPos {
        ChunkPos {
            x: chunk.x.rem_euclid(self.wild.columns()),
            y: chunk.y,
        }
    }
}
