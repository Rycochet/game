//! Farms: a farmhouse (a building, see `building`) in the middle of up to eight
//! fields, one in each of the eight places round it, one of which may be a pond
//! instead.
//!
//! The house's floors keep their cells: the ground floor's level just reaches
//! further, to north and west into negative coordinates, so stairs still line
//! up. Every field is a rectangle with a fence round it, and one gate as near
//! the front door as a side of the fence gets. Dirt tracks join the front door,
//! the gates and the edge of the level.

use serde::{Deserialize, Serialize};

use super::SiteLevel;
use super::building::{self, BuildingOptions};
use super::grid::{STEPS, chance, roll};
use super::town::Rect;
use crate::dungeon::{Cell, Level};
use crate::rng::{Rng, Seed};

/// Ground left between the house's yard, each field and the next. At least 1
/// must be walkable, or a fence jumped could land on nothing.
const LANE: i64 = 2;

/// Ground left round the outside of the fields.
const BORDER: i64 = 3;

/// The smallest a field can be across: 8 cells sown and the fence each side.
const MIN_FIELD: i64 = 10;

/// How much room a place north, east, south or west of the house may have.
const PLACE_SIZE: (i64, i64) = (12, 22);

/// Percentage chance that a farm with at least 2 places has a pond in one.
const POND_PERCENT: u32 = 33;

/// How many blobs a pond is made of, and how far each reaches from its middle.
const POND_BLOBS: (i64, i64) = (3, 6);
const POND_REACH: (i64, i64) = (2, 6);

/// Percentage chance a farm is all crops or all pasture; otherwise each field
/// chooses.
const ALL_CROPS_PERCENT: u64 = 50;
const ALL_PASTURE_PERCENT: u64 = 25;

/// Of each 100 cells of a ruined farm's fence, how many start a gap, per level
/// of ruin; and how long a gap may be.
const FENCE_GAP_PERCENT: u32 = 4;
const FENCE_GAP: (i64, i64) = (1, 3);

/// Options for the farm generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FarmOptions {
    pub house: BuildingOptions,
    /// How many of the places round the house are used, an inclusive range.
    pub fields: (i64, i64),
}

/// What is needed to choose what grows in a field. Which stage it is at is
/// the client's choice: it is only looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldLook {
    /// The cells sown, inside the fence.
    pub rect: Rect,
    /// Grazed rather than cropped.
    pub livestock: bool,
    /// A number for looks, so fields differ.
    pub variety: u32,
}

impl FieldLook {
    /// Whether a cell is in the field.
    pub fn contains(&self, x: i64, y: i64) -> bool {
        let Rect {
            x: left,
            y: top,
            width,
            height,
        } = self.rect;
        (left..left + width).contains(&x) && (top..top + height).contains(&y)
    }
}

/// A farm's plan. Fields depend on where the house's door is, so its ground
/// floor is made at once and kept.
#[derive(Clone, Debug)]
pub struct Plan {
    house: building::Plan,
    ground: SiteLevel,
}

impl Plan {
    pub fn new(options: &FarmOptions, seed: Seed) -> Self {
        let house = building::Plan::new(&options.house, seed);
        let ground = lay_out(options, seed, &house);
        Self { house, ground }
    }

    pub fn levels(&self) -> u32 {
        self.house.floors()
    }

    pub fn has_basement(&self) -> bool {
        self.house.has_basement()
    }

    pub fn level(&self, index: u32) -> SiteLevel {
        match index {
            0 => self.ground.clone(),
            _ => self.house.level(index),
        }
    }

    /// The house's size, without the ground round it.
    pub fn footprint(&self) -> (i64, i64) {
        self.house.footprint()
    }

    /// How many fields there are, without any pond.
    pub fn fields(&self) -> usize {
        self.ground.fields.len()
    }
}

/// A rectangle by its edges, the far ones excluded.
#[derive(Clone, Copy, Debug)]
struct Plot {
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
}

