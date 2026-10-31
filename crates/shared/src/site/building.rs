//! Buildings: an outline (rectangle, cut or rounded corners, circle) filled
//! with rooms and corridors by binary space partitioning (after RogueBasin's
//! "Basic BSP Dungeon generation"), one or more floors high.
//!
//! The space inside the outer wall is split again and again by one-cell
//! walls (or, near the top of the tree, by corridors with a wall each side);
//! each final part is a room that fills it completely, so rooms reach out to
//! their walls and no wall inside is more than a few cells thick. Rooms in a
//! corner cut by the outline follow the cut, keeping a wall round them.
//! Doors are then added until everything can be reached from the way in.
//!
//! Every floor has the same outline, the same size or smaller (shrunk the
//! same amount on every side, so its walls stand on the floor below). Stairs
//! between two floors are on the same cell of both, chosen before either is
//! generated, and the splits avoid them, so each floor can be generated on
//! its own.

use serde::{Deserialize, Serialize};

use super::grid::{self, Reach, STEPS, chance, roll};
use super::{Exit, SiteLevel, Stairs, StairsDirection};
use crate::dungeon::{Cell, Level, RoomId};
use crate::rng::{Rng, Seed};

/// Outside ground round the ground floor, in cells.
const BORDER: i64 = 3;

/// The smallest a room can be across, and in cells.
const MIN_ROOM: i64 = 3;
const MIN_ROOM_CELLS: i64 = 6;

/// Parts no bigger than this both ways may stop being split.
const MAX_ROOM: i64 = 9;
/// Percentage chance that a part small enough to be a room stops there.
const STOP_PERCENT: u32 = 40;

/// Splits along at least this many cells may be corridors, near the top of
/// the tree (depth below `CORRIDOR_DEPTH`), with this percentage chance.
const CORRIDOR_MIN_SPAN: i64 = 10;
const CORRIDOR_DEPTH: u32 = 3;
const CORRIDOR_PERCENT: u32 = 60;
/// Corridors along at least this many cells may be two wide.
const WIDE_CORRIDOR_SPAN: i64 = 24;

/// A floor narrower than this isn't built.
const MIN_FLOOR_SIZE: i64 = 9;

/// Percentage chance that a floor is the same size as the one below.
const SAME_SIZE_PERCENT: u64 = 50;

/// Up to one extra outside door per this many cells of width plus height
/// (at most 3), each at least `MIN_ENTRANCE_GAP` cells (walking along the
/// grid) from the others.
const CELLS_PER_EXTRA_ENTRANCE: i64 = 25;
const MIN_ENTRANCE_GAP: i64 = 6;

/// Extra doors (making loops) per this many rooms.
const ROOMS_PER_EXTRA_DOOR: u16 = 6;

/// Percentage of doors that become secret, where nothing depends on them.
const SECRET_DOOR_PERCENT: u32 = 8;

/// A cell's position.
type Spot = (i64, i64);

/// The shape of a building seen from above.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Outline {
    Rectangle,
    /// Corners cut off at 45°.
    CutCorners,
    /// Corners rounded off.
    RoundCorners,
    Circle,
}

/// Options for the building generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildingOptions {
    /// The ground floor's size, outer walls included, as inclusive ranges.
    /// A circle uses `width` for its diameter.
    pub width: (i64, i64),
    pub height: (i64, i64),
    /// Picked from evenly (repeat one to make it likelier).
    pub outlines: &'static [Outline],
    /// Square: the height is the width.
    pub square: bool,
    /// At most this many floors.
    pub max_floors: u32,
    /// Bigger buildings have more floors: up to one per this many cells of
    /// the ground floor's shorter side.
    pub size_per_floor: i64,
}

/// One floor's outline, in level coordinates.
#[derive(Clone, Copy, Debug)]
struct Shape {
    outline: Outline,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
    /// How far corners are cut or rounded.
    corner: i64,
}

