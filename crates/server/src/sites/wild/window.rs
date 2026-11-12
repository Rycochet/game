//! The window of the wilderness a visit's level is: the block a player is in
//! and those round it, with players' changes and the pois' cutouts in them.

use shared::dungeon::Level;
use shared::movement::{FlatGround, WildHeights};
use shared::poi::ChunkPos;
use shared::protocol::SeenStairs;
use shared::relief::Relief;
use shared::site::{BuildingLook, FieldLook, Rect, SiteLevel, Stairs};
use shared::wilderness::{BLOCK_SIZE, BlockPos, SCALE};
use std::collections::BTreeSet;

use super::ensure_chunk;
use crate::ServerWorld;
use crate::sites::{DoorSite, Mirror, SetIn, VisitDoors, set_doors};
use crate::storage::SiteId;
use crate::wilderness::WINDOW_BLOCKS;

/// A window of the wilderness, in the player's frame (see `wild`).
pub(super) struct Window {
    pub(super) centre: BlockPos,
    pub(super) level: SiteLevel,
    pub(super) stairs: Vec<SeenStairs>,
    pub(super) mirrors: Vec<Mirror>,
    pub(super) cutouts: Vec<SetIn>,
    /// Town cutouts' buildings and farm cutouts' fields, as players see them.
    pub(super) buildings: Vec<BuildingLook>,
    pub(super) fields: Vec<FieldLook>,
    /// The doors of its cutouts, as they are now.
    pub(super) doors: VisitDoors,
}

/// The window round a player at `(x, y)`.
pub(super) fn window(world: &mut ServerWorld, at: (i64, i64)) -> Window {
    window_of(world, at, WINDOW_BLOCKS)
}

/// The cells of the block a player at `(x, y)` is in and `blocks` round it,
/// in their frame: a live area's level, for its mobs to find their way on,
/// and its ground's shape (as the client draws it: `relief::Relief::of_land`
/// with the cutouts' flats and stairs).
pub(crate) fn area_level(world: &mut ServerWorld, at: (i64, i64), blocks: i64) -> (Level, Relief) {
    let window = window_of(world, at, blocks);
    let flats: Vec<FlatGround> = (window.cutouts.iter())
        .flat_map(|set_in| set_in.flats.iter().copied())
        .collect();
    let stairs: Vec<_> = window.stairs.iter().map(|s| (s.x, s.y)).collect();
    let level = window.level.level;
    let bounds = level.bounds();
    let heights = WildHeights::new(&world.ground, bounds).with_flats(&flats, &level);
    let relief = Relief::of_land(&heights, bounds, (0, 0), &stairs);
    (level, relief)
}

/// The window round a player at `(x, y)`, `blocks` each side of theirs.
fn window_of(world: &mut ServerWorld, (x, y): (i64, i64), blocks: i64) -> Window {
    let wild = &world.wilderness.wild;
    let width = wild.width();
    let centre = wild.block_of(x, y);
    let (bx, by, columns, rows) = wild.block_bounds(centre);
    let reach = blocks * BLOCK_SIZE;
    let mut frame = Frame {
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
    frame.bounds = with_towns(world, &frame, &chunks);
    let (min_x, min_y, columns, rows) = frame.bounds;
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
        doors: VisitDoors::new(),
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
    add_doors(world, &mut window);
    window
}

/// The doors of the pois set into the window: their sites' own.
fn add_doors(world: &mut ServerWorld, window: &mut Window) {
    for mirror in window.mirrors.clone() {
        let SiteId::Poi(chunk) = mirror.id else {
            continue;
        };
        let cutout = world.wilderness.chunk(chunk).and_then(|c| c.plan.cutout());
        let Some(poi) = cutout.map(|cutout| *cutout.poi()) else {
            continue;
        };
        // Its locks, if it is the first time anyone has come near.
        let placed = world.site(mirror.id).levels.get(&mirror.index);
        if !placed.is_some_and(|level| level.locks_placed)
            && let Some(site) = crate::sites::enter::make_site(world, &poi, mirror.id)
        {
            let first = site.level(mirror.index);
            crate::sites::ensure_locks(world, (mirror.id, &poi), mirror.index, &first);
        }
        let site = DoorSite {
            id: mirror.id,
            index: mirror.index,
            poi: &poi,
        };
        let offset = (-mirror.offset.0, -mirror.offset.1);
        let level = &mut window.level.level;
        let doors = set_doors(world, &site, level, mirror.bounds, offset);
        window.doors.extend(doors);
    }
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

/// How far along a cutout at `bounds` round the planet is in the window's
/// frame, if it lies in the window at all. Cutouts are kept inside their
/// chunk, which never lies across the seam.
fn turn_of(frame: &Frame, (min_x, min_y, columns, rows): (i64, i64, i64, i64)) -> Option<i64> {
    let (wx, wy, wc, wr) = frame.bounds;
    [-frame.width, 0, frame.width]
        .into_iter()
        .map(|turn| turn + frame.shift)
        .find(|&turn| {
            min_x + turn < wx + wc
                && min_x + columns + turn > wx
                && min_y < wy + wr
                && min_y + rows > wy
        })
}

/// The window's bounds grown to take in the whole of any town set in that
/// lies partly in it, so all of it can be sent (towns are known whole) and
/// kept by the client while it is near.
fn with_towns(world: &ServerWorld, frame: &Frame, chunks: &[ChunkPos]) -> (i64, i64, i64, i64) {
    let (mut min_x, mut min_y, columns, rows) = frame.bounds;
    let (mut max_x, mut max_y) = (min_x + columns, min_y + rows);
    for &chunk in chunks {
        let Some(wild_chunk) = world.wilderness.chunk(chunk) else {
            continue;
        };
        let Some(cutout) = wild_chunk.plan.cutout() else {
            continue;
        };
        let bounds = cutout.level().bounds();
        let Some(turn) = turn_of(frame, bounds).filter(|_| !wild_chunk.town.is_empty()) else {
            continue;
        };
        let (x, y, town_columns, town_rows) = bounds;
        (min_x, min_y) = (min_x.min(x + turn), min_y.min(y));
        (max_x, max_y) = (max_x.max(x + turn + town_columns), max_y.max(y + town_rows));
    }
    (min_x, min_y, max_x - min_x, max_y - min_y)
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
    let Some(turn) = turn_of(frame, cutout.level().bounds()) else {
        return;
    };
    let bounds = (min_x + turn, min_y, columns, rows);
    // Its way in: where its own visit starts, or the way down.
    let entrance = match cutout.offset() {
        Some(_) => wild_chunk.entrance,
        None => cutout.stairs().first().map(|stairs| (stairs.x, stairs.y)),
    };
    window.cutouts.push(SetIn {
        chunk,
        kind: cutout.poi().kind,
        bounds,
        entrance: entrance.map(|(x, y)| (x + turn, y)),
        border: wild_chunk
            .border
            .iter()
            .map(|&(x, y)| (x + turn, y))
            .collect(),
        flats: wild_chunk
            .flats
            .iter()
            .map(|flat| {
                let (x, y, width, height) = flat.bounds;
                FlatGround {
                    bounds: (x + turn, y, width, height),
                    ..*flat
                }
            })
            .collect(),
    });
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
    // The site's own changes, if anyone has made any (its save opened if it
    // isn't yet, as after a restart).
    let changes = world.site(id).levels.get(&0);
    for (&(sx, sy), &cell) in changes.iter().flat_map(|level| &level.changes) {
        window.level.level.set(sx + ox + turn, sy + oy, cell);
    }
}
