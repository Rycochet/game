//! Draws water and lava as channels cut down into the floor.
//!
//! As with walls, a cell is drawn in four quarters around its centre. What a
//! quarter looks like depends only on the four cells round the grid corner it
//! touches, so neighbouring quarters always meet: a quarter of liquid with
//! floor to both sides is rounded; with liquid on one side it is half a cell
//! wide, running that way; with liquid on both it fills the quarter, except a
//! round bite of floor at the corner if the cell diagonally across is floor.
//!
//! Walls count as liquid here. Liquid runs up to the foot of a wall (over the
//! floor of the wall's cell, like liquid in its own quarters), and the wall
//! carries on down to the bed (`walls`).
//!
//! Depth (the owner's rule): a cell with floor on two opposite sides is half a
//! cell deep, and stays so where it meets another such cell, so a narrow
//! channel's bed is flat. Where it meets a wider liquid cell, or a wall, it
//! slopes to a full cell deep. Any other cell is a full cell deep all over.

use bevy::prelude::*;
use shared::dungeon::{Cell, Level};

use super::mesh::{MEMORY_SHADE, Surface, with_ambient};
use super::walls::{Layer, Layers, Scene, flat_uv};
use super::{CELL_SIZE, mesh::LightGrid};

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// Half the width of a narrow channel, so also how far a shore stands back
/// from the edge of the cell.
pub const CHANNEL_HALF: f32 = CELL_SIZE / 4.0;

/// How far the surface of a liquid is below the floor.
const SURFACE_DROP: f32 = CELL_SIZE * 0.1;

/// How deep a narrow channel is in the middle of its cell, and how deep liquid
/// gets anywhere else.
const NARROW_DEPTH: f32 = CELL_SIZE * 0.5;
pub const FULL_DEPTH: f32 = CELL_SIZE;

/// Lengths a curved shore is cut into, in a quarter. Even, so that a corner of
/// the quarter is reached half way round.
const CURVE_PIECES: usize = 6;

/// How much darker the sides are at the floor, how much more they darken by a
/// full cell deep (evenly on the way down), and how much darker again the bed
/// is at any depth, so it stands out from them.
const RIM_DARKENING: f32 = 0.2;
const SIDE_DARKENING: f32 = 0.4;
const BED_DARKENING: f32 = 0.2;

/// How much of what is under a liquid's surface can't be seen through it.
pub fn opacity(surface: Surface) -> f32 {
    match surface {
        Surface::Water => 0.35,
        _ => 1.0,
    }
}

/// How a side (of a channel, or a wall) is shaded, `depth` below the floor.
pub fn depth_shade(depth: f32) -> f32 {
    1.0 - RIM_DARKENING - SIDE_DARKENING * depth / FULL_DEPTH
}

/// How the bed is shaded, `depth` below the floor.
fn bed_shade(depth: f32) -> f32 {
    depth_shade(depth) - BED_DARKENING
}

/// A point in a quarter of a cell: distances from its centre, towards the
/// quarter's east-west side and its north-south side.
type Point = (f32, f32);

/// The shape of the liquid in a quarter of a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outline {
    /// Floor to both sides: round, about the cell's centre.
    Round,
    /// Half a cell wide, running to the quarter's east-west side.
    AlongEast,
    /// Half a cell wide, running to the quarter's north-south side.
    AlongSouth,
    /// All of it but a round bite of floor at the corner of the cell.
    Notch,
    /// All of it.
    Full,
}

impl Outline {
    /// `beside` and `ahead` say whether the liquid carries on past the
    /// quarter's east-west and north-south sides, `across` past its corner.
    fn of(beside: bool, ahead: bool, across: bool) -> Self {
        match (beside, ahead) {
            (false, false) => Self::Round,
            (true, false) => Self::AlongEast,
            (false, true) => Self::AlongSouth,
            (true, true) if across => Self::Full,
            (true, true) => Self::Notch,
        }
    }

    /// The floor beside the liquid in a wall's quarter, if it is the part of
    /// the quarter past a straight shore, as `(east, south)`: the floor there
    /// is drawn like any other beside a wall.
    pub fn straight_shore(self) -> Option<(f32, f32)> {
        match self {
            Self::AlongEast => Some((HALF_CELL, CHANNEL_HALF)),
            Self::AlongSouth => Some((CHANNEL_HALF, HALF_CELL)),
            _ => None,
        }
    }

