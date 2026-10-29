//! Dungeons and buildings: a grid of floor and wall cells, and what can be
//! seen (or lit) from a cell.
//!
//! Walls block movement, sight and light; floors block nothing. Sight uses
//! symmetric shadowcasting with exact integer fractions, so the client and the
//! server always agree on what a character can see. Levels are made by the
//! generators in `crate::site`.

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
    /// Seen and lit across, but not walkable (for now: swimming later).
    Water,
    /// As water, hotter.
    Lava,
}

impl Cell {
    /// Whether it can be walked on.
    pub fn is_walkable(self) -> bool {
        matches!(self, Self::Floor | Self::Door | Self::Ground)
    }

    /// Whether sight and light pass through it.
    pub fn is_open(self) -> bool {
        matches!(
            self,
            Self::Floor | Self::Door | Self::Ground | Self::Water | Self::Lava
        )
    }
}

/// Which room of a level a cell belongs to, as planned by its generator
/// (corridors, walls and outside ground belong to none). For mobs later.
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

    /// Whether `(x, y)` is inside the level's rectangle.
    pub fn contains(&self, x: i64, y: i64) -> bool {
        self.index(x, y).is_some()
    }
}

/// The cells that can be seen (or lit) from one or more points in a level.
/// Walls are seen when their face is: they block what is behind them, not
/// themselves.
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
    /// when their centre is, so seeing is symmetric (if A sees B, B sees A):
    /// the rule for the game.
    pub fn add_view(&mut self, level: &Level, x: i64, y: i64, radius: Option<u32>) {
        self.add(level, x, y, radius, false);
    }

    /// As `add_view`, but open cells count as seen when any part of them is
    /// in view (as walls always do), e.g. a side opening glimpsed from a long
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
        let mut scan = Scan {
            level,
            sight: self,
            origin: (x, y),
            radius_squared: radius.map(|r| (r as i64) * (r as i64)),
            partly,
        };
        scan.reveal(x, y);
        for quadrant in Quadrant::ALL {
            scan.scan(
                quadrant,
                Row {
                    depth: 1,
                    start: Slope::new(-1, 1),
                    end: Slope::new(1, 1),
                },
            );
        }
    }
}

/// Symmetric shadowcasting (after Albert Ford's description). Each quadrant
/// is scanned row by row away from the origin; walls narrow the range of
/// slopes still visible. Slopes are exact fractions, so there are no floats.
struct Scan<'a> {
    level: &'a Level,
    sight: &'a mut Sight,
    origin: (i64, i64),
    radius_squared: Option<i64>,
    /// Reveal open cells partly in view, not only those whose centre is.
    partly: bool,
}

/// A quarter of the view: rows run away from the origin in this direction.
#[derive(Clone, Copy)]
enum Quadrant {
    North,
    East,
    South,
    West,
}

impl Quadrant {
    const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    /// The cell `depth` rows out and `col` across from `origin`.
    fn cell(self, (x, y): (i64, i64), depth: i64, col: i64) -> (i64, i64) {
        match self {
            Self::North => (x + col, y - depth),
            Self::South => (x + col, y + depth),
            Self::East => (x + depth, y + col),
            Self::West => (x - depth, y + col),
        }
    }
}

/// A fraction `num / den`, with `den > 0`.
#[derive(Clone, Copy)]
struct Slope {
    num: i64,
    den: i64,
}

impl Slope {
    fn new(num: i64, den: i64) -> Self {
        Self { num, den }
    }

    /// The slope of the near-side edge of the cell at `(depth, col)`.
    fn of_edge(depth: i64, col: i64) -> Self {
        Self::new(2 * col - 1, 2 * depth)
    }
}

/// One row of a quadrant, between two slopes.
#[derive(Clone, Copy)]
struct Row {
    depth: i64,
    start: Slope,
    end: Slope,
}

impl Row {
    /// The columns this row covers: `depth × start` rounded with ties up, to
    /// `depth × end` rounded with ties down.
    fn columns(&self) -> std::ops::RangeInclusive<i64> {
        // floor(n + 1/2) and ceil(n - 1/2) for n = depth × num / den.
        let first =
            (2 * self.depth * self.start.num + self.start.den).div_euclid(2 * self.start.den);
        let last = -(-(2 * self.depth * self.end.num - self.end.den)).div_euclid(2 * self.end.den);
        first..=last
    }

    /// Whether a floor cell's centre lies within the row's slopes, so that
    /// seeing it is symmetric (it could see back).
    fn is_symmetric(&self, col: i64) -> bool {
        col * self.start.den >= self.depth * self.start.num
            && col * self.end.den <= self.depth * self.end.num
    }

    fn next(&self) -> Self {
        Self {
            depth: self.depth + 1,
            ..*self
        }
    }
}

impl Scan<'_> {
    fn reveal(&mut self, x: i64, y: i64) {
        if let Some(limit) = self.radius_squared {
            let (dx, dy) = (x - self.origin.0, y - self.origin.1);
            if dx * dx + dy * dy > limit {
                return;
            }
        }
        self.sight.mark(x, y);
    }

    fn scan(&mut self, quadrant: Quadrant, mut row: Row) {
        // Rows beyond the radius can't show anything. Without a radius, the
        // level's edge (all wall beyond) stops the scan.
        if let Some(limit) = self.radius_squared
            && row.depth * row.depth > limit
        {
            return;
        }
        let mut previous_open: Option<bool> = None;
        for col in row.columns() {
            let (x, y) = quadrant.cell(self.origin, row.depth, col);
            let open = self.level.is_open(x, y);
            if !open || self.partly || row.is_symmetric(col) {
                self.reveal(x, y);
            }
            if previous_open == Some(false) && open {
                row.start = Slope::of_edge(row.depth, col);
            }
            if previous_open == Some(true) && !open {
                let mut next = row.next();
                next.end = Slope::of_edge(row.depth, col);
                self.scan(quadrant, next);
            }
            previous_open = Some(open);
        }
        if previous_open == Some(true) {
            self.scan(quadrant, row.next());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                "   ##############",
                "    ............#",
                "      ..........#",
                "      #.........#",
                "      #.........#",
                "      #.........#",
                "      ..........#",
                "    ............#",
                "   ##############",
            ]
        );
    }

    /// A long corridor with a side opening far along it: the opening's
    /// centre is out of view from the corridor's end, but part of it shows.
    #[test]
    fn glimpses_show_side_openings() {
        let mut level = Level::new(-1, -2, 12, 4);
        for x in 0..10 {
            level.set(x, 0, Cell::Floor);
        }
        level.set(8, -1, Cell::Floor);
        let mut sight = Sight::new(&level);
        sight.add_view(&level, 0, 0, None);
        assert!(!sight.is_seen(8, -1));
        sight.add_glimpses(&level, 0, 0, None);
        assert!(sight.is_seen(8, -1));
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
