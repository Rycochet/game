//! Caves (and dug dungeons such as crypts): irregular caves joined by
//! winding passages, one or more levels deep.
//!
//! Each cave is dug by a random walk inside an ellipse ("Random Walk Cave
//! Generation" on RogueBasin), smoothed once with the cellular automata rule
//! ("Cellular Automata Method for Generating Random Cave-Like Levels"), so
//! no cave is a rectangle. Passages join each cave to the nearest one
//! already joined (a spanning tree), plus a few more for loops; they run in
//! short straight stretches that turn as they go. Pools of water or lava are
//! only kept where everything can still be reached around them.
//!
//! The way in is a hole with stairs down from the world map, or a cave mouth:
//! a strip of outside ground (as wide as the mouth, not the cave) on the
//! south edge, sometimes another on the east or west. Every level's size is
//! its own, unrelated to the levels above and below.

use serde::{Deserialize, Serialize};

use super::grid::{self, Reach, STEPS, chance, roll};
use super::{Exit, SiteLevel, Stairs, StairsDirection};
use crate::dungeon::{Cell, Level, RoomId};
use crate::rng::{Rng, Seed};

/// Depth of the outside ground at a cave mouth, in cells.
const BORDER: i64 = 3;

/// Caves per this many cells of a level (at least 3, at most 24).
const CELLS_PER_CAVE: i64 = 220;

/// Cave radii, each way, in cells.
const MIN_RADIUS: i64 = 3;
const MAX_RADIUS: i64 = 8;

/// Percentage of a cave's ellipse dug before smoothing, and how many times
/// it is smoothed.
const FILL_PERCENT: u32 = 60;
const SMOOTHING_PASSES: u32 = 3;

/// Gap kept between caves' bounding boxes.
const CAVE_GAP: i64 = 3;

/// Passage stretches run this many cells before they may turn.
const MIN_RUN: i64 = 2;
const MAX_RUN: i64 = 6;

/// Percentage of passage stretches that wander sideways.
const WANDER_PERCENT: u32 = 30;

/// Percentage chance of a door where a passage enters a cave (with doors).
const DOOR_PERCENT: u32 = 40;

/// Percentage of doors that become secret, where nothing depends on them.
const SECRET_DOOR_PERCENT: u32 = 10;

/// Percentage chance of a second mouth on the east or west side.
const SECOND_MOUTH_PERCENT: u32 = 35;

/// One floor cell in this many of a natural cave's top level is open to the
/// sky, and how far (counting diagonals as 1) one stays from liquid and stairs.
const CELLS_PER_SKYLIGHT: u64 = 1500;
const SKYLIGHT_CLEARANCE: i64 = 2;

/// Which ways in a cave may have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaveEntrance {
    /// A hole in the ground: stairs down from the world map.
    Hole,
    /// A cave mouth: walk in from outside ground at the edge.
    Opening,
    /// Either, half the time each.
    Either,
}

/// Options for the cave generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaveOptions {
    /// Each level's width and height are picked separately from this
    /// inclusive range.
    pub size: (i64, i64),
    /// How many levels deep, inclusive range.
    pub levels: (u32, u32),
    pub entrance: CaveEntrance,
    /// Dug dungeons have doors where passages enter caves (some secret);
    /// natural caves have none.
    pub doors: bool,
    /// Percentage chance of water pools / a lava pool on each level.
    pub water_percent: u32,
    pub lava_percent: u32,
}

#[derive(Clone, Debug)]
pub struct Plan {
    seed: Seed,
    options: CaveOptions,
    levels: u32,
    opening: bool,
}

/// One cave as planned: centre and radii.
#[derive(Clone, Copy, Debug)]
struct Cave {
    x: i64,
    y: i64,
    rx: i64,
    ry: i64,
}

impl Cave {
    fn contains(&self, x: i64, y: i64) -> bool {
        let (dx, dy) = (x - self.x, y - self.y);
        dx * dx * self.ry * self.ry + dy * dy * self.rx * self.rx
            <= self.rx * self.rx * self.ry * self.ry
    }
}

impl Plan {
    pub fn new(options: &CaveOptions, seed: Seed) -> Self {
        let mut rng = seed.derive("cave").rng();
        let levels = roll(&mut rng, options.levels.0 as i64, options.levels.1 as i64).max(1) as u32;
        let opening = match options.entrance {
            CaveEntrance::Hole => false,
            CaveEntrance::Opening => true,
            CaveEntrance::Either => chance(&mut rng, 50),
        };
        Self {
            seed,
            options: *options,
            levels,
            opening,
        }
    }

