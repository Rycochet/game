//! Splitting a floor into rooms and corridors (BSP).

use super::super::grid::{chance, roll};
use super::{
    CORRIDOR_DEPTH, CORRIDOR_MIN_SPAN, CORRIDOR_PERCENT, MAX_ROOM, MIN_ROOM, MIN_ROOM_CELLS,
    STOP_PERCENT, Shape, WIDE_CORRIDOR_SPAN,
};
use crate::dungeon::{Cell, Level, RoomId};
use crate::rng::Rng;

/// A rectangle of cells, `x1` and `y1` excluded.
#[derive(Clone, Copy, Debug)]
pub(super) struct Rect {
    pub(super) x0: i64,
    pub(super) y0: i64,
    pub(super) x1: i64,
    pub(super) y1: i64,
}

/// A corridor made by a split: its floor cells, and whether it runs north to
/// south.
#[derive(Clone, Copy, Debug)]
pub(super) struct Corridor {
    pub(super) floor: Rect,
    pub(super) north_south: bool,
}

/// The binary space partitioning of one floor.
pub(super) struct Splitter<'a> {
    pub(super) level: &'a mut Level,
    pub(super) shape: &'a Shape,
    /// Cells that must stay floor (stairs): no wall may be built on them.
    pub(super) required: &'a [(i64, i64)],
    pub(super) rng: &'a mut Rng,
    pub(super) rooms: u16,
    pub(super) corridors: Vec<Corridor>,
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

    pub(super) fn split(&mut self, rect: Rect, depth: u32) {
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
