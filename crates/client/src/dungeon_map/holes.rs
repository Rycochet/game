//! Holes in the floor (`Cell::Hole`), built as shapes rather than pictures.
//!
//! A hole cell is drawn in four quarters, as walls are. Each quarter has what
//! is left of the floor beside it, the hole's side going down from that floor's
//! edge, and a black base: the client doesn't have the level below, so through
//! a hole it is only dark. The floor's edge wanders by hashes of position, and
//! where it crosses from one cell to the next the point depends only on that
//! place, whichever cell asks, so touching holes read as one hole.
//!
//! In a built place the side is the thickness of a floor; in a natural one it
//! goes the height of a wall, fading to black.

use bevy::prelude::*;
use shared::dungeon::{Cell, Level};

use super::mesh::{MEMORY_SHADE, Surface, corner_light, with_ambient};
use super::theme::Shape;
use super::walls::{Layer, Layers, Scene, flat_uv, noise};
use super::{CELL_SIZE, WALL_HEIGHT};

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// How far the floor's edge stands in from the side of a hole's cell, least and
/// most. It stays inside the cell, so a hole beside floor looks round.
const INSET_MIN: f32 = CELL_SIZE * 0.06;
const INSET_MAX: f32 = CELL_SIZE * 0.24;

/// How far the edge wanders from a straight line, as a part of its inset.
const WANDER: f32 = 0.3;

/// How thick a floor is, in a building.
const FLOOR_THICKNESS: f32 = CELL_SIZE * 0.2;

/// Shading of a hole's side: darker than the floor at its top, as stairs' are.
const SIDE_TOP_SHADE: f32 = 0.55;
const SIDE_BOTTOM_SHADE: f32 = 0.25;

/// The quarters of a cell, as the directions they lie in (`x` east, `y` south).
const QUARTERS: [(i64, i64); 4] = [(1, -1), (1, 1), (-1, 1), (-1, -1)];

/// A point in a quarter: how far from the cell's centre towards the quarter's
/// east-west side and its north-south side.
type Point = (f32, f32);

/// An inset from a hash of the numbers given.
fn inset(a: i64, b: i64, c: i64) -> f32 {
    let wander = noise(a, b, c) * 0.5 + 0.5;
    INSET_MIN + (INSET_MAX - INSET_MIN) * wander
}

/// What is left of the floor in a quarter of a hole's cell.
struct Remains {
    /// The floor, as points round it starting from the cell's corner, from
    /// which all of it can be seen in a straight line.
    floor: Vec<Point>,
    /// The floor's edge along the hole, which is part of `floor`.
    edge: Vec<Point>,
}