    /// The part of a wall's foot in this liquid: the part between `from` and
    /// `to` along a line at distance `plane` from the centre, running to the
    /// east-west side (`along_east`) or the north-south side. The bite out of
    /// a `Notch` never reaches the wall.
    pub fn span_along(
        self,
        along_east: bool,
        plane: f32,
        (from, to): (f32, f32),
    ) -> Option<(f32, f32)> {
        let span = match (self, along_east) {
            (Self::Round, _) => None,
            (Self::Full | Self::Notch, _) => Some((from, to)),
            (Self::AlongSouth, true) | (Self::AlongEast, false) => {
                Some((from, to.min(CHANNEL_HALF)))
            }
            (Self::AlongEast, true) | (Self::AlongSouth, false) => {
                (plane <= CHANNEL_HALF).then_some((from, to))
            }
        };
        span.filter(|(from, to)| to > from)
    }

    /// The quarter of the same cell a channel would run on into, past the
    /// cell's centre line, given the quarter it is in.
    fn behind(self, (sx, sz): (f32, f32)) -> Option<(f32, f32)> {
        match self {
            Self::AlongEast => Some((-sx, sz)),
            Self::AlongSouth => Some((sx, -sz)),
            _ => None,
        }
    }

    /// The pieces this shape is drawn from: along the shore, with the matching
    /// points the liquid and the floor stretch to from it.
    fn rings(self) -> Rings {
        let curve = |point: &dyn Fn(f32) -> Point| -> Vec<Point> {
            (0..=CURVE_PIECES)
                .map(|i| point(i as f32 / CURVE_PIECES as f32 * std::f32::consts::FRAC_PI_2))
                .collect()
        };
        // From a point out along a line until it reaches the edge of the
        // quarter furthest along.
        let to_edge = |(east, south): Point, reach: f32| {
            let far = reach / east.abs().max(south.abs());
            (east * far, south * far)
        };
        match self {
            Self::Round => {
                let shore =
                    curve(&|angle| (CHANNEL_HALF * angle.cos(), CHANNEL_HALF * angle.sin()));
                Rings {
                    liquid: vec![(0.0, 0.0); shore.len()],
                    floor: shore.iter().map(|&p| to_edge(p, HALF_CELL)).collect(),
                    shore,
                }
            }
            Self::Notch => {
                let from_corner = |p: Point| (HALF_CELL - p.0, HALF_CELL - p.1);
                let shore = curve(&|angle| {
                    from_corner((CHANNEL_HALF * angle.cos(), CHANNEL_HALF * angle.sin()))
                });
                Rings {
                    liquid: shore
                        .iter()
                        .map(|&p| from_corner(to_edge(from_corner(p), HALF_CELL)))
                        .collect(),
                    floor: vec![(HALF_CELL, HALF_CELL); shore.len()],
                    shore,
                }
            }
            Self::AlongEast => Rings {
                shore: vec![(0.0, CHANNEL_HALF), (HALF_CELL, CHANNEL_HALF)],
                liquid: vec![(0.0, 0.0), (HALF_CELL, 0.0)],
                floor: vec![(0.0, HALF_CELL), (HALF_CELL, HALF_CELL)],
            },
            Self::AlongSouth => Rings {
                shore: vec![(CHANNEL_HALF, 0.0), (CHANNEL_HALF, HALF_CELL)],
                liquid: vec![(0.0, 0.0), (0.0, HALF_CELL)],
                floor: vec![(HALF_CELL, 0.0), (HALF_CELL, HALF_CELL)],
            },
            Self::Full => Rings {
                shore: Vec::new(),
                liquid: Vec::new(),
                floor: Vec::new(),
            },
        }
    }
}

/// Matching lists of points: the shore, where the liquid stretches to from it
/// and where the floor does.
struct Rings {
    shore: Vec<Point>,
    liquid: Vec<Point>,
    floor: Vec<Point>,
}

/// The depth of a cell's bed at a point in a quarter.
#[derive(Clone, Copy)]
enum Bed {
    Flat(f32),
    /// Half deep in the middle of the cell, full at the east-west edge.
    SlopeEast,
    /// Half deep in the middle of the cell, full at the north-south edge.
    SlopeSouth,
}

impl Bed {
    fn depth(self, (east, south): Point) -> f32 {
        let slope = |along: f32| NARROW_DEPTH + (FULL_DEPTH - NARROW_DEPTH) * along / HALF_CELL;
        match self {
            Self::Flat(depth) => depth,
            Self::SlopeEast => slope(east),
            Self::SlopeSouth => slope(south),
        }
    }
}

