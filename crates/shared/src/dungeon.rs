//! Dungeons and buildings: a grid of floor and wall cells, and what can be
//! seen (or lit) from a cell.
//!
//! A wall takes up only a thin slice of its cell, running through the middle
//! towards the walls next to it (`WallShape`). It blocks movement in the whole
//! cell, but sight and light only where it really stands, so they pass beside
//! a column or the end of a wall. Sight follows straight lines in exact
//! integers, so the client and the server always agree on what a character can
//! see. Levels are made by the generators in `crate::site`.

use serde::{Deserialize, Serialize};

/// One grid cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    /// Open: can be walked on, seen across and lit through.
    Floor,
    /// Solid: blocks movement, sight and light.
    Wall,
    /// A doorway. Doors are open for now (opening and closing them comes
    /// with mobs), so it behaves like floor.
    Door,
    /// A hidden door: a wall until someone finds it (finding comes later).
    /// Generators only put them where nothing depends on them.
    SecretDoor,
    /// Outside ground round a building or cave mouth: walkable, and the way
    /// back to the world map.
    Ground,
    /// Seen and lit across, but not walkable: it can only be jumped over
    /// (`movement`; swimming later).
    Water,
    /// As water, hotter.
    Lava,
    /// Blocks movement but not sight or light. Joins the walls either side
    /// of it, like a door.
    Window,
    /// Road surfaces in a town, poorest first: walkable outside ground, the
    /// land's own beyond them.
    DirtTrack,
    GravelTrack,
    WoodenBoards,
    Cobblestone,
    Flagstones,
    DressedStone,
    /// Walkable outside ground, planted: a town's garden.
    Garden,
    /// The door of a town's solid building: part of its wall, with nothing
    /// behind it to go into. Used from the cell it faces.
    BuildingDoor,
    /// A window in a town's solid building, for looks: part of its wall.
    BlindWindow,
    /// A gap in the floor, or open air beyond a ruined floor's edge: seen and
    /// lit across, and not out in the weather. Not walkable: as with liquid,
    /// one cell of it can only be jumped over (`movement`).
    Hole,
}

impl Cell {
    /// Whether it can be walked on.
    pub fn is_walkable(self) -> bool {
        matches!(self, Self::Floor | Self::Door | Self::Ground | Self::Garden) || self.is_road()
    }

    /// Whether it is a road's surface.
    pub fn is_road(self) -> bool {
        matches!(
            self,
            Self::DirtTrack
                | Self::GravelTrack
                | Self::WoodenBoards
                | Self::Cobblestone
                | Self::Flagstones
                | Self::DressedStone
        )
    }

    /// Whether it is out in the open: weather falls on it.
    pub fn is_outside(self) -> bool {
        matches!(self, Self::Ground | Self::Garden) || self.is_road()
    }

    /// Whether it is a liquid: not walkable, but one cell of it can be jumped.
    pub fn is_liquid(self) -> bool {
        matches!(self, Self::Water | Self::Lava)
    }

    /// Whether one cell of it can be jumped over: liquid, or a hole.
    pub fn is_jumpable(self) -> bool {
        self.is_liquid() || self == Self::Hole
    }

    /// Whether sight and light pass through it.
    pub fn is_open(self) -> bool {
        self.is_walkable() || matches!(self, Self::Water | Self::Lava | Self::Window | Self::Hole)
    }

    /// Whether it is a wall (a hidden door looks and acts like one, and so do
    /// the doors and windows of a town's solid buildings).
    pub fn is_wall(self) -> bool {
        matches!(
            self,
            Self::Wall | Self::SecretDoor | Self::BuildingDoor | Self::BlindWindow
        )
    }

    /// Whether it is a hole in a wall, which joins the walls either side.
    pub fn is_opening(self) -> bool {
        matches!(self, Self::Door | Self::Window)
    }
}

/// Cells measured in units for sight's exact arithmetic: a cell is this wide,
/// with its centre on a multiple of it.
pub const CELL_UNITS: i64 = 40;

/// Half a wall's thickness, in units: it covers 45% to 55% of its cell.
pub const WALL_HALF_UNITS: i64 = 2;