/// The ground floor, with everything round the house.
fn lay_out(options: &FarmOptions, seed: Seed, house: &building::Plan) -> SiteLevel {
    let mut rng = seed.derive("farm").rng();
    let ruin = options.house.ruin;
    let inner = house.level(0);
    let (_, _, width, height) = inner.level.bounds();
    let entrance = inner.entrance.expect("a house has a way in");

    let plots = choose_plots(options, &mut rng, width, height);
    let pond = plots.len() >= 2 && chance(&mut rng, POND_PERCENT);
    let farming = match rng.range_u64(0, 100) {
        n if n < ALL_CROPS_PERCENT => Some(false),
        n if n < ALL_CROPS_PERCENT + ALL_PASTURE_PERCENT => Some(true),
        _ => None,
    };

    let mut level = surroundings(&inner.level, &plots);
    let mut fields = Vec::new();
    let mut gates = Vec::new();
    for (index, &plot) in plots.iter().enumerate() {
        if pond && index + 1 == plots.len() {
            dig_pond(&mut level, &mut rng, plot);
            continue;
        }
        let gate = fence_field(&mut level, &mut rng, plot, entrance, ruin);
        gates.push(gate);
        fields.push(FieldLook {
            rect: Rect {
                x: plot.x0 + 1,
                y: plot.y0 + 1,
                width: plot.x1 - plot.x0 - 2,
                height: plot.y1 - plot.y0 - 2,
            },
            livestock: farming.unwrap_or_else(|| chance(&mut rng, 40)),
            variety: rng.next_u64() as u32,
        });
    }
    lay_tracks(&mut level, house, entrance, &gates);

    SiteLevel {
        level,
        fields,
        ..inner
    }
}

/// Which of the eight places round the house are used, and the plot of each.
/// All the places on one side share how far out they may reach, so nothing
/// comes nearer another than `LANE`.
fn choose_plots(options: &FarmOptions, rng: &mut Rng, width: i64, height: i64) -> Vec<Plot> {
    let count = roll(rng, options.fields.0, options.fields.1).clamp(0, 8) as usize;
    let mut places = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    for i in (1..places.len()).rev() {
        places.swap(i, rng.range_u64(0, i as u64 + 1) as usize);
    }
    let reach = [(); 4].map(|_| roll(rng, PLACE_SIZE.0, PLACE_SIZE.1));
    let [west, north, east, south] = reach;
    // The room in a column or row: from `from`, this big.
    let span = |side: i64, size: i64, before: i64, after: i64| match side {
        -1 => (-LANE - before, before),
        0 => (0, size),
        _ => (size + LANE, after),
    };
    places[..count]
        .iter()
        .map(|&(column, row)| {
            let (x, room_x) = span(column, width, west, east);
            let (y, room_y) = span(row, height, north, south);
            let wide = roll(rng, MIN_FIELD, room_x);
            let tall = roll(rng, MIN_FIELD, room_y);
            // Up against the house's side of the place, or anywhere along it.
            let along = |rng: &mut Rng, side: i64, from: i64, room: i64, size: i64| match side {
                -1 => from + room - size,
                0 => roll(rng, from, from + room - size),
                _ => from,
            };
            let x0 = along(rng, column, x, room_x, wide);
            let y0 = along(rng, row, y, room_y, tall);
            Plot {
                x0,
                y0,
                x1: x0 + wide,
                y1: y0 + tall,
            }
        })
        .collect()
}

/// The house's level on bare ground reaching round all the plots.
fn surroundings(house: &Level, plots: &[Plot]) -> Level {
    let (_, _, width, height) = house.bounds();
    let edge = |near: fn(&Plot) -> i64, pick: fn(i64, i64) -> i64, house: i64, out: i64| {
        plots.iter().map(|plot| near(plot) + out).fold(house, pick)
    };
    let x0 = edge(|p| p.x0, i64::min, 0, -BORDER);
    let y0 = edge(|p| p.y0, i64::min, 0, -BORDER);
    let x1 = edge(|p| p.x1, i64::max, width, BORDER);
    let y1 = edge(|p| p.y1, i64::max, height, BORDER);
    let mut level = Level::new(x0, y0, x1 - x0, y1 - y0);
    for y in y0..y1 {
        for x in x0..x1 {
            if house.contains(x, y) {
                level.set(x, y, house.cell(x, y));
                level.set_room(x, y, house.room(x, y));
            } else {
                level.set(x, y, Cell::Ground);
            }
        }
    }
    level
}

