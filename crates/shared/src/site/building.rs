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
//! same amount on every side, so its walls stand on the floor below). Bigger
//! buildings may have several staircases between two floors. Each is on the
//! same cell of both floors, chosen before either is
//! generated, and the splits avoid them, so each floor can be generated on
//! its own. A ruined building's floors depend on each other (see `ruin`), so
//! they are all made together at the start and kept.

use serde::{Deserialize, Serialize};

use super::grid::{self, Reach, STEPS, chance, roll};
use super::ruin;
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

/// The smallest floor a tapering building has: a smallest room and its walls.
const MIN_TAPERED_FLOOR: i64 = MIN_ROOM + 2;

/// Up to one extra staircase between two floors per this many cells of the
/// upper floor (at most `MOST_EXTRA_STAIRS`), each with this percentage chance,
/// at least `MIN_STAIRS_GAP` cells from the others between those floors.
const CELLS_PER_EXTRA_STAIRS: i64 = 300;
const MOST_EXTRA_STAIRS: i64 = 3;
const EXTRA_STAIRS_PERCENT: u32 = 50;
const MIN_STAIRS_GAP: i64 = 8;

/// Percentage chance that a building of `same_size` floors has a small top.
const SMALL_TOP_PERCENT: u32 = 50;

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
    /// At least this many floors, whatever its size.
    pub min_floors: u32,
    /// At most this many floors.
    pub max_floors: u32,
    /// Every floor is the same size as the ground floor. Above `min_floors`,
    /// the top one may be about half the size.
    pub same_size: bool,
    /// Floors go on up, each smaller by this many cells per side (an inclusive
    /// range), until no room fits at the top: instead of the other floor rules.
    pub taper: Option<(i64, i64)>,
    /// Bigger buildings have more floors: up to one per this many cells of
    /// the ground floor's shorter side.
    pub size_per_floor: i64,
    /// How ruined it is, 0 (a normal building) to 5. The damage has its own
    /// seed, so the floorplan is the same at every level.
    pub ruin: u8,
    /// Percentage chance of a basement under the ground floor.
    pub basement_percent: u32,
    /// How big a basement is, if not the size of the ground floor: walls
    /// included, inclusive ranges, never smaller than the ground floor.
    pub basement_size: Option<((i64, i64), (i64, i64))>,
    /// Of the places in outside walls a window could go, what percentage have
    /// one, on every floor but a basement.
    pub windows_percent: u32,
    /// Whether the ground floor has them too.
    pub ground_windows: bool,
}