    pub fn levels(&self) -> u32 {
        self.levels
    }

    pub fn level(&self, index: u32) -> SiteLevel {
        let mut made = self.without_skylights(index);
        if index == 0 && !self.options.doors {
            let seed = self.seed.derive("skylight");
            open_skylights(&mut made.level, seed, &made.stairs);
        }
        made
    }

    /// The level before any skylights are opened in it.
    pub(super) fn without_skylights(&self, index: u32) -> SiteLevel {
        let mut rng = Seed::new(self.seed.derive("cave level").at(&[index as i64])).rng();
        let width = roll(&mut rng, self.options.size.0, self.options.size.1);
        let height = roll(&mut rng, self.options.size.0, self.options.size.1);
        let mut level = Level::new(0, 0, width, height);
        let opening = index == 0 && self.opening;
        // Keep caves off the edges (and clear of any mouth's ground).
        let margin = if opening { BORDER + 2 } else { 2 };

        let caves = place_caves(&mut rng, width, height, margin);
        for (i, cave) in caves.iter().enumerate() {
            dig_cave(&mut level, &mut rng, cave, RoomId(i as u16));
        }
        // Passages stay off the edges (and clear of any mouth's ground).
        let inner = if opening {
            (1 + BORDER, 1, width - 2 - BORDER, height - 2 - BORDER)
        } else {
            (1, 1, width - 2, height - 2)
        };
        join_caves(&mut level, &mut rng, &caves, inner);

        // The way in.
        let mut stairs = Vec::new();
        let mut entrance = None;
        let arrival = if opening {
            let mouth = dig_mouths(&mut level, &mut rng, &caves, inner);
            entrance = Some(mouth);
            mouth
        } else {
            let cave = caves[rng.range_u64(0, caves.len() as u64) as usize];
            let to = if index == 0 {
                Exit::WorldMap
            } else {
                Exit::Level(index - 1)
            };
            stairs.push(Stairs {
                x: cave.x,
                y: cave.y,
                direction: StairsDirection::Up,
                to,
            });
            if index == 0 {
                entrance = Some((cave.x, cave.y));
            }
            (cave.x, cave.y)
        };

        // Anything not reached yet gets a passage to what is.
        loop {
            let mut reach = Reach::new(&level);
            reach.spread(&level, &[arrival]);
            let Some(from) = reach.first_unreached(&level) else {
                break;
            };
            let to = nearest_reached(&level, &reach, from);
            wind(&mut level, &mut rng, from, to, inner);
            // If the passage couldn't get there, dig straight across.
            let mut reach = Reach::new(&level);
            reach.spread(&level, &[arrival]);
            if !reach.is(from.0, from.1) {
                let edge = (width, height);
                let dug = grid::dig_to_unreached(&mut level, &reach, |x, y| {
                    x > 0 && y > 0 && x < edge.0 - 1 && y < edge.1 - 1
                });
                if !dug {
                    break;
                }
            }
        }

        // Stairs down in the cave furthest from the way in.
        if index + 1 < self.levels {
            let (x, y) = furthest_cave(&level, &caves, arrival);
            stairs.push(Stairs {
                x,
                y,
                direction: StairsDirection::Down,
                to: Exit::Level(index + 1),
            });
        }

        let keep: Vec<(i64, i64)> = stairs.iter().map(|s| (s.x, s.y)).chain([arrival]).collect();
        if self.options.doors {
            add_doors(&mut level, &mut rng, &keep);
        }
        if chance(&mut rng, self.options.water_percent) {
            for _ in 0..roll(&mut rng, 1, 3) {
                add_pool(&mut level, &mut rng, &caves, Cell::Water, &keep, arrival);
            }
        }
        if chance(&mut rng, self.options.lava_percent) {
            add_pool(&mut level, &mut rng, &caves, Cell::Lava, &keep, arrival);
        }
        if self.options.doors {
            grid::hide_some_doors(
                &mut level,
                &mut rng,
                SECRET_DOOR_PERCENT,
                &[arrival],
                |_, _| false,
            );
        }

        SiteLevel {
            level,
            stairs,
            entrance,
            buildings: Vec::new(),
            fields: Vec::new(),
        }
    }
}