impl Shape {
    /// Whether `(x, y)` is inside the outline (outer wall included).
    fn contains(&self, x: i64, y: i64) -> bool {
        let (cx, cy) = (x - self.x, y - self.y);
        if cx < 0 || cy < 0 || cx >= self.width || cy >= self.height {
            return false;
        }
        // Distance in from the nearest side, each way.
        let dx = cx.min(self.width - 1 - cx);
        let dy = cy.min(self.height - 1 - cy);
        let r = self.corner;
        match self.outline {
            Outline::Rectangle => true,
            Outline::CutCorners => dx + dy >= r,
            // Within a quarter circle of radius r in each corner, measured
            // between cell centres in half cells, so it stays in integers.
            Outline::RoundCorners => {
                dx >= r || dy >= r || {
                    let (ax, ay) = (2 * (r - dx) - 1, 2 * (r - dy) - 1);
                    ax * ax + ay * ay <= 4 * r * r
                }
            }
            Outline::Circle => {
                let (ax, ay) = (2 * cx + 1 - self.width, 2 * cy + 1 - self.height);
                ax * ax + ay * ay <= self.width * self.height
            }
        }
    }

    /// Inside the outer wall: in the outline, and so are all eight
    /// neighbours.
    fn is_interior(&self, x: i64, y: i64) -> bool {
        (-1..=1).all(|dy| (-1..=1).all(|dx| self.contains(x + dx, y + dy)))
    }

    /// The same outline `by` cells smaller on every side.
    fn shrunk(&self, by: i64) -> Self {
        Self {
            x: self.x + by,
            y: self.y + by,
            width: self.width - 2 * by,
            height: self.height - 2 * by,
            corner: (self.corner - by).max(1),
            ..*self
        }
    }
}

/// A building's plan: its floors' outlines and the stairs between them.
#[derive(Clone, Debug)]
pub struct Plan {
    seed: Seed,
    /// The level's size: the ground floor and the border round it.
    width: i64,
    height: i64,
    floors: Vec<Shape>,
    /// `stairs[n]` joins floor `n` to floor `n + 1`.
    stairs: Vec<(i64, i64)>,
}

impl Plan {
    pub fn new(options: &BuildingOptions, seed: Seed) -> Self {
        let mut rng = seed.derive("building").rng();
        let outline = options.outlines[rng.range_u64(0, options.outlines.len() as u64) as usize];
        let width = roll(&mut rng, options.width.0, options.width.1);
        let height = if options.square || outline == Outline::Circle {
            width
        } else {
            roll(&mut rng, options.height.0, options.height.1)
        };
        let corner = roll(&mut rng, 2, (width.min(height) / 4).max(2));
        let ground = Shape {
            outline,
            x: BORDER,
            y: BORDER,
            width,
            height,
            corner,
        };

        // Bigger buildings may have more floors.
        let most = (width.min(height) / options.size_per_floor.max(1))
            .clamp(1, options.max_floors.max(1) as i64);
        let wanted = roll(&mut rng, 1, most);
        let mut floors = vec![ground];
        while (floors.len() as i64) < wanted {
            let below = floors[floors.len() - 1];
            // Half the time the same size; otherwise a cell smaller on each
            // side for every 10 in the roll above 50.
            let roll = rng.range_u64(0, 100);
            let by = if roll < SAME_SIZE_PERCENT {
                0
            } else {
                ((roll - SAME_SIZE_PERCENT) / 10 + 1) as i64
            };
            let above = below.shrunk(by);
            if above.width.min(above.height) < MIN_FLOOR_SIZE {
                break;
            }
            floors.push(above);
        }

        // Stairs: on cells well inside both floors (not next to an outer
        // wall), never next to the previous stairs.
        let mut stairs: Vec<(i64, i64)> = Vec::new();
        for pair in floors.windows(2) {
            let above = pair[1];
            let fits = |x: i64, y: i64| {
                (-1..=1).all(|dy| (-1..=1).all(|dx| above.is_interior(x + dx, y + dy)))
                    && stairs
                        .last()
                        .is_none_or(|&(sx, sy)| (sx - x).abs() > 2 || (sy - y).abs() > 2)
            };
            let random = (0..100).find_map(|_| {
                let x = roll(&mut rng, above.x, above.x + above.width - 1);
                let y = roll(&mut rng, above.y, above.y + above.height - 1);
                fits(x, y).then_some((x, y))
            });
            let found = random.or_else(|| {
                (above.y..above.y + above.height)
                    .flat_map(|y| (above.x..above.x + above.width).map(move |x| (x, y)))
                    .find(|&(x, y)| fits(x, y))
            });
            match found {
                Some(spot) => stairs.push(spot),
                None => break,
            }
        }
        floors.truncate(stairs.len() + 1);

        Self {
            seed,
            width: width + 2 * BORDER,
            height: height + 2 * BORDER,
            floors,
            stairs,
        }
    }