/// One floor's outline, in level coordinates.
#[derive(Clone, Copy, Debug)]
pub(super) struct Shape {
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
    pub(super) fn contains(&self, x: i64, y: i64) -> bool {
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
    /// `stairs[n]` are the staircases joining floor `n` to floor `n + 1`, at
    /// least one each.
    stairs: Vec<Vec<(i64, i64)>>,
    /// Where the stairs down to the basement are, if it has one. The basement
    /// is the last level, and is never ruined: nothing above it comes down
    /// that far.
    basement: Option<(i64, i64)>,
    /// Its outline, which reaches at least as far as the ground floor's.
    basement_shape: Shape,
    windows_percent: u32,
    ground_windows: bool,
    /// A ruined building's floors, all of them, from the ground up; empty if
    /// it isn't ruined.
    ruined: Vec<SiteLevel>,
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
        let least = options.min_floors.max(1) as i64;
        let most = (width.min(height) / options.size_per_floor.max(1))
            .clamp(1, options.max_floors.max(1) as i64)
            .max(least);
        let wanted = roll(&mut rng, least, most);
        let mut floors = vec![ground];
        while options.taper.is_some() || (floors.len() as i64) < wanted {
            let below = floors[floors.len() - 1];
            let smallest = |floor: Shape| floor.width.min(floor.height);
            let above = if let Some((fewest, greatest)) = options.taper {
                let by = roll(&mut rng, fewest, greatest);
                if smallest(below.shrunk(by)) >= MIN_TAPERED_FLOOR {
                    below.shrunk(by)
                } else {
                    // The bigger step may not fit where the smaller one does.
                    below.shrunk(fewest)
                }
            } else {
                // Half the time the same size; otherwise a cell smaller on each
                // side for every 10 in the roll above 50.
                let roll = rng.range_u64(0, 100);
                let by = if options.same_size || roll < SAME_SIZE_PERCENT {
                    0
                } else {
                    ((roll - SAME_SIZE_PERCENT) / 10 + 1) as i64
                };
                // Leaving room for the floors it must have.
                let needed = least - floors.len() as i64;
                let by = if needed > 0 {
                    by.min((smallest(below) - MIN_FLOOR_SIZE) / (2 * needed))
                } else {
                    by
                };
                below.shrunk(by)
            };
            let too_small = if options.taper.is_some() {
                MIN_TAPERED_FLOOR
            } else {
                MIN_FLOOR_SIZE
            };
            if smallest(above) < too_small {
                break;
            }
            floors.push(above);
        }
        if options.same_size && floors.len() as i64 > least && chance(&mut rng, SMALL_TOP_PERCENT) {
            let top = floors.len() - 1;
            floors[top] = floors[top].shrunk(floors[top].width.min(floors[top].height) / 4);
        }

        // Stairs: on cells well inside both floors (not next to an outer
        // wall), never next to those on the floor above. From the top down,
        // as small floors have the fewest places for them.
        let mut stairs: Vec<Vec<(i64, i64)>> = Vec::new();
        for pair in (0..floors.len() - 1).rev() {
            let above = floors[pair + 1];
            let clear_of = |spots: &[(i64, i64)], gap: i64, x: i64, y: i64| {
                spots
                    .iter()
                    .all(|&(sx, sy)| (sx - x).abs() > gap || (sy - y).abs() > gap)
            };
            let upper = stairs.last().cloned().unwrap_or_default();
            let inside = |x: i64, y: i64| {
                (-1..=1).all(|dy| (-1..=1).all(|dx| above.is_interior(x + dx, y + dy)))
                    && clear_of(&upper, 2, x, y)
            };
            let cells = above.width * above.height;
            let random = (0..100).find_map(|_| {
                let x = roll(&mut rng, above.x, above.x + above.width - 1);
                let y = roll(&mut rng, above.y, above.y + above.height - 1);
                inside(x, y).then_some((x, y))
            });
            let found = random.or_else(|| {
                (above.y..above.y + above.height)
                    .flat_map(|y| (above.x..above.x + above.width).map(move |x| (x, y)))
                    .find(|&(x, y)| inside(x, y))
            });
            let Some(first) = found else {
                // No way up from here: what is above goes.
                floors.truncate(pair + 1);
                stairs.clear();
                continue;
            };
            let mut here = vec![first];
            for _ in 0..(cells / CELLS_PER_EXTRA_STAIRS).min(MOST_EXTRA_STAIRS) {
                if !chance(&mut rng, EXTRA_STAIRS_PERCENT) {
                    continue;
                }
                let extra = (0..100).find_map(|_| {
                    let x = roll(&mut rng, above.x, above.x + above.width - 1);
                    let y = roll(&mut rng, above.y, above.y + above.height - 1);
                    (inside(x, y) && clear_of(&here, MIN_STAIRS_GAP, x, y)).then_some((x, y))
                });
                here.extend(extra);
            }
            stairs.push(here);
        }
        stairs.reverse();

        // (No draw at all for buildings that can't have one.)
        let basement = if options.basement_percent > 0 && chance(&mut rng, options.basement_percent)
        {
            let shape = floors[0];
            let above = stairs.first().cloned().unwrap_or_default();
            (0..100).find_map(|_| {
                let x = roll(&mut rng, shape.x, shape.x + shape.width - 1);
                let y = roll(&mut rng, shape.y, shape.y + shape.height - 1);
                let clear = above
                    .iter()
                    .all(|&(sx, sy)| (sx - x).abs() > 2 || (sy - y).abs() > 2);
                ((-1..=1).all(|dy| (-1..=1).all(|dx| shape.is_interior(x + dx, y + dy))) && clear)
                    .then_some((x, y))
            })
        } else {
            None
        };

        let basement_shape = match options.basement_size.filter(|_| basement.is_some()) {
            Some((wide, tall)) => {
                let ground = floors[0];
                let width = roll(&mut rng, wide.0.max(ground.width), wide.1.max(ground.width));
                let height = roll(
                    &mut rng,
                    tall.0.max(ground.height),
                    tall.1.max(ground.height),
                );
                Shape {
                    outline: Outline::Rectangle,
                    x: ground.x - roll(&mut rng, 0, width - ground.width),
                    y: ground.y - roll(&mut rng, 0, height - ground.height),
                    width,
                    height,
                    corner: ground.corner,
                }
            }
            None => floors[0],
        };

        let mut plan = Self {
            seed,
            width: width + 2 * BORDER,
            height: height + 2 * BORDER,
            floors,
            stairs,
            basement,
            basement_shape,
            windows_percent: options.windows_percent,
            ground_windows: options.ground_windows,
            ruined: Vec::new(),
        };
        if options.ruin > 0 {
            let storeys = plan.floors.len() as u32;
            let intact = (0..storeys).map(|n| plan.intact_level(n)).collect();
            plan.ruined = ruin::ruin_floors(
                seed,
                options.ruin,
                intact,
                &plan.floors,
                &plan.stairs,
                basement,
            );
            // Floors may have been lost: the basement is still the last level.
            let last = plan.ruined.len() as u32;
            for stairs in &mut plan.ruined[0].stairs {
                if stairs.direction == StairsDirection::Down {
                    stairs.to = Exit::Level(last);
                }
            }
            for (index, floor) in plan.ruined.iter_mut().enumerate() {
                if index > 0 || plan.ground_windows {
                    add_windows(
                        &mut floor.level,
                        &plan.floors[index],
                        seed,
                        plan.windows_percent,
                        index as i64,
                    );
                }
            }
        }
        plan
    }