/// Very rarely, a floor cell is open to the sky: outside ground, away from
/// liquid and stairs and off the level's edge (so it is no way out). Chosen by
/// a hash of its position, so nothing else about the level changes.
fn open_skylights(level: &mut Level, seed: Seed, stairs: &[Stairs]) {
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if level.cell(x, y) != Cell::Floor
                || !seed.at(&[x, y]).is_multiple_of(CELLS_PER_SKYLIGHT)
            {
                continue;
            }
            let reach = -SKYLIGHT_CLEARANCE..=SKYLIGHT_CLEARANCE;
            let clear = reach.clone().all(|dy| {
                reach.clone().all(|dx| {
                    let (nx, ny) = (x + dx, y + dy);
                    let cell = level.cell(nx, ny);
                    level.contains(nx, ny)
                        && !cell.is_liquid()
                        && !cell.is_outside()
                        && !stairs.iter().any(|s| (s.x, s.y) == (nx, ny))
                })
            });
            if clear {
                level.set(x, y, Cell::Ground);
                level.set_room(x, y, None);
            }
        }
    }
}

/// Picks where the caves go, apart from each other. Always at least one.
fn place_caves(rng: &mut Rng, width: i64, height: i64, margin: i64) -> Vec<Cave> {
    let wanted = (width * height / CELLS_PER_CAVE).clamp(3, 24) as usize;
    let mut caves: Vec<Cave> = Vec::new();
    for _ in 0..wanted * 12 {
        if caves.len() >= wanted {
            break;
        }
        let rx = roll(rng, MIN_RADIUS, MAX_RADIUS);
        let ry = roll(rng, MIN_RADIUS, MAX_RADIUS);
        let (x0, x1) = (margin + rx, width - 1 - margin - rx);
        let (y0, y1) = (margin + ry, height - 1 - margin - ry);
        if x1 < x0 || y1 < y0 {
            continue;
        }
        let cave = Cave {
            x: roll(rng, x0, x1),
            y: roll(rng, y0, y1),
            rx,
            ry,
        };
        let clear = caves.iter().all(|other| {
            (cave.x - other.x).abs() > cave.rx + other.rx + CAVE_GAP
                || (cave.y - other.y).abs() > cave.ry + other.ry + CAVE_GAP
        });
        if clear {
            caves.push(cave);
        }
    }
    if caves.is_empty() {
        caves.push(Cave {
            x: width / 2,
            y: height / 2,
            rx: MIN_RADIUS,
            ry: MIN_RADIUS,
        });
    }
    caves
}

