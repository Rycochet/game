//! The window of the wilderness a visit's level is: the block a player is in
//! and those round it, with players' changes and the pois' cutouts in them.

use shared::dungeon::Level;
use shared::poi::ChunkPos;
use shared::protocol::SeenStairs;
use shared::site::{BuildingLook, FieldLook, Rect, SiteLevel, Stairs};
use shared::wilderness::{BLOCK_SIZE, BlockPos, SCALE};
use std::collections::BTreeSet;

use super::ensure_chunk;
use crate::ServerWorld;
use crate::sites::Mirror;
use crate::storage::SiteId;
use crate::wilderness::WINDOW_BLOCKS;

/// A window of the wilderness, in the player's frame (see `wild`).
pub(super) struct Window {
    pub(super) centre: BlockPos,
    pub(super) level: SiteLevel,
    pub(super) stairs: Vec<SeenStairs>,
    pub(super) mirrors: Vec<Mirror>,
    pub(super) cutouts: Vec<(ChunkPos, (i64, i64, i64, i64))>,
    /// Town cutouts' buildings and farm cutouts' fields, as players see them.
    pub(super) buildings: Vec<BuildingLook>,
    pub(super) fields: Vec<FieldLook>,
}

/// The window round a player at `(x, y)`.
pub(super) fn window(world: &mut ServerWorld, (x, y): (i64, i64)) -> Window {
    let wild = &world.wilderness.wild;
    let width = wild.width();
    let centre = wild.block_of(x, y);
    let (bx, by, columns, rows) = wild.block_bounds(centre);
    let reach = WINDOW_BLOCKS * BLOCK_SIZE;
    let frame = Frame {
        // The player's x less where it is round the planet.
        shift: x - wild.wrap(x),
        width,
        bounds: (
            bx + x - wild.wrap(x) - reach,
            by - reach,
            columns + 2 * reach,
            rows + 2 * reach,
        ),
    };
    let (min_x, min_y, columns, rows) = frame.bounds;
    let mut chunks = BTreeSet::new();
    for py in (min_y..min_y + rows).step_by(SCALE as usize) {
        for px in (min_x..min_x + columns).step_by(SCALE as usize) {
            chunks.insert(wild.chunk_of(px, py));
        }
        chunks.insert(wild.chunk_of(min_x + columns - 1, py));
    }
    let chunks: Vec<ChunkPos> = chunks.into_iter().collect();
    // Twice: making room for one may let go of those before it.
    for &chunk in chunks.iter().chain(&chunks) {
        ensure_chunk(world, chunk);
    }
    let mut level = Level::new(min_x, min_y, columns, rows);
    for cy in min_y..min_y + rows {
        for cx in min_x..min_x + columns {
            let wrapped = cx.rem_euclid(width);
            let block = world.wilderness.wild.block_of(wrapped, cy);
            let made = world.wilderness.block(block);
            level.set(cx, cy, made.cell(wrapped, cy));
            level.set_room(cx, cy, made.room(wrapped, cy));
        }
    }
    let mut window = Window {
        centre,
        level: SiteLevel {
            level,
            stairs: Vec::new(),
            entrance: None,
            buildings: Vec::new(),
            fields: Vec::new(),
        },
        stairs: Vec::new(),
        mirrors: Vec::new(),
        cutouts: Vec::new(),
        buildings: Vec::new(),
        fields: Vec::new(),
    };
    for chunk in chunks {
        add_cutout(world, &frame, chunk, &mut window);
    }
    let changes = world.site(SiteId::Wilderness).levels.get(&0);
    for (&(cx, cy), &cell) in changes.iter().flat_map(|level| &level.changes) {
        if let Some(cx) = frame.place(cx, cy) {
            window.level.level.set(cx, cy, cell);
        }
    }
    window
}

/// Where the player is in the planet, and the window's cells.
struct Frame {
    shift: i64,
    width: i64,
    bounds: (i64, i64, i64, i64),
}

impl Frame {
    /// Where a cell round the planet is in the window, if it is in it.
    fn place(&self, x: i64, y: i64) -> Option<i64> {
        let (min_x, min_y, columns, rows) = self.bounds;
        if !(min_y..min_y + rows).contains(&y) {
            return None;
        }
        [-self.width, 0, self.width]
            .into_iter()
            .map(|turn| x + self.shift + turn)
            .find(|x| (min_x..min_x + columns).contains(x))
    }
}

/// A chunk's cutout in the window: its stairs, the poi it shows, and its
/// site's first level's changes, kept for both.
fn add_cutout(world: &mut ServerWorld, frame: &Frame, chunk: ChunkPos, window: &mut Window) {
    let Some(wild_chunk) = world.wilderness.chunk(chunk) else {
        return;
    };
    let Some(cutout) = wild_chunk.plan.cutout() else {
        return;
    };
    let (min_x, min_y, columns, rows) = cutout.level().bounds();
    // Cutouts are kept inside their chunk, which never lies across the seam.
    let turn = [-frame.width, 0, frame.width]
        .into_iter()
        .map(|turn| turn + frame.shift)
        .find(|&turn| {
            let (wx, wy, wc, wr) = frame.bounds;
            min_x + turn < wx + wc
                && min_x + columns + turn > wx
                && min_y < wy + wr
                && min_y + rows > wy
        });
    let Some(turn) = turn else {
        return;
    };
    let bounds = (min_x + turn, min_y, columns, rows);
    window.cutouts.push((chunk, bounds));
    for stairs in &wild_chunk.stairs {
        let x = stairs.x + turn;
        window.stairs.push(SeenStairs { x, ..*stairs });
    }
    for stairs in cutout.stairs() {
        let x = stairs.x + turn;
        window.level.stairs.push(Stairs { x, ..*stairs });
    }
    let turned = |rect: Rect| Rect {
        x: rect.x + turn,
        ..rect
    };
    for building in &wild_chunk.buildings {
        window.buildings.push(BuildingLook {
            rects: building.rects.iter().copied().map(turned).collect(),
            door: (building.door.0 + turn, building.door.1),
            ..building.clone()
        });
    }
    for field in &wild_chunk.fields {
        window.fields.push(FieldLook {
            rect: turned(field.rect),
            ..*field
        });
    }
    let Some((ox, oy)) = cutout.offset() else {
        return;
    };
    let id = SiteId::Poi(chunk);
    window.mirrors.push(Mirror {
        id,
        index: 0,
        offset: (-ox - turn, -oy),
        bounds,
    });
    // The site's own changes, if anyone has made any.
    let changes = world.sites.get(&id).and_then(|site| site.levels.get(&0));
    for (&(sx, sy), &cell) in changes.iter().flat_map(|level| &level.changes) {
        window.level.level.set(sx + ox + turn, sy + oy, cell);
    }
}