/// What is left of the floor in the quarter of cell `(x, y)` lying towards
/// `(sx, sy)`: `None` if it is all hole. Which cells count as hole decides the
/// shape, and off the level's edge counts as hole, so no floor is left along it;
/// see the module's notes on how quarters meet.
fn remains(level: &Level, (x, y): (i64, i64), (sx, sy): (i64, i64)) -> Option<Remains> {
    let is_hole = |x, y| !level.contains(x, y) || level.cell(x, y) == Cell::Hole;
    let side_open = is_hole(x + sx, y);
    let end_open = is_hole(x, y + sy);
    let corner = (HALF_CELL, HALF_CELL);
    // The corner of the grid this quarter touches.
    let (gx, gy) = (x + i64::from(sx > 0), y + i64::from(sy > 0));

    // On the edge between this cell and the next, some way from the grid's
    // corner. Both cells work out the same point.
    let across_side = HALF_CELL - inset(gx, gy, if sy > 0 { 1 } else { 2 });
    let across_end = HALF_CELL - inset(gx, gy, if sx > 0 { 3 } else { 4 });
    // Where the edge crosses the middle of this cell, beside solid floor.
    let by_side = HALF_CELL - inset(x, y, 5 + i64::from(sx > 0));
    let by_end = HALF_CELL - inset(x, y, 7 + i64::from(sy > 0));
    // A point for it to wander through, between those: near enough their
    // line that all the floor can still be seen from the grid's corner.
    let quarter = (sx + 2) * 10 + sy;
    let wander =
        |from_side: f32, salt: i64| from_side * (1.0 + WANDER * noise(x, y, salt + quarter));
    let (in_side, in_end) = (HALF_CELL - by_side, HALF_CELL - by_end);
    let far_x = wander(in_side, 20);
    let far_y = wander(in_end, 40);

    let edge = match (side_open, end_open) {
        // Floor on both sides: the edge rounds the hole's corner.
        (false, false) => vec![
            (by_side, 0.0),
            (HALF_CELL - far_x, HALF_CELL - far_y),
            (0.0, by_end),
        ],
        // Floor on one side: the edge runs along it to the next hole cell.
        (false, true) => vec![
            (by_side, 0.0),
            (HALF_CELL - far_x, HALF_CELL * 0.5),
            (across_end, HALF_CELL),
        ],
        (true, false) => vec![
            (0.0, by_end),
            (HALF_CELL * 0.5, HALF_CELL - far_y),
            (HALF_CELL, across_side),
        ],
        // Hole both ways: only a bite of floor from the diagonal cell, if any.
        (true, true) => {
            if is_hole(x + sx, y + sy) {
                return None;
            }
            vec![
                (HALF_CELL, across_side),
                (HALF_CELL - far_x * 0.7, HALF_CELL - far_y * 0.7),
                (across_end, HALF_CELL),
            ]
        }
    };
    // Round the outside of the quarter from where the edge ends to where it
    // starts, by the corner.
    let mut floor = vec![corner];
    let (start, end) = (edge[0], edge[edge.len() - 1]);
    if end.0 == 0.0 {
        floor.push((0.0, HALF_CELL));
    }
    floor.extend(edge.iter().rev());
    if start.1 == 0.0 {
        floor.push((HALF_CELL, 0.0));
    }
    if start.0 == 0.0 {
        floor.push((0.0, HALF_CELL));
    }
    floor.dedup();
    Some(Remains { floor, edge })
}

/// Which picture a hole's floor has: that of the cell beside it it belongs to.
fn floor_surface(level: &Level, (x, y): (i64, i64), (sx, sy): (i64, i64)) -> Surface {
    [(x + sx, y), (x, y + sy), (x + sx, y + sy)]
        .iter()
        .map(|&(x, y)| level.cell(x, y))
        .find(|&cell| cell != Cell::Hole && cell.is_open() && !cell.is_liquid())
        .map_or(Surface::Floor, Surface::of_flat)
}