/// How far past the centre a wall joined to only one other goes, in units: it
/// stops 60% of the way across its cell.
pub const WALL_END_UNITS: i64 = 4;

/// Half a cell, in units.
const HALF_CELL: i64 = CELL_UNITS / 2;

/// Which of a wall cell's four sides the wall runs out to, because it joins
/// another wall (or a door or window in line with it) there.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WallShape {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

impl WallShape {
    /// How far the wall reaches from the cell's centre towards each side, in
    /// units, as `[north, east, south, west]`: to the cell's edge where it
    /// joins something, otherwise just its thickness, except that a wall
    /// joined on one side only carries on a little past the centre.
    pub fn reach(self) -> [i64; 4] {
        let sides = [self.north, self.east, self.south, self.west];
        let joined = sides.iter().filter(|&&side| side).count();
        std::array::from_fn(|i| {
            if sides[i] {
                HALF_CELL
            } else if joined == 1 && sides[(i + 2) % 4] {
                WALL_END_UNITS
            } else {
                WALL_HALF_UNITS
            }
        })
    }
}

/// Which room of a level a cell belongs to, as planned by its generator
/// (corridors, walls and outside ground belong to none). For mobs later. In a
/// town it is the building instead, whose walls have it too: walls of different
/// buildings don't join, leaving a gap that can be seen along.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomId(pub u16);

/// Stored for cells in no room.
const NO_ROOM: u16 = u16::MAX;

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
        }
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

    /// Whether `(x, y)` can be seen across and lit through.
    pub fn is_open(&self, x: i64, y: i64) -> bool {
        self.cell(x, y).is_open()
    }

    /// Whether `(x, y)` can be walked on.
    pub fn is_walkable(&self, x: i64, y: i64) -> bool {
        self.cell(x, y).is_walkable()
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
        self.cell(x, y).is_wall().then(|| WallShape {
            north: self.same_building((x, y), (x, y - 1)) && self.joins_wall(x, y - 1, false),
            east: self.same_building((x, y), (x + 1, y)) && self.joins_wall(x + 1, y, true),
            south: self.same_building((x, y), (x, y + 1)) && self.joins_wall(x, y + 1, false),
            west: self.same_building((x, y), (x - 1, y)) && self.joins_wall(x - 1, y, true),
        })
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
        let cell = self.cell(x, y);
        cell.is_wall() || (cell.is_opening() && self.opening_runs_east_west(x, y) == east_west)
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

/// The cells that can be seen (or lit) from one or more points in a level.
/// A wall is seen when an open cell touching it (corners too) is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sight {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    seen: Vec<bool>,
}