    /// The ground floor's size in cells, without the border round it.
    pub fn footprint(&self) -> (i64, i64) {
        (self.floors[0].width, self.floors[0].height)
    }

    /// The shape of the ground floor.
    pub fn outline(&self) -> Outline {
        self.floors[0].outline
    }

    pub fn floors(&self) -> u32 {
        self.floors.len() as u32
    }

    /// Generates floor `index` (0 the ground floor).
    pub fn level(&self, index: u32) -> SiteLevel {
        let n = index as usize;
        let shape = self.floors[n];
        let mut rng = Seed::new(self.seed.derive("building floor").at(&[index as i64])).rng();
        let mut level = Level::new(0, 0, self.width, self.height);

        // Stairs down to the floor below and up to the one above.
        let mut stairs = Vec::new();
        if n > 0 {
            let (x, y) = self.stairs[n - 1];
            stairs.push(Stairs {
                x,
                y,
                direction: StairsDirection::Down,
                to: Exit::Level(index - 1),
            });
        }
        if n + 1 < self.floors.len() {
            let (x, y) = self.stairs[n];
            stairs.push(Stairs {
                x,
                y,
                direction: StairsDirection::Up,
                to: Exit::Level(index + 1),
            });
        }
        let required: Vec<(i64, i64)> = stairs.iter().map(|s| (s.x, s.y)).collect();

        if n == 0 {
            for y in 0..self.height {
                for x in 0..self.width {
                    if !shape.contains(x, y) {
                        level.set(x, y, Cell::Ground);
                    }
                }
            }
        }

        let mut splitter = Splitter {
            level: &mut level,
            shape: &shape,
            required: &required,
            rng: &mut rng,
            rooms: 0,
            corridors: Vec::new(),
        };
        let all = Rect {
            x0: shape.x + 1,
            y0: shape.y + 1,
            x1: shape.x + shape.width - 1,
            y1: shape.y + shape.height - 1,
        };
        splitter.split(all, 0);
        let rooms = splitter.rooms;
        let corridors = std::mem::take(&mut splitter.corridors);
        join_corridors(&mut level, &shape, &corridors);

        // The way in: doors in the outer wall on the ground floor.
        let mut entrances = Vec::new();
        let mut entrance = None;
        if n == 0 {
            entrances = add_entrances(&mut level, &shape, &mut rng);
            entrance = entrances.first().map(|&(_, outside)| outside);
        }
        let start = entrance
            .or_else(|| required.first().copied())
            .expect("a way in: an entrance or stairs");

        add_doors(&mut level, &shape, &mut rng, start);
        add_extra_doors(&mut level, &shape, &mut rng, rooms / ROOMS_PER_EXTRA_DOOR);
        let doors: Vec<(i64, i64)> = entrances.iter().map(|&(door, _)| door).collect();
        grid::hide_some_doors(
            &mut level,
            &mut rng,
            SECRET_DOOR_PERCENT,
            &[start],
            |x, y| doors.contains(&(x, y)),
        );

        SiteLevel {
            level,
            stairs,
            entrance,
            buildings: Vec::new(),
        }
    }
}

/// A rectangle of cells, `x1` and `y1` excluded.
#[derive(Clone, Copy, Debug)]
struct Rect {
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
}

/// A corridor made by a split: its floor cells, and whether it runs north to
/// south.
#[derive(Clone, Copy, Debug)]
struct Corridor {
    floor: Rect,
    north_south: bool,
}

/// The binary space partitioning of one floor.
struct Splitter<'a> {
    level: &'a mut Level,
    shape: &'a Shape,
    /// Cells that must stay floor (stairs): no wall may be built on them.
    required: &'a [(i64, i64)],
    rng: &'a mut Rng,
    rooms: u16,
    corridors: Vec<Corridor>,
}