/// Draws the hole at `(x, y)`, whose cell is remembered: only the floor round
/// it if that is all that is known now.
pub fn build_hole(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let in_sight = scene.sight.is_seen(x, y);
    let layer = if in_sight {
        Layer::Solid
    } else {
        Layer::Memory
    };
    let centre = Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE);
    let depth = match scene.shape {
        Shape::Built => FLOOR_THICKNESS,
        Shape::Natural => WALL_HEIGHT,
    };
    let lit = |position: Vec3| match layer {
        Layer::Memory => Vec3::splat(MEMORY_SHADE),
        _ => {
            let light = corner_light(scene.level, scene.light, position);
            with_ambient(light.unwrap_or(scene.light.at(x, y)))
        }
    };
    for (sx, sy) in QUARTERS {
        let place = |(east, south): Point| {
            let (e, s) = (sx as f32 * east, sy as f32 * south);
            (centre + Vec3::new(e, 0.0, s), flat_uv(e, s))
        };
        if let Some(remains) = remains(scene.level, (x, y), (sx, sy)) {
            let points: Vec<_> = remains.floor.iter().map(|&p| place(p)).collect();
            let surface = floor_surface(scene.level, (x, y), (sx, sy));
            let builder = layers.builder(layer, surface);
            let (first, rest) = points.split_first().expect("a floor has points");
            for pair in rest.windows(2) {
                let corners = [*first, pair[0], pair[1], pair[1]];
                builder.free(
                    corners.map(|(position, _)| position),
                    Vec3::Y,
                    corners.map(|(_, uv)| uv),
                    corners.map(|(position, _)| lit(position)),
                );
            }
            if layer == Layer::Solid {
                let corner = place((HALF_CELL, HALF_CELL)).0;
                let edge: Vec<Vec3> = remains.edge.iter().map(|&p| place(p).0).collect();
                let wall = layers.builder(layer, Surface::Wall);
                for pair in edge.windows(2) {
                    add_side(wall, pair, corner, depth, scene.shape, &lit);
                }
            }
        }
        if layer == Layer::Solid {
            // The black base under the whole quarter.
            let at = |east: f32, south: f32| place((east, south)).0 - Vec3::Y * depth;
            layers.builder(layer, Surface::Wall).free(
                [
                    at(0.0, 0.0),
                    at(HALF_CELL, 0.0),
                    at(HALF_CELL, HALF_CELL),
                    at(0.0, HALF_CELL),
                ],
                Vec3::Y,
                [[0.0, 0.0]; 4],
                [Vec3::ZERO; 4],
            );
        }
    }
}