/// The cells round a plot's edge, going round it from its north-west corner.
fn fence_line(plot: Plot) -> Vec<(i64, i64)> {
    let Plot { x0, y0, x1, y1 } = plot;
    let top = (x0..x1 - 1).map(|x| (x, y0));
    let right = (y0..y1 - 1).map(|y| (x1 - 1, y));
    let bottom = (x0 + 1..x1).rev().map(|x| (x, y1 - 1));
    let left = (y0 + 1..y1).rev().map(|y| (x0, y));
    top.chain(right).chain(bottom).chain(left).collect()
}

/// Sows a plot and fences it, with its gate on the part of the fence (not a
/// corner) nearest `door`. Gives the cell outside the gate.
fn fence_field(
    level: &mut Level,
    rng: &mut Rng,
    plot: Plot,
    door: (i64, i64),
    ruin: u8,
) -> (i64, i64) {
    let Plot { x0, y0, x1, y1 } = plot;
    for y in y0 + 1..y1 - 1 {
        for x in x0 + 1..x1 - 1 {
            level.set(x, y, Cell::Field);
        }
    }
    let line = fence_line(plot);
    let corner = |&(x, y): &(i64, i64)| (x == x0 || x == x1 - 1) && (y == y0 || y == y1 - 1);
    let distance = |&(x, y): &(i64, i64)| (x - door.0).pow(2) + (y - door.1).pow(2);
    let gate = line
        .iter()
        .filter(|spot| !corner(spot))
        .min_by_key(|spot| distance(spot))
        .copied()
        .expect("a fence has sides");
    let mut gap = 0;
    for &(x, y) in &line {
        if ruin > 0 && gap == 0 && chance(rng, FENCE_GAP_PERCENT * u32::from(ruin)) {
            gap = roll(rng, FENCE_GAP.0, FENCE_GAP.1);
        }
        let cell = if (x, y) == gate {
            Cell::FieldGate
        } else if gap > 0 {
            Cell::Ground
        } else {
            Cell::Fence
        };
        level.set(x, y, cell);
        gap = (gap - 1).max(0);
    }
    let out = if gate.1 == y0 {
        (0, -1)
    } else if gate.0 == x1 - 1 {
        (1, 0)
    } else if gate.1 == y1 - 1 {
        (0, 1)
    } else {
        (-1, 0)
    };
    (gate.0 + out.0, gate.1 + out.1)
}

/// Fills part of a plot with water, in one piece of no particular shape.
fn dig_pond(level: &mut Level, rng: &mut Rng, plot: Plot) {
    let Plot { x0, y0, x1, y1 } = plot;
    let (width, height) = (x1 - x0, y1 - y0);
    let at = |x: i64, y: i64| ((y - y0) * width + (x - x0)) as usize;
    let mut water = vec![false; (width * height) as usize];
    // Each blob starts inside what is there, so they are all one piece.
    let mut middle = ((x0 + x1) / 2, (y0 + y1) / 2);
    for _ in 0..roll(rng, POND_BLOBS.0, POND_BLOBS.1) {
        let (reach_x, reach_y) = (
            roll(rng, POND_REACH.0, POND_REACH.1),
            roll(rng, POND_REACH.0, POND_REACH.1),
        );
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = ((x - middle.0) * reach_y, (y - middle.1) * reach_x);
                if dx * dx + dy * dy <= (reach_x * reach_y).pow(2) {
                    water[at(x, y)] = true;
                }
            }
        }
        let (x, y) = (
            roll(rng, middle.0 - reach_x, middle.0 + reach_x),
            roll(rng, middle.1 - reach_y, middle.1 + reach_y),
        );
        middle = (x.clamp(x0, x1 - 1), y.clamp(y0, y1 - 1));
        if !water[at(middle.0, middle.1)] {
            middle = ((x0 + x1) / 2, (y0 + y1) / 2);
        }
    }
    // Land inside it, with no way off, would be out of reach: flood from the
    // plot's edge, and what isn't reached is water.
    let mut dry = vec![false; water.len()];
    let mut stack: Vec<(i64, i64)> = fence_line(plot)
        .into_iter()
        .filter(|&(x, y)| !water[at(x, y)])
        .collect();
    while let Some((x, y)) = stack.pop() {
        if !dry[at(x, y)] {
            dry[at(x, y)] = true;
            for (dx, dy) in STEPS {
                let (nx, ny) = (x + dx, y + dy);
                let inside = (x0..x1).contains(&nx) && (y0..y1).contains(&ny);
                if inside && !water[at(nx, ny)] {
                    stack.push((nx, ny));
                }
            }
        }
    }
    for y in y0..y1 {
        for x in x0..x1 {
            if !dry[at(x, y)] {
                level.set(x, y, Cell::Water);
            }
        }
    }
}