/// Whether liquid carries on past this cell: it is liquid, or a wall.
fn wet(cell: Cell) -> bool {
    cell.is_liquid() || cell.is_wall()
}

/// The three other cells round the corner of `(x, y)` in a quarter, beside it,
/// ahead of it and across the corner.
fn around((x, y): (i64, i64), (sx, sz): (f32, f32)) -> [(i64, i64); 3] {
    let (dx, dy) = (sx as i64, sz as i64);
    [(x + dx, y), (x, y + dy), (x + dx, y + dy)]
}

/// The liquid in a quarter of a cell, if any: a liquid cell always has some,
/// and a wall where it meets any. Beside a wall, liquid stays in the same
/// shape: the wall only counts as liquid if the liquid comes round to it.
pub fn outline_at(level: &Level, (x, y): (i64, i64), quarter: (f32, f32)) -> Option<Outline> {
    let cell = level.cell(x, y);
    let [a, b, c] = around((x, y), quarter).map(|(x, y)| level.cell(x, y));
    if cell.is_liquid() {
        return Some(Outline::of(wet(a), wet(b), wet(c)));
    }
    if !cell.is_wall() || ![a, b, c].iter().any(|cell| cell.is_liquid()) {
        return None;
    }
    let joins =
        |near: Cell, other: Cell| near.is_liquid() || (near.is_wall() && wet(other) && wet(c));
    let (beside, ahead) = (joins(a, b), joins(b, a));
    if !beside && !ahead {
        return None;
    }
    Some(Outline::of(beside, ahead, wet(c)))
}

/// Whether a liquid cell is narrow: floor on two opposite sides.
fn is_narrow(level: &Level, x: i64, y: i64) -> bool {
    let wet_at = |x, y| wet(level.cell(x, y));
    (!wet_at(x, y - 1) && !wet_at(x, y + 1)) || (!wet_at(x + 1, y) && !wet_at(x - 1, y))
}

/// The bed of a quarter of the liquid cell `(x, y)`.
fn bed_at(level: &Level, (x, y): (i64, i64), quarter: (f32, f32), outline: Outline) -> Bed {
    if !is_narrow(level, x, y) {
        return Bed::Flat(FULL_DEPTH);
    }
    let [beside, ahead, _] = around((x, y), quarter);
    // The next cell of the channel, if it is narrow too, keeps the same depth.
    let carries_on = |(x, y): (i64, i64)| level.cell(x, y).is_liquid() && is_narrow(level, x, y);
    match outline {
        Outline::AlongEast if !carries_on(beside) => Bed::SlopeEast,
        Outline::AlongSouth if !carries_on(ahead) => Bed::SlopeSouth,
        _ => Bed::Flat(NARROW_DEPTH),
    }
}

/// The liquid and the floor round the corners of `(x, y)`, for its sides and
/// bed: the first of each found, nearest first.
pub fn surfaces_near(level: &Level, (x, y): (i64, i64)) -> (Surface, Surface) {
    const NEAR: [(i64, i64); 9] = [
        (0, 0),
        (0, -1),
        (1, 0),
        (0, 1),
        (-1, 0),
        (1, -1),
        (1, 1),
        (-1, 1),
        (-1, -1),
    ];
    let cells = || NEAR.iter().map(|&(dx, dy)| level.cell(x + dx, y + dy));
    let liquid = cells()
        .find(|cell| cell.is_liquid())
        .map_or(Surface::Water, Surface::of_flat);
    let floor = cells()
        .find(|&cell| cell.is_open() && !cell.is_liquid())
        .map_or(Surface::Floor, Surface::of_flat);
    (liquid, floor)
}

/// The light on a liquid cell at a point: between its corners, like floors.
fn cell_light(level: &Level, light: &LightGrid, (x, y): (i64, i64), point: Vec3) -> Vec3 {
    let east = (point.x / CELL_SIZE - x as f32 + 0.5).clamp(0.0, 1.0);
    let south = (point.z / CELL_SIZE - y as f32 + 0.5).clamp(0.0, 1.0);
    let here = light.at(x, y);
    let corner = |cx, cy| light.corner(level, cx, cy).unwrap_or(here);
    let north = corner(x, y).lerp(corner(x + 1, y), east);
    let below = corner(x, y + 1).lerp(corner(x + 1, y + 1), east);
    north.lerp(below, south)
}