/// One stretch of a hole's side, from the floor's edge straight down, facing
/// away from the grid's `corner` (into the hole).
fn add_side(
    wall: &mut super::mesh::Builder,
    pair: &[Vec3],
    corner: Vec3,
    depth: f32,
    shape: Shape,
    lit: &impl Fn(Vec3) -> Vec3,
) {
    let (a, b) = (pair[0], pair[1]);
    let along = b - a;
    let mut normal = Vec3::new(along.z, 0.0, -along.x).normalize_or_zero();
    if normal.dot((a + b) / 2.0 - corner) < 0.0 {
        normal = -normal;
    }
    let u = |p: Vec3| ((p.x + p.z) / CELL_SIZE).rem_euclid(1.0);
    let (ua, mut ub) = (u(a), u(b));
    if ub < ua {
        ub = (ua + along.length() / CELL_SIZE).min(1.0);
    }
    let v = (depth / CELL_SIZE).min(1.0);
    let bottom_shade = match shape {
        Shape::Built => SIDE_BOTTOM_SHADE,
        // Fading to black, as stairs' shafts do.
        Shape::Natural => 0.0,
    };
    let down = Vec3::Y * depth;
    wall.free(
        [a, b, b - down, a - down],
        normal,
        [[ua, 0.0], [ub, 0.0], [ub, v], [ua, v]],
        [
            lit(a) * SIDE_TOP_SHADE,
            lit(b) * SIDE_TOP_SHADE,
            lit(b) * bottom_shade,
            lit(a) * bottom_shade,
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A level of floor with holes where the picture has `O`, offset so that
    /// no cell in it is on the level's edge.
    fn level_of(rows: &[&str]) -> Level {
        let mut level = Level::new(-1, -1, 12, 12);
        for y in -1..11 {
            for x in -1..11 {
                level.set(x, y, Cell::Floor);
            }
        }
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if c == 'O' {
                    level.set(x as i64, y as i64, Cell::Hole);
                }
            }
        }
        level
    }

    /// Every point of a level's holes' edges that lies on the line between
    /// two cells, in level units, rounded to compare.
    fn crossings(level: &Level) -> Vec<(i64, i64)> {
        let mut found = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                if level.cell(x, y) != Cell::Hole {
                    continue;
                }
                for (sx, sy) in QUARTERS {
                    let Some(remains) = remains(level, (x, y), (sx, sy)) else {
                        continue;
                    };
                    for &(east, south) in &remains.edge {
                        if east == HALF_CELL || south == HALF_CELL {
                            let px = x as f32 * CELL_SIZE + sx as f32 * east;
                            let pz = y as f32 * CELL_SIZE + sy as f32 * south;
                            found
                                .push(((px * 1000.0).round() as i64, (pz * 1000.0).round() as i64));
                        }
                    }
                }
            }
        }
        found.sort();
        found
    }

    /// Where a hole's edge leaves a cell, the neighbouring hole cell's edge
    /// starts at exactly that point, in any arrangement of cells.
    #[test]
    fn touching_holes_line_up() {
        let level = level_of(&[
            "..........",
            ".OOO..O...",
            ".O.O.OO...",
            ".OOO..O...",
            "...O......",
            "..OOOOO...",
            "..O.O.....",
            "...O.O....",
        ]);
        let found = crossings(&level);
        assert!(!found.is_empty());
        // Each point comes in a pair: one from each cell sharing the edge.
        for pair in found.chunks(2) {
            assert_eq!(pair.len(), 2, "{found:?}");
            assert_eq!(pair[0], pair[1], "{found:?}");
        }
    }

    /// All-hole quarters have no floor; a lone hole keeps floor in every
    /// quarter, within the cell, round a dark middle.
    #[test]
    fn a_quarter_has_floor_only_beside_floor() {
        let lone = level_of(&["...", ".O.", "..."]);
        for quarter in QUARTERS {
            let remains = remains(&lone, (1, 1), quarter).unwrap();
            assert!(remains.floor.len() >= 4);
            for &(east, south) in &remains.floor {
                assert!((0.0..=HALF_CELL).contains(&east));
                assert!((0.0..=HALF_CELL).contains(&south));
                assert!(east > 0.0 || south > 0.0);
            }
        }
        let open = level_of(&["OOO", "OOO", "OOO"]);
        assert!(remains(&open, (1, 1), (1, 1)).is_none());
    }

    /// A hole on the level's edge, or in its corner, has no floor along the
    /// edge: only where it meets floor inside the level.
    #[test]
    fn nothing_is_left_along_the_levels_edge() {
        let mut level = Level::new(0, 0, 3, 3);
        for y in 0..3 {
            for x in 0..3 {
                level.set(x, y, Cell::Hole);
            }
        }
        for quarter in QUARTERS {
            assert!(remains(&level, (0, 0), quarter).is_none(), "{quarter:?}");
            assert!(remains(&level, (1, 0), quarter).is_none(), "{quarter:?}");
        }
        level.set(1, 1, Cell::Floor);
        // Towards the floor, a bite of it; off the edge, still nothing.
        assert!(remains(&level, (0, 0), (1, 1)).is_some());
        assert!(remains(&level, (0, 0), (-1, -1)).is_none());
    }

    /// In every arrangement of the three cells round a quarter, the floor's
    /// outline stays a simple shape seen whole from the grid's corner, with
    /// no area lost (the angle round the corner only ever goes one way).
    #[test]
    fn every_arrangement_has_a_good_outline() {
        for mask in 0..8 {
            let mut level = level_of(&["...", ".O.", "..."]);
            for (bit, (x, y)) in [(2, 1), (1, 2), (2, 2)].into_iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    level.set(x, y, Cell::Hole);
                }
            }
            let Some(remains) = remains(&level, (1, 1), (1, 1)) else {
                assert_eq!(mask, 7);
                continue;
            };
            let angle = |&(east, south): &Point| (HALF_CELL - south).atan2(HALF_CELL - east);
            let angles: Vec<f32> = remains.floor[1..].iter().map(angle).collect();
            let up = angles.windows(2).all(|p| p[1] >= p[0] - 1e-4);
            let down = angles.windows(2).all(|p| p[1] <= p[0] + 1e-4);
            assert!(up || down, "mask {mask}: {:?}", remains.floor);
        }
    }
}