/// Digs one cave: its ellipse filled at random, plus a random walk from the
/// centre (so the middle joins up), smoothed by the cellular automata rule,
/// keeping only what joins up with the centre. Never a plain rectangle.
fn dig_cave(level: &mut Level, rng: &mut Rng, cave: &Cave, room: RoomId) {
    let (x0, y0) = (cave.x - cave.rx - 1, cave.y - cave.ry - 1);
    let (w, h) = (2 * cave.rx + 3, 2 * cave.ry + 3);
    let index = |x: i64, y: i64| ((y - y0) * w + (x - x0)) as usize;
    let inside = |x: i64, y: i64| x >= x0 && y >= y0 && x < x0 + w && y < y0 + h;
    let mut dug = vec![false; (w * h) as usize];

    for cy in y0..y0 + h {
        for cx in x0..x0 + w {
            dug[index(cx, cy)] = cave.contains(cx, cy) && chance(rng, FILL_PERCENT);
        }
    }
    let (mut x, mut y) = (cave.x, cave.y);
    dug[index(x, y)] = true;
    for _ in 0..cave.rx * cave.ry * 2 {
        let (dx, dy) = STEPS[rng.range_u64(0, 4) as usize];
        if cave.contains(x + dx, y + dy) {
            (x, y) = (x + dx, y + dy);
            dug[index(x, y)] = true;
        }
    }

    // Cellular automata: dug where most of the eight neighbours are.
    let count = |dug: &[bool], x: i64, y: i64| {
        let mut n = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if (dx, dy) != (0, 0) && inside(x + dx, y + dy) && dug[index(x + dx, y + dy)] {
                    n += 1;
                }
            }
        }
        n
    };
    for _ in 0..SMOOTHING_PASSES {
        let before = dug.clone();
        for cy in y0..y0 + h {
            for cx in x0..x0 + w {
                let n = count(&before, cx, cy);
                let i = index(cx, cy);
                dug[i] = cave.contains(cx, cy) && (n >= 5 || (before[i] && n >= 4));
            }
        }
    }
    dug[index(cave.x, cave.y)] = true;

    // Keep only what joins up with the centre.
    let mut keep = vec![false; dug.len()];
    let mut stack = vec![(cave.x, cave.y)];
    keep[index(cave.x, cave.y)] = true;
    while let Some((cx, cy)) = stack.pop() {
        for (dx, dy) in STEPS {
            let (nx, ny) = (cx + dx, cy + dy);
            if inside(nx, ny) && dug[index(nx, ny)] && !keep[index(nx, ny)] {
                keep[index(nx, ny)] = true;
                stack.push((nx, ny));
            }
        }
    }

    // Never a rectangle: if it fills its bounding box, take a corner off.
    let cells: Vec<(i64, i64)> = (y0..y0 + h)
        .flat_map(|cy| (x0..x0 + w).map(move |cx| (cx, cy)))
        .filter(|&(cx, cy)| keep[index(cx, cy)])
        .collect();
    let (min_x, max_x) = (
        cells.iter().map(|c| c.0).min(),
        cells.iter().map(|c| c.0).max(),
    );
    let (min_y, max_y) = (
        cells.iter().map(|c| c.1).min(),
        cells.iter().map(|c| c.1).max(),
    );
    if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (min_x, max_x, min_y, max_y)
        && (max_x - min_x + 1) * (max_y - min_y + 1) == cells.len() as i64
    {
        if max_x > min_x && max_y > min_y {
            // A corner away from the centre (a full rectangle stays joined up
            // without one).
            let corner = [
                (min_x, min_y),
                (max_x, min_y),
                (min_x, max_y),
                (max_x, max_y),
            ]
            .into_iter()
            .find(|&c| c != (cave.x, cave.y))
            .expect("four corners");
            keep[index(corner.0, corner.1)] = false;
        } else {
            // A line or a single cell: add cells to make an L.
            keep[index(min_x + 1, min_y)] = true;
            keep[index(min_x, max_y + 1)] = true;
        }
    }

    for cy in y0..y0 + h {
        for cx in x0..x0 + w {
            if keep[index(cx, cy)] && level.cell(cx, cy) == Cell::Wall {
                level.set(cx, cy, Cell::Floor);
                level.set_room(cx, cy, Some(room));
            }
        }
    }
}

/// Joins every cave to the nearest one already joined, then adds a few
/// more passages for loops.
fn join_caves(level: &mut Level, rng: &mut Rng, caves: &[Cave], bounds: (i64, i64, i64, i64)) {
    let distance = |a: &Cave, b: &Cave| (a.x - b.x).pow(2) + (a.y - b.y).pow(2);
    let mut joined = vec![false; caves.len()];
    joined[0] = true;
    for _ in 1..caves.len() {
        let mut best: Option<(i64, usize, usize)> = None;
        for (a, cave_a) in caves.iter().enumerate().filter(|(a, _)| joined[*a]) {
            for (b, cave_b) in caves.iter().enumerate().filter(|(b, _)| !joined[*b]) {
                let d = distance(cave_a, cave_b);
                if best.is_none_or(|(best_d, _, _)| d < best_d) {
                    best = Some((d, a, b));
                }
            }
        }
        let Some((_, a, b)) = best else { break };
        joined[b] = true;
        wind(
            level,
            rng,
            (caves[a].x, caves[a].y),
            (caves[b].x, caves[b].y),
            bounds,
        );
    }
    // Loops: some caves get a passage to their second nearest.
    for (a, cave_a) in caves.iter().enumerate() {
        if caves.len() < 3 || !chance(rng, 25) {
            continue;
        }
        let mut others: Vec<(i64, usize)> = caves
            .iter()
            .enumerate()
            .filter(|(b, _)| *b != a)
            .map(|(b, cave_b)| (distance(cave_a, cave_b), b))
            .collect();
        others.sort();
        let b = others[1].1;
        wind(
            level,
            rng,
            (cave_a.x, cave_a.y),
            (caves[b].x, caves[b].y),
            bounds,
        );
    }
}

