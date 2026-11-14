//! A level: its cells and rooms.

use std::collections::{BTreeMap, BTreeSet};

use super::{Cell, WallBar, WallShape, bars_of};
use crate::door::Swing;

/// Which room of a level a cell belongs to, as planned by its generator
/// (corridors, walls and outside ground belong to none). For mobs later. In a
/// town it is the building instead, whose walls have it too: walls of different
/// buildings don't join, leaving a gap that can be seen along.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomId(pub u16);

/// Stored for cells in no room.
pub(super) const NO_ROOM: u16 = u16::MAX;

/// A rectangular grid of cells. Coordinates are `x` east and `y` south, like
/// the world grid; everything outside the rectangle counts as wall.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Level {
    /// The north-west corner.
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    /// Row by row from the north, `width * height` cells.
    cells: Vec<Cell>,
    /// The room of each cell, in the same order, `NO_ROOM` for none.
    rooms: Vec<u16>,
    /// How each door hangs, where it has been set (see `doors.rs`).
    pub(super) swings: BTreeMap<(i64, i64), Swing>,
    /// The doors known to be locked (see `doors.rs`).
    pub(super) locked: BTreeSet<(i64, i64)>,
    /// Whether its walls are natural, wider at the foot (`PoiKind::natural_walls`).
    natural_walls: bool,
}

impl Level {
    /// A level `width × height` cells with its north-west corner at
    /// `(min_x, min_y)`, all wall.
    pub fn new(min_x: i64, min_y: i64, width: i64, height: i64) -> Self {
        Self {
            min_x,
            min_y,
            width,
            height,
            cells: vec![Cell::Wall; (width * height) as usize],
            rooms: vec![NO_ROOM; (width * height) as usize],
            swings: BTreeMap::new(),
            locked: BTreeSet::new(),
            natural_walls: false,
        }
    }

    /// Whether its walls are natural: wider at the foot, as drawn, so wider
    /// where they block (`clearance`). Built (thin) unless set.
    pub fn natural_walls(&self) -> bool {
        self.natural_walls
    }

    /// Sets whether its walls are natural, from its poi's kind
    /// (`PoiKind::natural_walls`): the server's site levels and the client's
    /// copy alike.
    pub fn set_natural_walls(&mut self, natural: bool) {
        self.natural_walls = natural;
    }

    /// For testing the view: a room with 15 × 7 floor cells (east–west ×
    /// north–south), walled all round, centred on `(0, 0)`, with a 1 × 3 wall
    /// two steps west of the centre.
    pub fn test_room() -> Self {
        let mut level = Self::new(-8, -4, 17, 9);
        for y in -3..=3 {
            for x in -7..=7 {
                level.set(x, y, Cell::Floor);
            }
        }
        for y in -1..=1 {
            level.set(-2, y, Cell::Wall);
        }
        level
    }

    /// The cells covered, as `(min_x, min_y, width, height)`.
    pub fn bounds(&self) -> (i64, i64, i64, i64) {
        (self.min_x, self.min_y, self.width, self.height)
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        if (0..self.width).contains(&col) && (0..self.height).contains(&row) {
            Some((row * self.width + col) as usize)
        } else {
            None
        }
    }

    /// The cell at `(x, y)`; wall outside the level.
    pub fn cell(&self, x: i64, y: i64) -> Cell {
        self.index(x, y).map_or(Cell::Wall, |i| self.cells[i])
    }

    /// Changes a cell; does nothing outside the level.
    pub fn set(&mut self, x: i64, y: i64, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = cell;
        }
    }

    /// Whether `(x, y)` can be seen across and lit through: a door only when
    /// it isn't closed.
    pub fn is_open(&self, x: i64, y: i64) -> bool {
        match self.cell(x, y) {
            Cell::Door | Cell::SecretDoor => !self.is_shut(x, y),
            cell => cell.is_open(),
        }
    }

    /// Whether `(x, y)` can be walked on: a door only when it isn't closed.
    pub fn is_walkable(&self, x: i64, y: i64) -> bool {
        match self.cell(x, y) {
            Cell::Door | Cell::SecretDoor => !self.is_shut(x, y),
            cell => cell.is_walkable(),
        }
    }

    /// The room `(x, y)` is part of, if any.
    pub fn room(&self, x: i64, y: i64) -> Option<RoomId> {
        let room = self.index(x, y).map_or(NO_ROOM, |i| self.rooms[i]);
        (room != NO_ROOM).then_some(RoomId(room))
    }

    /// Puts a cell in a room (or none); does nothing outside the level.
    pub fn set_room(&mut self, x: i64, y: i64, room: Option<RoomId>) {
        if let Some(i) = self.index(x, y) {
            self.rooms[i] = room.map_or(NO_ROOM, |r| r.0);
        }
    }

    /// The shape of the wall at `(x, y)`, if there is a wall.
    pub fn wall_shape(&self, x: i64, y: i64) -> Option<WallShape> {
        self.is_wall(x, y).then(|| WallShape {
            north: self.same_building((x, y), (x, y - 1)) && self.joins_wall(x, y - 1, false),
            east: self.same_building((x, y), (x + 1, y)) && self.joins_wall(x + 1, y, true),
            south: self.same_building((x, y), (x, y + 1)) && self.joins_wall(x, y + 1, false),
            west: self.same_building((x, y), (x - 1, y)) && self.joins_wall(x - 1, y, true),
        })
    }

    /// Where the wall at `(x, y)` stands, if there is a wall: two bars crossing
    /// at the cell's centre, each its `(smallest, largest)` corners in units.
    /// What blocks sight, and all that does.
    pub fn wall_bars(&self, x: i64, y: i64) -> Option<[WallBar; 2]> {
        self.wall_shape(x, y).map(|shape| bars_of((x, y), shape))
    }

    /// Whether two cells aren't walls of different buildings of a town. Walls
    /// elsewhere are in no room.
    fn same_building(&self, a: (i64, i64), b: (i64, i64)) -> bool {
        match (self.room(a.0, a.1), self.room(b.0, b.1)) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        }
    }

    /// Whether a wall beside `(x, y)`, to its east or west (`east_west`) or
    /// north or south, joins it: walls join walls, and openings in line.
    fn joins_wall(&self, x: i64, y: i64, east_west: bool) -> bool {
        self.is_wall(x, y)
            || (self.is_opening(x, y) && self.opening_runs_east_west(x, y) == east_west)
    }

    /// Whether the door or window at `(x, y)` is in a wall running east-west
    /// (rather than north-south): the way with more walls beside it.
    pub fn opening_runs_east_west(&self, x: i64, y: i64) -> bool {
        let solid = |x, y| {
            let cell = self.cell(x, y);
            (cell.is_wall() || cell.is_opening()) as u8
        };
        solid(x - 1, y) + solid(x + 1, y) >= solid(x, y - 1) + solid(x, y + 1)
    }

    /// Whether `(x, y)` is inside the level's rectangle.
    pub fn contains(&self, x: i64, y: i64) -> bool {
        self.index(x, y).is_some()
    }
}