impl Splitter<'_> {
    /// Interior cells in a rectangle.
    fn interior_cells(&self, rect: Rect) -> i64 {
        let mut count = 0;
        for y in rect.y0..rect.y1 {
            for x in rect.x0..rect.x1 {
                if self.shape.is_interior(x, y) {
                    count += 1;
                }
            }
        }
        count
    }

    fn split(&mut self, rect: Rect, depth: u32) {
        let (w, h) = (rect.x1 - rect.x0, rect.y1 - rect.y0);
        let small = w <= MAX_ROOM && h <= MAX_ROOM;
        if !(small && chance(self.rng, STOP_PERCENT)) && self.try_split(rect, depth, w, h) {
            return;
        }
        self.make_room(rect);
    }

    /// Splits `rect` in two (with a wall or a corridor) and goes on into each
    /// half. False if it can't be split.
    fn try_split(&mut self, rect: Rect, depth: u32, w: i64, h: i64) -> bool {
        // Across the longer side, unless it is roughly square.
        let first_vertical = if w * 4 > h * 5 {
            true
        } else if h * 4 > w * 5 {
            false
        } else {
            chance(self.rng, 50)
        };
        for vertical in [first_vertical, !first_vertical] {
            // `vertical`: a north–south wall, splitting east from west.
            let (across, along) = if vertical { (w, h) } else { (h, w) };
            let corridor = along >= CORRIDOR_MIN_SPAN
                && depth < CORRIDOR_DEPTH
                && chance(self.rng, CORRIDOR_PERCENT);
            let corridor_width = if !corridor {
                0
            } else if along >= WIDE_CORRIDOR_SPAN && chance(self.rng, 50) {
                2
            } else {
                1
            };
            // Wall, corridor floor, wall; or just a wall.
            let thickness = if corridor { corridor_width + 2 } else { 1 };
            if across < 2 * MIN_ROOM + thickness {
                continue;
            }
            let start = if vertical { rect.x0 } else { rect.y0 };
            for _ in 0..10 {
                let at = roll(
                    self.rng,
                    start + MIN_ROOM,
                    start + across - MIN_ROOM - thickness,
                );
                if self.place(rect, vertical, at, thickness, depth) {
                    return true;
                }
            }
        }
        false
    }

    /// Splits at `at` (the first wall row or column) if the stairs aren't
    /// on a wall and both halves are big enough.
    fn place(&mut self, rect: Rect, vertical: bool, at: i64, thickness: i64, depth: u32) -> bool {
        let walls = if thickness == 1 {
            vec![at]
        } else {
            vec![at, at + thickness - 1]
        };
        let on_wall = self.required.iter().any(|&(x, y)| {
            let (across, along) = if vertical { (x, y) } else { (y, x) };
            let (along0, along1) = if vertical {
                (rect.y0, rect.y1)
            } else {
                (rect.x0, rect.x1)
            };
            walls.contains(&across) && (along0..along1).contains(&along)
        });
        if on_wall {
            return false;
        }
        let (first, middle, second) = if vertical {
            (
                Rect { x1: at, ..rect },
                Rect {
                    x0: at + 1,
                    x1: at + thickness - 1,
                    ..rect
                },
                Rect {
                    x0: at + thickness,
                    ..rect
                },
            )
        } else {
            (
                Rect { y1: at, ..rect },
                Rect {
                    y0: at + 1,
                    y1: at + thickness - 1,
                    ..rect
                },
                Rect {
                    y0: at + thickness,
                    ..rect
                },
            )
        };
        if self.interior_cells(first) < MIN_ROOM_CELLS
            || self.interior_cells(second) < MIN_ROOM_CELLS
        {
            return false;
        }
        if thickness > 1 {
            for y in middle.y0..middle.y1 {
                for x in middle.x0..middle.x1 {
                    if self.shape.is_interior(x, y) {
                        self.level.set(x, y, Cell::Floor);
                    }
                }
            }
            self.corridors.push(Corridor {
                floor: middle,
                north_south: vertical,
            });
        }
        self.split(first, depth + 1);
        self.split(second, depth + 1);
        true
    }

    /// The whole part (inside the outline) becomes one room.
    fn make_room(&mut self, rect: Rect) {
        let room = RoomId(self.rooms);
        let mut any = false;
        for y in rect.y0..rect.y1 {
            for x in rect.x0..rect.x1 {
                if self.shape.is_interior(x, y) {
                    self.level.set(x, y, Cell::Floor);
                    self.level.set_room(x, y, Some(room));
                    any = true;
                }
            }
        }
        if any {
            self.rooms += 1;
        }
    }
}