    /// Whether it has a basement, its last level.
    pub fn has_basement(&self) -> bool {
        self.basement.is_some()
    }

    /// The ground floor's outline, in level coordinates.
    pub(super) fn ground(&self) -> Shape {
        self.floors[0]
    }

    /// Each floor's width, from the ground up.
    #[cfg(test)]
    pub fn floor_widths(&self) -> Vec<i64> {
        self.floors.iter().map(|floor| floor.width).collect()
    }

    /// The ground floor's size in cells, without the border round it.
    pub fn footprint(&self) -> (i64, i64) {
        (self.floors[0].width, self.floors[0].height)
    }

    /// The shape of the ground floor.
    pub fn outline(&self) -> Outline {
        self.floors[0].outline
    }

    /// How many levels it has: its floors, and any basement.
    pub fn floors(&self) -> u32 {
        self.storeys() + u32::from(self.has_basement())
    }

    /// How many floors it has above the ground, and on it.
    fn storeys(&self) -> u32 {
        if self.ruined.is_empty() {
            self.floors.len() as u32
        } else {
            self.ruined.len() as u32
        }
    }

    /// Level `index` (0 the ground floor, then up, the last any basement).
    pub fn level(&self, index: u32) -> SiteLevel {
        if index >= self.storeys() {
            return self.basement_level();
        }
        match self.ruined.get(index as usize) {
            Some(floor) => floor.clone(),
            None => {
                let mut floor = self.intact_level(index);
                let shape = &self.floors[index as usize];
                if index > 0 || self.ground_windows {
                    add_windows(
                        &mut floor.level,
                        shape,
                        self.seed,
                        self.windows_percent,
                        i64::from(index),
                    );
                }
                floor
            }
        }
    }