/// One quarter of liquid being drawn.
pub struct Quarter<'a> {
    pub layer: Layer,
    pub centre: Vec3,
    pub quarter: (f32, f32),
    pub outline: Outline,
    pub liquid: Surface,
    pub floor: Surface,
    /// Whether to draw the floor beside the liquid.
    pub with_floor: bool,
    /// The vertex colour at a point, before shading.
    pub colour: &'a dyn Fn(Vec3) -> Vec3,
}

/// Draws the liquid cell at `(x, y)`, whose cell is remembered: lit if in
/// sight, otherwise as memory.
pub fn build_cell(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    const QUARTERS: [(f32, f32); 4] = [(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)];
    let level = scene.level;
    let in_sight = scene.sight.is_seen(x, y);
    let colour = |point: Vec3| {
        if in_sight {
            with_ambient(cell_light(level, scene.light, (x, y), point))
        } else {
            Vec3::splat(MEMORY_SHADE)
        }
    };
    let (liquid, floor) = surfaces_near(level, (x, y));
    for quarter in QUARTERS {
        let Some(outline) = outline_at(level, (x, y), quarter) else {
            continue;
        };
        let bed = bed_at(level, (x, y), quarter, outline);
        let drawn = Quarter {
            layer: if in_sight {
                Layer::Solid
            } else {
                Layer::Memory
            },
            centre: Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE),
            quarter,
            outline,
            liquid,
            floor,
            with_floor: true,
            colour: &colour,
        };
        drawn.build(layers, bed);
    }
}