/// Digs a winding passage from `from` to `to`: short straight stretches,
/// mostly towards the target, sometimes wandering sideways. Stays within
/// `bounds` (`min_x, min_y, max_x, max_y`, inclusive). New cells are floor
/// in no room.
fn wind(
    level: &mut Level,
    rng: &mut Rng,
    from: (i64, i64),
    to: (i64, i64),
    bounds: (i64, i64, i64, i64),
) {
    let (min_x, min_y, max_x, max_y) = bounds;
    let dig = |level: &mut Level, x: i64, y: i64| {
        if level.cell(x, y) == Cell::Wall {
            level.set(x, y, Cell::Floor);
            level.set_room(x, y, None);
        }
    };
    let (mut x, mut y) = from;
    dig(level, x, y);
    let most_runs = 4 * ((to.0 - x).abs() + (to.1 - y).abs()) / MIN_RUN + 8;
    // Gives up after twice the expected runs (if the target is out of bounds).
    for run in 0..most_runs * 2 {
        if (x, y) == to {
            return;
        }
        let (gx, gy) = (to.0 - x, to.1 - y);
        let wander = run < most_runs && chance(rng, WANDER_PERCENT);
        // Towards the target along one axis (the further, more often), or
        // sideways off it.
        let along_x = if gx == 0 {
            false
        } else if gy == 0 {
            true
        } else {
            rng.range_u64(0, gx.unsigned_abs() + gy.unsigned_abs()) < gx.unsigned_abs()
        };
        let (dx, dy, length) = if wander {
            let sign = if chance(rng, 50) { 1 } else { -1 };
            let length = roll(rng, MIN_RUN, MAX_RUN / 2);
            if along_x {
                (0, sign, length)
            } else {
                (sign, 0, length)
            }
        } else if along_x {
            let length = roll(rng, MIN_RUN, MAX_RUN).min(gx.abs());
            (
                gx.signum(),
                0,
                if run < most_runs { length } else { gx.abs() },
            )
        } else {
            let length = roll(rng, MIN_RUN, MAX_RUN).min(gy.abs());
            (
                0,
                gy.signum(),
                if run < most_runs { length } else { gy.abs() },
            )
        };
        for _ in 0..length {
            let (nx, ny) = (x + dx, y + dy);
            if nx < min_x || nx > max_x || ny < min_y || ny > max_y {
                break;
            }
            (x, y) = (nx, ny);
            dig(level, x, y);
        }
    }
}

/// The reached cell nearest to `from` (straight-line), first in row order on
/// ties.
fn nearest_reached(level: &Level, reach: &Reach, from: (i64, i64)) -> (i64, i64) {
    let (min_x, min_y, width, height) = level.bounds();
    let mut best: Option<(i64, (i64, i64))> = None;
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if reach.is(x, y) {
                let d = (x - from.0).pow(2) + (y - from.1).pow(2);
                if best.is_none_or(|(best_d, _)| d < best_d) {
                    best = Some((d, (x, y)));
                }
            }
        }
    }
    best.expect("something reached").1
}

/// Digs the cave mouths: outside ground on the south edge (and sometimes the
/// east or west), as wide as the mouth, with a passage in. Returns where you
/// arrive from the world map: the middle of the south ground.
fn dig_mouths(
    level: &mut Level,
    rng: &mut Rng,
    caves: &[Cave],
    bounds: (i64, i64, i64, i64),
) -> (i64, i64) {
    let (_, _, width, height) = level.bounds();
    let mouth = |level: &mut Level, rng: &mut Rng, south: bool, east: bool| {
        let span = roll(rng, 4, 8);
        // Ground: `x0..x0 + w`, `y0..y0 + h`; the passage starts just inside.
        let (x0, y0, w, h, start) = if south {
            let x0 = roll(rng, 2, width - 2 - span);
            (
                x0,
                height - BORDER,
                span,
                BORDER,
                (x0 + span / 2, height - BORDER - 1),
            )
        } else {
            let y0 = roll(rng, height / 2, height - 2 - span);
            let x0 = if east { width - BORDER } else { 0 };
            let inside_x = if east { x0 - 1 } else { BORDER };
            (x0, y0, BORDER, span, (inside_x, y0 + span / 2))
        };
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                level.set(x, y, Cell::Ground);
                level.set_room(x, y, None);
            }
        }
        // A passage two wide into the nearest cave.
        let nearest = caves
            .iter()
            .min_by_key(|c| (c.x - start.0).pow(2) + (c.y - start.1).pow(2))
            .expect("a cave");
        let side = if south { (1, 0) } else { (0, 1) };
        wind(level, rng, start, (nearest.x, nearest.y), bounds);
        wind(
            level,
            rng,
            (start.0 + side.0, start.1 + side.1),
            (nearest.x, nearest.y),
            bounds,
        );
        (x0 + w / 2, y0 + h / 2)
    };
    let arrival = mouth(level, rng, true, false);
    if chance(rng, SECOND_MOUTH_PERCENT) {
        let east = chance(rng, 50);
        mouth(level, rng, false, east);
    }
    arrival
}