    /// The basement.
    fn basement_level(&self) -> SiteLevel {
        let (x, y) = self.basement.expect("a basement");
        let stairs = vec![Stairs {
            x,
            y,
            direction: StairsDirection::Up,
            to: Exit::Level(0),
        }];
        // (Not one of the floors' seeds: the floor count can change with ruin.)
        let rng = self.seed.derive("building basement").rng();
        self.fill_floor(self.basement_shape, rng, stairs, false)
    }

    /// Generates floor `index` as it is before any ruin.
    fn intact_level(&self, index: u32) -> SiteLevel {
        let n = index as usize;
        let shape = self.floors[n];
        let rng = Seed::new(self.seed.derive("building floor").at(&[index as i64])).rng();

        // Stairs down to the floor below and up to the one above.
        let mut stairs = Vec::new();
        if let (0, Some((x, y))) = (n, self.basement) {
            stairs.push(Stairs {
                x,
                y,
                direction: StairsDirection::Down,
                to: Exit::Level(self.floors.len() as u32),
            });
        }
        if n > 0 {
            for &(x, y) in &self.stairs[n - 1] {
                stairs.push(Stairs {
                    x,
                    y,
                    direction: StairsDirection::Down,
                    to: Exit::Level(index - 1),
                });
            }
        }
        if n + 1 < self.floors.len() {
            for &(x, y) in &self.stairs[n] {
                stairs.push(Stairs {
                    x,
                    y,
                    direction: StairsDirection::Up,
                    to: Exit::Level(index + 1),
                });
            }
        }
        self.fill_floor(shape, rng, stairs, n == 0)
    }

    /// Splits an outline into rooms round its stairs, with the ground outside
    /// and the doors out if it is on the ground.
    fn fill_floor(
        &self,
        shape: Shape,
        mut rng: Rng,
        stairs: Vec<Stairs>,
        on_ground: bool,
    ) -> SiteLevel {
        // (A bigger basement can reach past the ground floor's level.)
        let (x0, y0) = (shape.x.min(0), shape.y.min(0));
        let x1 = (shape.x + shape.width).max(self.width);
        let y1 = (shape.y + shape.height).max(self.height);
        let mut level = Level::new(x0, y0, x1 - x0, y1 - y0);
        let required: Vec<(i64, i64)> = stairs.iter().map(|s| (s.x, s.y)).collect();

        if on_ground {
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
        if on_ground {
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
            fields: Vec::new(),
        }
    }
}

/// Puts windows in a floor's outside walls: a straight wall between the inside
/// and the outside of its outline, with wall on either side along it and none
/// at a diagonal, which would look cramped (the solid wall beyond an upper floor's
/// outline doesn't count). After any ruin, as what is left decides where they fit. By hashes, not the floor's
/// sequence: where they go changes nothing else.
fn add_windows(level: &mut Level, shape: &Shape, seed: Seed, percent: u32, floor: i64) {
    if percent == 0 {
        return;
    }
    let hashes = seed.derive("building windows");
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let fits = STEPS.iter().any(|&(dx, dy)| {
                let beside = |sign: i64| {
                    let (bx, by) = (x + dy * sign, y + dx * sign);
                    level.cell(bx, by) == Cell::Wall && shape.contains(bx, by)
                };
                !shape.contains(x + dx, y + dy)
                    && level.is_walkable(x - dx, y - dy)
                    && beside(1)
                    && beside(-1)
            });
            let clear = [(1, 1), (1, -1), (-1, 1), (-1, -1)]
                .iter()
                .all(|&(dx, dy)| {
                    !(level.cell(x + dx, y + dy).is_wall() && shape.contains(x + dx, y + dy))
                });
            if level.cell(x, y) == Cell::Wall
                && shape.contains(x, y)
                && fits
                && clear
                && hashes.at(&[floor, x, y]) % 100 < u64::from(percent)
            {
                level.set(x, y, Cell::Window);
            }
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