impl Quarter<'_> {
    /// Draws liquid in a wall's quarter, which is a full cell deep throughout.
    pub fn build_in_wall(&self, layers: &mut Layers) {
        self.build(layers, Bed::Flat(FULL_DEPTH));
    }

    /// A channel running into a wall's cell stops at the centre line if the
    /// far side of the wall is dry. Where the wall is narrower than the channel
    /// (at its end) that shows from outside, so it is closed there, in the
    /// wall's picture. `(x, y)` is the wall's cell.
    pub fn close_behind_wall(&self, layers: &mut Layers, level: &Level, cell: (i64, i64)) {
        let Some(behind) = self.outline.behind(self.quarter) else {
            return;
        };
        if self.layer == Layer::Memory || outline_at(level, cell, behind).is_some() {
            return;
        }
        let (a, b, towards) = if self.outline == Outline::AlongEast {
            ((0.0, 0.0), (0.0, CHANNEL_HALF), (HALF_CELL, 0.0))
        } else {
            ((0.0, 0.0), (CHANNEL_HALF, 0.0), (0.0, HALF_CELL))
        };
        let bed = Bed::Flat(FULL_DEPTH);
        self.side(layers, Surface::Wall, (a, b), towards, bed);
    }

    /// The surface, then for what is in sight the bed and sides under it (what
    /// is remembered stays quiet: only the outline, flat), then the floor.
    fn build(&self, layers: &mut Layers, bed: Bed) {
        let in_sight = self.layer != Layer::Memory;
        if self.outline == Outline::Full {
            let corners = [
                (0.0, 0.0),
                (HALF_CELL, 0.0),
                (HALF_CELL, HALF_CELL),
                (0.0, HALF_CELL),
            ];
            self.surface(layers, corners);
            if in_sight {
                self.bed(layers, corners, bed);
            }
            return;
        }
        let Rings {
            shore,
            liquid,
            floor,
        } = self.outline.rings();
        for i in 0..shore.len() - 1 {
            let water = [shore[i], shore[i + 1], liquid[i + 1], liquid[i]];
            self.surface(layers, water);
            if in_sight {
                self.bed(layers, water, bed);
                let piece = (shore[i], shore[i + 1]);
                self.side(layers, self.floor, piece, liquid[i], bed);
            }
            if self.with_floor {
                let beside = [shore[i], shore[i + 1], floor[i + 1], floor[i]];
                self.flat(layers, self.floor, beside, 0.0, &|_| 1.0);
            }
        }
    }

    fn place(&self, (east, south): Point, height: f32) -> Vec3 {
        let (sx, sz) = self.quarter;
        self.centre + Vec3::new(sx * east, height, sz * south)
    }

    fn uv(&self, (east, south): Point) -> [f32; 2] {
        let (sx, sz) = self.quarter;
        flat_uv(sx * east, sz * south)
    }

    fn colour_at(&self, position: Vec3, shade: f32) -> Vec3 {
        (self.colour)(position) * shade
    }

    /// A flat piece at a height, shaded per corner.
    fn flat(
        &self,
        layers: &mut Layers,
        surface: Surface,
        corners: [Point; 4],
        height: f32,
        shade: &dyn Fn(Point) -> f32,
    ) {
        let positions = corners.map(|point| self.place(point, height));
        let colours = corners.map(|point| self.colour_at(self.place(point, height), shade(point)));
        layers.builder(self.layer, surface).free(
            positions,
            Vec3::Y,
            corners.map(|point| self.uv(point)),
            colours,
        );
    }

    fn surface(&self, layers: &mut Layers, corners: [Point; 4]) {
        self.flat(layers, self.liquid, corners, -SURFACE_DROP, &|_| 1.0);
    }

    fn bed(&self, layers: &mut Layers, corners: [Point; 4], bed: Bed) {
        let positions = corners.map(|point| self.place(point, -bed.depth(point)));
        let colours = corners.map(|point| {
            let depth = bed.depth(point);
            self.colour_at(self.place(point, -depth), bed_shade(depth))
        });
        layers.builder(self.layer, self.floor).free(
            positions,
            Vec3::Y,
            corners.map(|point| self.uv(point)),
            colours,
        );
    }

    /// The side of the channel along a piece of shore, facing the liquid at
    /// `towards`. It keeps the picture's scale, starting again from the top at
    /// the floor, so it is never stretched.
    fn side(
        &self,
        layers: &mut Layers,
        surface: Surface,
        (a, b): (Point, Point),
        towards: Point,
        bed: Bed,
    ) {
        let (sx, sz) = self.quarter;
        let length = Vec2::new(b.0 - a.0, b.1 - a.1).length();
        let inwards = Vec3::new(sx * (towards.0 - a.0), 0.0, sz * (towards.1 - a.1)).normalize();
        let start = (a.0 + a.1) / CELL_SIZE;
        let u1 = start + length / CELL_SIZE;
        let (depth_a, depth_b) = (bed.depth(a), bed.depth(b));
        let positions = [
            self.place(a, 0.0),
            self.place(b, 0.0),
            self.place(b, -depth_b),
            self.place(a, -depth_a),
        ];
        let uvs = [
            [start, 0.0],
            [u1, 0.0],
            [u1, depth_b / CELL_SIZE],
            [start, depth_a / CELL_SIZE],
        ];
        let shades = [
            depth_shade(0.0),
            depth_shade(0.0),
            depth_shade(depth_b),
            depth_shade(depth_a),
        ];
        let colours = std::array::from_fn(|i| self.colour_at(positions[i], shades[i]));
        layers
            .builder(self.layer, surface)
            .free(positions, inwards, uvs, colours);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How far from the middle of a cell the liquid reaches along the edge of a
    /// quarter: its east-west edge, or its north-south one.
    fn reach_on_side(outline: Option<Outline>, east_side: bool) -> f32 {
        match (outline, east_side) {
            (Some(Outline::Full), _) => HALF_CELL,
            (Some(Outline::Notch), _) => CHANNEL_HALF,
            (Some(Outline::AlongEast), true) | (Some(Outline::AlongSouth), false) => CHANNEL_HALF,
            _ => 0.0,
        }
    }

    /// Whatever the four cells round a grid corner are, the quarters touching
    /// it agree where the liquid crosses from one cell to the next.
    #[test]
    fn quarters_meet_their_neighbours() {
        const KINDS: [Cell; 3] = [Cell::Floor, Cell::Wall, Cell::Water];
        for &nw in &KINDS {
            for &ne in &KINDS {
                for &sw in &KINDS {
                    for &se in &KINDS {
                        let mut level = Level::new(0, 0, 2, 2);
                        level.set(0, 0, nw);
                        level.set(1, 0, ne);
                        level.set(0, 1, sw);
                        level.set(1, 1, se);
                        let cases = (nw, ne, sw, se);
                        let at = |cell, quarter| outline_at(&level, cell, quarter);
                        let [a, b, c, d] = [
                            at((0, 0), (1.0, 1.0)),
                            at((1, 0), (-1.0, 1.0)),
                            at((0, 1), (1.0, -1.0)),
                            at((1, 1), (-1.0, -1.0)),
                        ];
                        let side = reach_on_side;
                        assert_eq!(side(a, true), side(b, true), "north edge: {cases:?}");
                        assert_eq!(side(c, true), side(d, true), "south edge: {cases:?}");
                        assert_eq!(side(a, false), side(c, false), "west edge: {cases:?}");
                        assert_eq!(side(b, false), side(d, false), "east edge: {cases:?}");
                    }
                }
            }
        }
    }

    fn pool(cells: &[(i64, i64)]) -> Level {
        let mut level = Level::new(-4, -4, 9, 9);
        for y in -4..=4 {
            for x in -4..=4 {
                level.set(x, y, Cell::Floor);
            }
        }
        for &(x, y) in cells {
            level.set(x, y, Cell::Water);
        }
        level
    }

    #[test]
    fn a_channel_is_narrow_and_curves_round_its_corner() {
        const NE: (f32, f32) = (1.0, -1.0);
        const SW: (f32, f32) = (-1.0, 1.0);
        // Running east-west.
        let level = pool(&[(-1, 0), (0, 0), (1, 0)]);
        assert!(is_narrow(&level, 0, 0));
        assert_eq!(outline_at(&level, (0, 0), NE), Some(Outline::AlongEast));
        // Coming from the north and turning east, with the outside of the
        // bend reaching past the middle of the cell.
        let level = pool(&[(0, -1), (0, 0), (1, 0)]);
        assert!(!is_narrow(&level, 0, 0));
        assert_eq!(outline_at(&level, (0, 0), NE), Some(Outline::Notch));
        assert_eq!(outline_at(&level, (0, 0), SW), Some(Outline::Round));
        // Floor isn't liquid.
        assert_eq!(outline_at(&level, (2, 0), NE), None);
    }

    #[test]
    fn narrow_channels_are_flat_until_they_meet_something_deeper() {
        let at_the_seam = (HALF_CELL, 0.0);
        let depth_at_east_seam = |level: &Level, cell| {
            let quarter = (1.0, -1.0);
            let outline = outline_at(level, cell, quarter).unwrap();
            bed_at(level, cell, quarter, outline).depth(at_the_seam)
        };
        // Along a channel it stays half deep.
        let mut level = pool(&[(-1, 0), (0, 0), (1, 0)]);
        assert_eq!(depth_at_east_seam(&level, (0, 0)), NARROW_DEPTH);
        // Running into a wall it reaches full depth there...
        level.set(1, 0, Cell::Wall);
        assert_eq!(depth_at_east_seam(&level, (0, 0)), FULL_DEPTH);
        // ...and into something wider too: (1, 0) has liquid north and east.
        let level = pool(&[(-1, 0), (0, 0), (1, 0), (1, -1), (2, 0)]);
        assert_eq!(depth_at_east_seam(&level, (0, 0)), FULL_DEPTH);
    }

    #[test]
    fn liquid_beside_a_wall_runs_to_its_foot() {
        // A wall along the north of a row of water.
        let mut level = pool(&[(-1, 1), (0, 1), (1, 1)]);
        for x in -2..=2 {
            level.set(x, 0, Cell::Wall);
        }
        let south_east = (1.0, 1.0);
        assert_eq!(
            outline_at(&level, (0, 0), south_east),
            Some(Outline::Full),
            "all along it"
        );
        // Beside the water's end it is only as wide as the channel, and past
        // it there is none.
        assert_eq!(
            outline_at(&level, (1, 0), south_east),
            Some(Outline::AlongSouth)
        );
        assert_eq!(outline_at(&level, (2, 0), south_east), None);
        // And its north side stays dry.
        assert_eq!(outline_at(&level, (0, 0), (1.0, -1.0)), None);
    }

    #[test]
    fn deeper_is_darker_and_the_bed_darker_than_the_sides() {
        let close = |a: f32, b: f32| (a - b).abs() < 0.001;
        assert!(close(depth_shade(NARROW_DEPTH), 0.6));
        assert!(close(depth_shade(FULL_DEPTH), 0.4));
        assert!(close(bed_shade(NARROW_DEPTH), 0.4));
        assert!(close(bed_shade(FULL_DEPTH), 0.2));
    }

    #[test]
    fn a_channel_runs_on_behind_its_centre_line() {
        let quarter = (-1.0, 1.0);
        assert_eq!(Outline::AlongEast.behind(quarter), Some((1.0, 1.0)));
        assert_eq!(Outline::AlongSouth.behind(quarter), Some((-1.0, -1.0)));
        assert_eq!(Outline::Full.behind(quarter), None);
    }

    #[test]
    fn a_wall_goes_down_only_where_liquid_touches_it() {
        let span = |outline: Outline| outline.span_along(true, 1.0, (0.0, HALF_CELL));
        assert_eq!(span(Outline::Full), Some((0.0, HALF_CELL)));
        assert_eq!(span(Outline::AlongSouth), Some((0.0, CHANNEL_HALF)));
        assert_eq!(span(Outline::Round), None);
    }
}