/// Dirt tracks over bare ground: from the front door to the nearest edge of
/// the level, and from each gate to the nearest track.
fn lay_tracks(level: &mut Level, house: &building::Plan, door: (i64, i64), gates: &[(i64, i64)]) {
    let shape = house.ground();
    let at_edge = |level: &Level, x: i64, y: i64| {
        STEPS.iter().any(|(dx, dy)| !level.contains(x + dx, y + dy))
    };
    let main = route(level, |x, y| shape.contains(x, y), door, at_edge);
    let is_track = |level: &Level, x: i64, y: i64| level.cell(x, y) == Cell::DirtTrack;
    for (x, y) in main {
        level.set(x, y, Cell::DirtTrack);
    }
    for &outside in gates {
        for (x, y) in route(level, |x, y| shape.contains(x, y), outside, is_track) {
            level.set(x, y, Cell::DirtTrack);
        }
    }
}

/// The shortest way from `from` across bare ground or track to a cell `done`
/// accepts, with every cell on it. Nothing if there is none.
fn route(
    level: &Level,
    forbidden: impl Fn(i64, i64) -> bool,
    from: (i64, i64),
    done: impl Fn(&Level, i64, i64) -> bool,
) -> Vec<(i64, i64)> {
    let (min_x, min_y, width, height) = level.bounds();
    let at = |x: i64, y: i64| ((y - min_y) * width + (x - min_x)) as usize;
    let mut came_from: Vec<Option<(i64, i64)>> = vec![None; (width * height) as usize];
    came_from[at(from.0, from.1)] = Some(from);
    let mut queue = std::collections::VecDeque::from([from]);
    while let Some((x, y)) = queue.pop_front() {
        if done(level, x, y) {
            let mut path = vec![(x, y)];
            let mut spot = (x, y);
            while spot != from {
                spot = came_from[at(spot.0, spot.1)].expect("came from somewhere");
                path.push(spot);
            }
            return path;
        }
        for (dx, dy) in STEPS {
            let (nx, ny) = (x + dx, y + dy);
            let cell = level.cell(nx, ny);
            if level.contains(nx, ny)
                && matches!(cell, Cell::Ground | Cell::DirtTrack)
                && !forbidden(nx, ny)
                && came_from[at(nx, ny)].is_none()
            {
                came_from[at(nx, ny)] = Some((x, y));
                queue.push_back((nx, ny));
            }
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::super::grid::Reach;
    use super::super::{Generator, Site, StairsDirection, leads_out};
    use super::*;
    use crate::poi::PoiKind;

    fn farms(ruin: u8) -> impl Iterator<Item = Site> {
        (0..150).map(move |n| {
            let kind = PoiKind::Farm;
            let generator = Generator::for_poi(kind).with_ruin(ruin);
            Site::with_generator(kind, Seed::new(n * 37 + 5), generator.version(), generator)
                .expect("the current version runs")
        })
    }

    fn cells_of(level: &Level) -> impl Iterator<Item = (i64, i64)> + use<> {
        let (min_x, min_y, width, height) = level.bounds();
        (min_y..min_y + height).flat_map(move |y| (min_x..min_x + width).map(move |x| (x, y)))
    }

    /// One to eight fields, or a pond too; all rectangles of at least 8 by 8
    /// fenced all round, with one gate, on a side and not a corner.
    #[test]
    fn fields_are_fenced_rectangles_with_one_gate() {
        let (mut most, mut ponds) = (0, 0);
        for site in farms(0) {
            let ground = site.level(0);
            let level = &ground.level;
            let water = cells_of(level).any(|(x, y)| level.cell(x, y) == Cell::Water);
            let count = ground.fields.len() + usize::from(water);
            assert!((1..=8).contains(&count), "{count}");
            assert!(!ground.fields.is_empty());
            most = most.max(count);
            ponds += usize::from(water);
            for field in &ground.fields {
                let Rect {
                    x,
                    y,
                    width,
                    height,
                } = field.rect;
                assert!(width >= 8 && height >= 8, "{field:?}");
                let plot = Plot {
                    x0: x - 1,
                    y0: y - 1,
                    x1: x + width + 1,
                    y1: y + height + 1,
                };
                assert!(cells_of(level).all(|(cx, cy)| {
                    field.contains(cx, cy) == (level.cell(cx, cy) == Cell::Field)
                        || !field.contains(cx, cy)
                }));
                let line = fence_line(plot);
                let gates: Vec<_> = line
                    .iter()
                    .filter(|&&(cx, cy)| level.cell(cx, cy) == Cell::FieldGate)
                    .collect();
                assert_eq!(gates.len(), 1, "{field:?}");
                let &&(gx, gy) = &gates[0];
                let x_side = gx == plot.x0 || gx == plot.x1 - 1;
                let y_side = gy == plot.y0 || gy == plot.y1 - 1;
                assert!(x_side != y_side, "a gate on a corner");
                assert!(line.iter().all(|&(cx, cy)| {
                    matches!(level.cell(cx, cy), Cell::Fence | Cell::FieldGate)
                }));
            }
        }
        assert_eq!(most, 8);
        assert!((20..80).contains(&ponds), "{ponds}");
    }

    /// The gate is as near the door as any cell of the fence's sides.
    #[test]
    fn gates_are_nearest_the_front_door() {
        for site in farms(0) {
            let ground = site.level(0);
            let level = &ground.level;
            let door = ground.entrance.expect("a way in");
            let distance = |x: i64, y: i64| (x - door.0).pow(2) + (y - door.1).pow(2);
            for field in &ground.fields {
                let Rect {
                    x,
                    y,
                    width,
                    height,
                } = field.rect;
                let sides = (x..x + width)
                    .flat_map(|cx| [(cx, y - 1), (cx, y + height)])
                    .chain((y..y + height).flat_map(|cy| [(x - 1, cy), (x + width, cy)]));
                let (gate, best) = sides.fold(((0, 0), i64::MAX), |(gate, best), (cx, cy)| {
                    let gate = if level.cell(cx, cy) == Cell::FieldGate {
                        (cx, cy)
                    } else {
                        gate
                    };
                    (gate, best.min(distance(cx, cy)))
                });
                assert_eq!(distance(gate.0, gate.1), best);
            }
        }
    }

    /// Nothing touches the house, or another field or pond; every cell and
    /// gate can be walked to without a jump; a track leads out.
    #[test]
    fn everything_keeps_clear_and_can_be_walked_to() {
        for ruin in [0, 3, 5] {
            for site in farms(ruin) {
                let ground = site.level(0);
                let level = &ground.level;
                let start = ground.entrance.expect("a way in");
                assert_eq!(level.cell(start.0, start.1), Cell::DirtTrack);
                let mut reach = Reach::new(level);
                reach.spread(level, &[start]);
                let farmed = |cell: Cell| {
                    matches!(
                        cell,
                        Cell::Field | Cell::Fence | Cell::FieldGate | Cell::Water
                    )
                };
                let house = cells_of(level)
                    .filter(|&(x, y)| level.room(x, y).is_some() || level.cell(x, y).is_wall());
                for (x, y) in house {
                    for (dx, dy) in (-1..=1).flat_map(|d| (-1..=1).map(move |e| (d, e))) {
                        assert!(!farmed(level.cell(x + dx, y + dy)), "at ({x}, {y})");
                    }
                }
                let mut out = false;
                for (x, y) in cells_of(level) {
                    assert!(!level.is_walkable(x, y) || reach.is(x, y), "({x}, {y})");
                    out |= level.cell(x, y) == Cell::DirtTrack && leads_out(level, x, y);
                    // Different fields' fences have ground between them.
                    if level.cell(x, y) == Cell::Fence {
                        for field in &ground.fields {
                            let Rect {
                                x: fx,
                                y: fy,
                                width,
                                height,
                            } = field.rect;
                            let beside = (fx - 2..=fx + width + 1).contains(&x)
                                && (fy - 2..=fy + height + 1).contains(&y);
                            let on_it = (fx - 1..=fx + width).contains(&x)
                                && (fy - 1..=fy + height).contains(&y);
                            assert!(!beside || on_it, "fences touch at ({x}, {y})");
                        }
                    }
                }
                assert!(out, "no track leads out");
            }
        }
    }

    /// Two floors (a ruin may lose the upper one), and sometimes a basement,
    /// joined by stairs on the same cell.
    #[test]
    fn farmhouses_have_two_floors_and_some_a_basement() {
        let mut basements = 0;
        for site in farms(0) {
            let (storeys, basement) = if site.has_basement() {
                basements += 1;
                (site.levels() - 1, true)
            } else {
                (site.levels(), false)
            };
            assert_eq!(storeys, 2);
            if basement {
                let last = site.levels() - 1;
                let below = site.level(last);
                assert_eq!(below.stairs.len(), 1);
                let way_up = below.stairs[0];
                assert_eq!(way_up.direction, StairsDirection::Up);
                let down = site.level(0).stairs;
                let down: Vec<_> = down
                    .iter()
                    .filter(|s| s.direction == StairsDirection::Down)
                    .collect();
                assert_eq!(down.len(), 1);
                assert_eq!((down[0].x, down[0].y), (way_up.x, way_up.y));
                assert_eq!(down[0].to, super::super::Exit::Level(last));
                let mut reach = Reach::new(&below.level);
                reach.spread(&below.level, &[(way_up.x, way_up.y)]);
                assert!(
                    cells_of(&below.level)
                        .all(|(x, y)| !below.level.is_walkable(x, y) || reach.is(x, y))
                );
            }
        }
        assert!((40..110).contains(&basements), "{basements}");
    }

    /// However much of a ruin is gone, its basement is the last level and the
    /// ground floor's stairs still go there.
    #[test]
    fn a_ruins_basement_stays_joined() {
        for ruin in 1..=5 {
            for site in farms(ruin).filter(Site::has_basement) {
                let last = site.levels() - 1;
                let ground = site.level(0);
                let down = ground
                    .stairs
                    .iter()
                    .find(|s| s.direction == StairsDirection::Down)
                    .expect("stairs down");
                assert_eq!(down.to, super::super::Exit::Level(last));
                let mut reach = Reach::new(&ground.level);
                reach.spread(&ground.level, &[ground.entrance.expect("a way in")]);
                assert!(reach.is(down.x, down.y));
            }
        }
    }

    #[test]
    fn ruined_farms_lose_stretches_of_fence() {
        let fence = |site: &Site| {
            let level = site.level(0).level;
            cells_of(&level)
                .filter(|&(x, y)| level.cell(x, y) == Cell::Fence)
                .count()
        };
        let whole: usize = farms(0).map(|site| fence(&site)).sum();
        let ruined: usize = farms(5).map(|site| fence(&site)).sum();
        assert!(ruined * 10 < whole * 9, "{ruined} of {whole}");
    }

    #[test]
    fn the_same_seed_makes_the_same_farm() {
        let (a, b) = (
            Site::new(PoiKind::Farm, Seed::new(7)),
            Site::new(PoiKind::Farm, Seed::new(7)),
        );
        assert_eq!(a.level(0), b.level(0));
    }
}