impl Sight {
    /// Nothing seen yet, sized for `level`.
    pub fn new(level: &Level) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        Self {
            min_x,
            min_y,
            width,
            height,
            seen: vec![false; (width * height) as usize],
        }
    }

    /// Whether `(x, y)` is seen. Always false outside the level.
    pub fn is_seen(&self, x: i64, y: i64) -> bool {
        let (col, row) = (x - self.min_x, y - self.min_y);
        (0..self.width).contains(&col)
            && (0..self.height).contains(&row)
            && self.seen[(row * self.width + col) as usize]
    }

    /// Counts `(x, y)` as seen, e.g. a cell remembered from earlier.
    /// Nothing happens outside the level.
    pub fn insert(&mut self, x: i64, y: i64) {
        self.mark(x, y);
    }

    fn mark(&mut self, x: i64, y: i64) {
        let (col, row) = (x - self.min_x, y - self.min_y);
        if (0..self.width).contains(&col) && (0..self.height).contains(&row) {
            self.seen[(row * self.width + col) as usize] = true;
        }
    }

    /// Adds everything seen from `(x, y)`, out to `radius` cells (straight-line
    /// distance between cell centres; `None` for no limit). Several calls
    /// combine, e.g. for allies sharing their sight. Open cells count as seen
    /// when a straight line between the centres misses every wall, so seeing
    /// is symmetric (if A sees B, B sees A): the rule for the game.
    pub fn add_view(&mut self, level: &Level, x: i64, y: i64, radius: Option<u32>) {
        self.add(level, x, y, radius, false);
    }

    /// As `add_view`, but open cells count as seen when a line reaches near
    /// any of their corners too, e.g. a side opening glimpsed from a long
    /// corridor. Not symmetric, so only for drawing what a viewer can make
    /// out, never for game rules.
    pub fn add_glimpses(&mut self, level: &Level, x: i64, y: i64, radius: Option<u32>) {
        self.add(level, x, y, radius, true);
    }

    fn add(&mut self, level: &Level, x: i64, y: i64, radius: Option<u32>, partly: bool) {
        // Nothing can be seen from inside a wall or outside the level.
        if !level.is_open(x, y) {
            return;
        }
        let from = (x * CELL_UNITS, y * CELL_UNITS);
        for cy in self.min_y..self.min_y + self.height {
            for cx in self.min_x..self.min_x + self.width {
                let (dx, dy) = (cx - x, cy - y);
                let in_reach = radius.is_none_or(|r| dx * dx + dy * dy <= i64::from(r).pow(2));
                if !in_reach || !level.is_open(cx, cy) || self.is_seen(cx, cy) {
                    continue;
                }
                let centre = (cx * CELL_UNITS, cy * CELL_UNITS);
                let seen = clear_line(level, from, centre)
                    || (partly
                        && GLIMPSE_POINTS.iter().any(|(ox, oy)| {
                            clear_line(level, from, (centre.0 + ox, centre.1 + oy))
                        }));
                if seen {
                    self.reveal(level, cx, cy);
                }
            }
        }
    }

    /// Counts an open cell as seen, and the walls touching it.
    fn reveal(&mut self, level: &Level, x: i64, y: i64) {
        self.mark(x, y);
        for ny in y - 1..=y + 1 {
            for nx in x - 1..=x + 1 {
                if !level.is_open(nx, ny) {
                    self.mark(nx, ny);
                }
            }
        }
    }
}

/// The points near an open cell's corners that count for glimpses, in units
/// from its centre.
const GLIMPSE_POINTS: [(i64, i64); 4] = [(-16, -16), (16, -16), (-16, 16), (16, 16)];

/// Whether the straight line between two points (in units) misses every wall.
/// Goes through the cells it crosses, column by column from `from`.
fn clear_line(level: &Level, from: (i64, i64), to: (i64, i64)) -> bool {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let (first, last) = (cell_of(from.0), cell_of(to.0));
    let step = if last >= first { 1 } else { -1 };
    let mut column = first;
    loop {
        // The part of the line in this column.
        let left = (column * CELL_UNITS - HALF_CELL).max(from.0.min(to.0));
        let right = (column * CELL_UNITS + HALF_CELL).min(from.0.max(to.0));
        let (y1, y2) = if dx == 0 {
            (from.1, to.1)
        } else {
            (
                from.1 + (left - from.0) * dy / dx,
                from.1 + (right - from.0) * dy / dx,
            )
        };
        // One unit more each way, as the divisions above round.
        let (top, bottom) = (cell_of(y1.min(y2) - 1), cell_of(y1.max(y2) + 1));
        for row in top..=bottom {
            if let Some(shape) = level.wall_shape(column, row)
                && line_hits_wall(from, to, (column, row), shape)
            {
                return false;
            }
        }
        if column == last {
            return true;
        }
        column += step;
    }
}

/// The cell a coordinate (in units) is in.
fn cell_of(units: i64) -> i64 {
    (units + HALF_CELL).div_euclid(CELL_UNITS)
}

/// Whether a line touches the wall in `cell`. The wall is two bars crossing
/// at the cell's centre.
fn line_hits_wall(from: (i64, i64), to: (i64, i64), cell: (i64, i64), shape: WallShape) -> bool {
    let (cx, cy) = (cell.0 * CELL_UNITS, cell.1 * CELL_UNITS);
    let [north, east, south, west] = shape.reach();
    let t = WALL_HALF_UNITS;
    line_hits_box(from, to, (cx - west, cy - t), (cx + east, cy + t))
        || line_hits_box(from, to, (cx - t, cy - north), (cx + t, cy + south))
}