/// Where a corridor ends at the wall of another corridor (made by an earlier
/// split), opens the wall between them.
fn join_corridors(level: &mut Level, shape: &Shape, corridors: &[Corridor]) {
    let is_corridor = |level: &Level, x: i64, y: i64| {
        level.cell(x, y) == Cell::Floor && level.room(x, y).is_none()
    };
    for corridor in corridors {
        let f = corridor.floor;
        // Each end: the wall just past it, and the cell beyond that.
        let ends: Vec<Vec<(Spot, Spot, Spot)>> = if corridor.north_south {
            vec![
                (f.x0..f.x1)
                    .map(|x| ((x, f.y0), (x, f.y0 - 1), (x, f.y0 - 2)))
                    .collect(),
                (f.x0..f.x1)
                    .map(|x| ((x, f.y1 - 1), (x, f.y1), (x, f.y1 + 1)))
                    .collect(),
            ]
        } else {
            vec![
                (f.y0..f.y1)
                    .map(|y| ((f.x0, y), (f.x0 - 1, y), (f.x0 - 2, y)))
                    .collect(),
                (f.y0..f.y1)
                    .map(|y| ((f.x1 - 1, y), (f.x1, y), (f.x1 + 1, y)))
                    .collect(),
            ]
        };
        for end in ends {
            for (inside, wall, beyond) in end {
                if is_corridor(level, inside.0, inside.1)
                    && level.cell(wall.0, wall.1) == Cell::Wall
                    && shape.is_interior(wall.0, wall.1)
                    && is_corridor(level, beyond.0, beyond.1)
                {
                    level.set(wall.0, wall.1, Cell::Floor);
                }
            }
        }
    }
}

/// Doors in the outer wall: one, preferably facing south (the side seen from
/// the world map), and up to three more for bigger buildings. Inside rooms
/// are only joined up to what these doors already reach, so a side door may
/// be the only way into part of the building.
/// Returns each door and the ground outside it.
fn add_entrances(level: &mut Level, shape: &Shape, rng: &mut Rng) -> Vec<((i64, i64), (i64, i64))> {
    // Outer wall cells with ground straight out, floor straight in, and wall
    // either side.
    let mut candidates: Vec<(Spot, Spot, usize)> = Vec::new();
    for y in shape.y..shape.y + shape.height {
        for x in shape.x..shape.x + shape.width {
            if !shape.contains(x, y) || shape.is_interior(x, y) {
                continue;
            }
            for (side, (dx, dy)) in STEPS.into_iter().enumerate() {
                let outside = (x + dx, y + dy);
                let inside = (x - dx, y - dy);
                if level.cell(outside.0, outside.1) == Cell::Ground
                    && level.cell(inside.0, inside.1) == Cell::Floor
                    && level.cell(x + dy, y + dx) == Cell::Wall
                    && level.cell(x - dy, y - dx) == Cell::Wall
                {
                    candidates.push(((x, y), outside, side));
                }
            }
        }
    }
    let mut chosen = Vec::new();
    // STEPS[2] is south.
    let south: Vec<_> = candidates.iter().filter(|c| c.2 == 2).copied().collect();
    let pool = if south.is_empty() {
        &candidates
    } else {
        &south
    };
    if pool.is_empty() {
        return Vec::new();
    }
    let first = pool[rng.range_u64(0, pool.len() as u64) as usize];
    chosen.push(first);
    // More doors for bigger buildings, on sides without one if possible and
    // never close to another.
    let extra = roll(
        rng,
        0,
        ((shape.width + shape.height) / CELLS_PER_EXTRA_ENTRANCE).min(3),
    );
    for _ in 0..extra {
        let apart = |c: &(Spot, Spot, usize)| {
            chosen.iter().all(|d: &(Spot, Spot, usize)| {
                (d.0.0 - c.0.0).abs() + (d.0.1 - c.0.1).abs() >= MIN_ENTRANCE_GAP
            })
        };
        let new_side: Vec<_> = candidates
            .iter()
            .filter(|c| apart(c) && chosen.iter().all(|d| d.2 != c.2))
            .copied()
            .collect();
        let any_side: Vec<_> = candidates.iter().filter(|c| apart(c)).copied().collect();
        let pool = if new_side.is_empty() {
            any_side
        } else {
            new_side
        };
        if pool.is_empty() {
            break;
        }
        chosen.push(pool[rng.range_u64(0, pool.len() as u64) as usize]);
    }
    for &((x, y), _, _) in &chosen {
        level.set(x, y, Cell::Door);
    }
    chosen
        .into_iter()
        .map(|(door, outside, _)| (door, outside))
        .collect()
}