/// The centre of the cave furthest (by walking) from `from`.
fn furthest_cave(level: &Level, caves: &[Cave], from: (i64, i64)) -> (i64, i64) {
    let (min_x, min_y, width, height) = level.bounds();
    let index = |x: i64, y: i64| ((y - min_y) * width + (x - min_x)) as usize;
    let mut steps: Vec<Option<u32>> = vec![None; (width * height) as usize];
    let mut queue = std::collections::VecDeque::from([from]);
    steps[index(from.0, from.1)] = Some(0);
    while let Some((x, y)) = queue.pop_front() {
        let here = steps[index(x, y)].expect("visited");
        for (dx, dy) in STEPS {
            let (nx, ny) = (x + dx, y + dy);
            if level.is_walkable(nx, ny) && steps[index(nx, ny)].is_none() {
                steps[index(nx, ny)] = Some(here + 1);
                queue.push_back((nx, ny));
            }
        }
    }
    let other_caves = caves
        .iter()
        .filter(|c| (c.x, c.y) != from)
        .max_by_key(|c| steps[index(c.x, c.y)].unwrap_or(0))
        .map(|c| (c.x, c.y));
    // With only one cave, the furthest floor cell (first in row order).
    other_caves.unwrap_or_else(|| {
        let mut best = (0, from);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if let Some(n) = steps[index(x, y)]
                    && n > best.0
                    && level.cell(x, y) == Cell::Floor
                {
                    best = (n, (x, y));
                }
            }
        }
        best.1
    })
}

/// Doors where a passage enters a cave through a gap one cell wide.
fn add_doors(level: &mut Level, rng: &mut Rng, keep: &[(i64, i64)]) {
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if level.cell(x, y) != Cell::Floor
                || level.room(x, y).is_some()
                || keep.contains(&(x, y))
            {
                continue;
            }
            // A doorway: walls on two opposite sides, a cave cell on one of
            // the others and a passage on the last.
            let doorway = [(1, 0), (0, 1)].into_iter().any(|(dx, dy)| {
                let walls = level.cell(x + dy, y + dx) == Cell::Wall
                    && level.cell(x - dy, y - dx) == Cell::Wall;
                let a = (x + dx, y + dy);
                let b = (x - dx, y - dy);
                let ends = |p: (i64, i64), q: (i64, i64)| {
                    level.room(p.0, p.1).is_some()
                        && level.cell(q.0, q.1) == Cell::Floor
                        && level.room(q.0, q.1).is_none()
                };
                walls && (ends(a, b) || ends(b, a))
            });
            if doorway && !grid::door_nearby(level, x, y) && chance(rng, DOOR_PERCENT) {
                level.set(x, y, Cell::Door);
            }
        }
    }
}

/// A small pool of `liquid` in a random cave, kept only if everything can
/// still be reached and it stays clear of the stairs and the way in.
fn add_pool(
    level: &mut Level,
    rng: &mut Rng,
    caves: &[Cave],
    liquid: Cell,
    keep: &[(i64, i64)],
    arrival: (i64, i64),
) {
    let room = rng.range_u64(0, caves.len() as u64) as usize;
    let cave = caves[room];
    let clear = |x: i64, y: i64| {
        keep.iter()
            .all(|&(kx, ky)| (kx - x).abs() > 1 || (ky - y).abs() > 1)
    };
    let mut changed = Vec::new();
    let (mut x, mut y) = (
        cave.x + roll(rng, -cave.rx / 2, cave.rx / 2),
        cave.y + roll(rng, -cave.ry / 2, cave.ry / 2),
    );
    for _ in 0..roll(rng, 4, 14) {
        if level.cell(x, y) == Cell::Floor
            && level.room(x, y) == Some(RoomId(room as u16))
            && clear(x, y)
        {
            level.set(x, y, liquid);
            changed.push((x, y));
        }
        let (dx, dy) = STEPS[rng.range_u64(0, 4) as usize];
        if level.room(x + dx, y + dy) == Some(RoomId(room as u16)) {
            (x, y) = (x + dx, y + dy);
        }
    }
    if !grid::all_reached(level, &[arrival]) {
        for (x, y) in changed {
            level.set(x, y, Cell::Floor);
        }
    }
}