/// Whether a line touches a box given by its smallest and largest corners:
/// their ranges overlap, and the box's corners aren't all on one side of it.
fn line_hits_box(from: (i64, i64), to: (i64, i64), low: (i64, i64), high: (i64, i64)) -> bool {
    if from.0.max(to.0) < low.0
        || from.0.min(to.0) > high.0
        || from.1.max(to.1) < low.1
        || from.1.min(to.1) > high.1
    {
        return false;
    }
    let side = |x: i64, y: i64| (to.0 - from.0) * (y - from.1) - (to.1 - from.1) * (x - from.0);
    let sides = [
        side(low.0, low.1),
        side(high.0, low.1),
        side(low.0, high.1),
        side(high.0, high.1),
    ];
    !(sides.iter().all(|&s| s > 0) || sides.iter().all(|&s| s < 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hole is seen and lit across, but nothing else.
    #[test]
    fn holes_are_open_but_not_walkable() {
        let hole = Cell::Hole;
        assert!(hole.is_open());
        assert!(!hole.is_walkable() && !hole.is_outside() && !hole.is_liquid());
        assert!(!hole.is_wall() && !hole.is_opening());
        assert!(hole.is_jumpable());
    }

    /// The room as a picture, `#` wall, `.` floor.
    fn picture(level: &Level, mark: impl Fn(i64, i64) -> char) -> Vec<String> {
        let (min_x, min_y, width, height) = level.bounds();
        (min_y..min_y + height)
            .map(|y| (min_x..min_x + width).map(|x| mark(x, y)).collect())
            .collect()
    }

    #[test]
    fn test_room_layout() {
        let level = Level::test_room();
        let rows = picture(&level, |x, y| if level.is_open(x, y) { '.' } else { '#' });
        assert_eq!(
            rows,
            [
                "#################",
                "#...............#",
                "#...............#",
                "#.....#.........#",
                "#.....#.........#",
                "#.....#.........#",
                "#...............#",
                "#...............#",
                "#################",
            ]
        );
        // Outside the level is wall.
        assert!(!level.is_open(100, 0));
    }

    /// Golden test: what the player sees from the middle of the test room.
    /// The short wall two steps west hides most of the room's west end.
    #[test]
    fn sight_from_the_middle_of_the_test_room() {
        let level = Level::test_room();
        let mut sight = Sight::new(&level);
        sight.add_view(&level, 0, 0, None);
        let rows = picture(&level, |x, y| {
            match (sight.is_seen(x, y), level.is_open(x, y)) {
                (true, true) => '.',
                (true, false) => '#',
                (false, _) => ' ',
            }
        });
        assert_eq!(
            rows,
            [
                "  ###############",
                "   .............#",
                "     ...........#",
                "      #.........#",
                "      #.........#",
                "      #.........#",
                "     ...........#",
                "   .............#",
                "  ###############",
            ]
        );
    }

    /// A wall's end hides a far cell's centre, but one of its corners shows.
    #[test]
    fn glimpses_show_parts_of_cells() {
        let mut level = Level::test_room();
        level.set(4, -1, Cell::Wall);
        level.set(4, 0, Cell::Wall);
        let mut sight = Sight::new(&level);
        sight.add_view(&level, 2, 0, None);
        assert!(!sight.is_seen(6, 0));
        sight.add_glimpses(&level, 2, 0, None);
        assert!(sight.is_seen(6, 0));
    }

    /// A floor with walls placed by a picture: `#` wall, `+` door, `W` window.
    fn drawn(rows: &[&str]) -> Level {
        let mut level = Level::new(0, 0, rows[0].len() as i64, rows.len() as i64);
        for (y, row) in rows.iter().enumerate() {
            for (x, mark) in row.chars().enumerate() {
                let cell = match mark {
                    '#' => Cell::Wall,
                    '+' => Cell::Door,
                    'W' => Cell::Window,
                    _ => Cell::Floor,
                };
                level.set(x as i64, y as i64, cell);
            }
        }
        level
    }

    #[test]
    fn walls_take_their_shape_from_their_neighbours() {
        let level = drawn(&[
            ".....", //
            ".#...", //
            ".....", //
            ".###.", //
            "...#.", //
            ".###.", //
            "...#.", //
            ".#+#.", //
        ]);
        let shape = |x, y| level.wall_shape(x, y).unwrap();
        let reach = |x, y| shape(x, y).reach();
        // A column.
        assert_eq!(reach(1, 1), [2, 2, 2, 2]);
        // The end of a wall stops 60% of the way across.
        assert_eq!(reach(1, 3), [2, 20, 2, 4]);
        // Straight on, and a corner.
        assert_eq!(reach(2, 3), [2, 20, 2, 20]);
        assert_eq!(reach(3, 3), [2, 2, 20, 20]);
        // A T.
        assert_eq!(reach(3, 5), [20, 2, 20, 20]);
        // Walls join a door in line with them.
        assert!(shape(1, 7).east && shape(3, 7).west);
        // Not one they stand beside.
        assert!(level.opening_runs_east_west(2, 7));
        assert!(!shape(2, 5).south);
        assert_eq!(level.wall_shape(2, 7), None);
    }

    #[test]
    fn walls_block_only_where_they_stand() {
        let mut level = Level::test_room();
        // Beside a column, along a line that crosses its cell off centre.
        level.set(2, 0, Cell::Wall);
        assert!(clear_line(&level, (0, -10), (160, 30)));
        let mut sight = Sight::new(&level);
        sight.add_view(&level, 0, 0, None);
        // Straight through it is hidden, the cells either side are not.
        assert!(!sight.is_seen(4, 0));
        assert!(sight.is_seen(4, 1));
        // Seen from beside it, past its cell's corner.
        let mut sight = Sight::new(&level);
        sight.add_view(&level, 1, 1, None);
        assert!(sight.is_seen(3, 0));
        assert!(sight.is_seen(2, 0));
    }

    #[test]
    fn doors_and_windows_let_sight_through_but_walls_do_not() {
        let level = drawn(&[
            "...", //
            "#W#", //
            "...", //
            "#+#", //
            "...", //
            "###", //
            "...", //
        ]);
        let mut sight = Sight::new(&level);
        sight.add_view(&level, 1, 2, None);
        assert!(sight.is_seen(1, 0));
        assert!(sight.is_seen(1, 4));
        assert!(!sight.is_seen(1, 6));
        // Windows can't be walked through; both can be seen across.
        assert!(!level.is_walkable(1, 1) && level.is_open(1, 1));
        assert!(level.is_walkable(1, 3) && level.is_open(1, 3));
        // Walls touching what is seen are seen.
        assert!(sight.is_seen(1, 5));
    }

    /// Sight is the same both ways, also past thin walls.
    #[test]
    fn seeing_is_mutual() {
        let level = drawn(&[
            "........", //
            ".#..##..", //
            "....#...", //
            "..#.....", //
            ".....W#.", //
            ".##+##..", //
            "........", //
        ]);
        let (_, _, width, height) = level.bounds();
        let view = |x, y| {
            let mut sight = Sight::new(&level);
            sight.add_view(&level, x, y, None);
            sight
        };
        for y in 0..height {
            for x in 0..width {
                if !level.is_open(x, y) {
                    continue;
                }
                let sight = view(x, y);
                for oy in 0..height {
                    for ox in 0..width {
                        if level.is_open(ox, oy) {
                            let back = view(ox, oy).is_seen(x, y);
                            assert_eq!(sight.is_seen(ox, oy), back, "{x},{y} {ox},{oy}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn radius_limits_sight() {
        let level = Level::test_room();
        let mut sight = Sight::new(&level);
        sight.add_view(&level, 0, 0, Some(2));
        assert!(sight.is_seen(2, 0));
        assert!(sight.is_seen(1, 1));
        assert!(!sight.is_seen(2, 1));
        assert!(!sight.is_seen(3, 0));
    }

    #[test]
    fn views_combine() {
        let level = Level::test_room();
        let mut sight = Sight::new(&level);
        sight.add_view(&level, 0, 0, None);
        assert!(!sight.is_seen(-4, 0));
        sight.add_view(&level, -6, 0, None);
        assert!(sight.is_seen(-4, 0));
        assert!(sight.is_seen(5, 0));
    }

    #[test]
    fn nothing_is_seen_from_inside_a_wall() {
        let level = Level::test_room();
        let mut sight = Sight::new(&level);
        sight.add_view(&level, -2, 0, None);
        assert!(!sight.is_seen(-1, 0));
    }
}