/// A wall cell inside the building between two walkable cells (north and
/// south, or east and west).
fn between(level: &Level, x: i64, y: i64) -> Option<((i64, i64), (i64, i64))> {
    [(0, 1), (1, 0)].into_iter().find_map(|(dx, dy)| {
        let (a, b) = ((x - dx, y - dy), (x + dx, y + dy));
        (level.cell(a.0, a.1) == Cell::Floor && level.cell(b.0, b.1) == Cell::Floor)
            .then_some((a, b))
    })
}

/// Adds doors in inside walls until every room and corridor can be reached
/// from `start`, preferring doors from corridors.
fn add_doors(level: &mut Level, shape: &Shape, rng: &mut Rng, start: (i64, i64)) {
    let mut reach = Reach::new(level);
    reach.spread(level, &[start]);
    while reach.first_unreached(level).is_some() {
        let mut found: Option<(i64, i64)> = None;
        // Strict first (no door next to another), then relaxed.
        for strict in [true, false] {
            let mut candidates = Vec::new();
            let mut from_corridors = Vec::new();
            for y in shape.y..shape.y + shape.height {
                for x in shape.x..shape.x + shape.width {
                    if level.cell(x, y) != Cell::Wall || !shape.is_interior(x, y) {
                        continue;
                    }
                    if strict && grid::door_nearby(level, x, y) {
                        continue;
                    }
                    let Some((a, b)) = between(level, x, y) else {
                        continue;
                    };
                    let reached = match (reach.is(a.0, a.1), reach.is(b.0, b.1)) {
                        (true, false) => a,
                        (false, true) => b,
                        _ => continue,
                    };
                    candidates.push((x, y));
                    if level.room(reached.0, reached.1).is_none() {
                        from_corridors.push((x, y));
                    }
                }
            }
            let pool = if from_corridors.is_empty() {
                candidates
            } else {
                from_corridors
            };
            if !pool.is_empty() {
                found = Some(pool[rng.range_u64(0, pool.len() as u64) as usize]);
                break;
            }
        }
        match found {
            Some((x, y)) => {
                level.set(x, y, Cell::Door);
                reach.spread(level, &[(x, y)]);
            }
            None => {
                // Separated by more than one wall (it shouldn't happen): dig.
                if !grid::dig_to_unreached(level, &reach, |x, y| shape.is_interior(x, y)) {
                    return;
                }
                reach = Reach::new(level);
                reach.spread(level, &[start]);
            }
        }
    }
}

/// A few more doors between different rooms, making loops (and places for
/// secret doors).
fn add_extra_doors(level: &mut Level, shape: &Shape, rng: &mut Rng, count: u16) {
    let mut candidates = Vec::new();
    for y in shape.y..shape.y + shape.height {
        for x in shape.x..shape.x + shape.width {
            if level.cell(x, y) != Cell::Wall || !shape.is_interior(x, y) {
                continue;
            }
            if let Some((a, b)) = between(level, x, y)
                && level.room(a.0, a.1) != level.room(b.0, b.1)
            {
                candidates.push((x, y));
            }
        }
    }
    for _ in 0..count {
        if candidates.is_empty() {
            return;
        }
        let (x, y) = candidates.swap_remove(rng.range_u64(0, candidates.len() as u64) as usize);
        if !grid::door_nearby(level, x, y) {
            level.set(x, y, Cell::Door);
        }
    }
}
