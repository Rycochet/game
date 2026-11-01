//! Stairs in 3D, one model for each `StairsStyle`. Stairs up stand on the
//! floor. Stairs down are the same model sunk into a hole (a shaft with a
//! solid black base) so that its top meets the floor, with handrails and the
//! tops of ladders coming up above it.
//!
//! Everything is unlit, so faces are shaded by hand from the way they face.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::protocol::SeenStairs;
use shared::site::{StairsDirection, StairsStyle};

use super::mesh::{Builder, Surface, View};
use super::{CELL_SIZE, WALL_HEIGHT};

/// How deep the hole for stairs down is, and so how high their model is.
const STAIRS_DOWN_DEPTH: f32 = WALL_HEIGHT * 0.75;
/// How high the model is going up: three times that, so stairs climb at a
/// believable rate, cut off at `STAIRS_UP_TOP`.
const STAIRS_UP_HEIGHT: f32 = STAIRS_DOWN_DEPTH * 3.0;
/// The same for turning stairs, whose flights are short and need less.
const TURNING_UP_HEIGHT: f32 = STAIRS_DOWN_DEPTH * 2.25;
/// Where models going up are cut off: a tile and a half.
const STAIRS_UP_TOP: f32 = CELL_SIZE * 1.5;
/// How much further apart rungs and knots are going up.
const UP_SPACING: f32 = 1.5;

/// How tall a viewer's token is taken to be, when working out whether stairs
/// stand in the way of seeing it.
const VIEWER_HEIGHT: f32 = CELL_SIZE;

/// Half the width of the holes and models: floor is left all round them.
const SQUARE_HALF: f32 = CELL_SIZE * 0.4;
const ROUND_RADIUS: f32 = CELL_SIZE * 0.4;
/// The gap between a spiral's steps and the side of its hole.
const SPIRAL_GAP: f32 = 0.3;

/// Sides of a round hole: a multiple of 8, so some point at the cell's corners.
const ROUND_SIDES: usize = 32;

/// How far above its steps a handrail runs, and how thick it is.
const RAIL_HEIGHT: f32 = 9.0;
const RAIL_RADIUS: f32 = 0.3;
/// How far outside the hole the rail round its top stands.
const RIM_RAIL_OUTSIDE: f32 = 0.3;
/// Pieces of the rail round the top of a round hole, and a post every so many.
const RIM_RAIL_PIECES: usize = 16;
const RIM_POST_EVERY: usize = 4;
const POST_RADIUS: f32 = 0.22;

const SPIRAL_STEPS: usize = 9;
const SPIRAL_TURNS: f32 = 1.5;
/// Pieces each step's outer edge and tread are cut into, to follow the circle.
const SPIRAL_STEP_PIECES: usize = 4;
const COLUMN_RADIUS: f32 = 1.5;
/// How far a step's sides reach below the tread of the step before.
const SPIRAL_STEP_OVERLAP: f32 = 0.4;

/// Steps in each of the two flights of turning stairs.
const FLIGHT_STEPS: usize = 2;
/// How much of the model's depth the landing takes.
const LANDING_DEPTH: f32 = SQUARE_HALF * 0.5;

const LADDER_HALF_WIDTH: f32 = 2.4;
const LADDER_POLE_RADIUS: f32 = 0.4;
const LADDER_RUNG_RADIUS: f32 = 0.28;
const RUNG_SPACING: f32 = 4.2;
/// How far the tops of ladders come up above the floor, going down.
const LADDER_ABOVE_FLOOR: f32 = 4.0;
/// How far a leaning ladder reaches from the middle, where it meets the floor
/// (it rests just inside the edge of the hole).
const LEAN_REACH: f32 = ROUND_RADIUS * 0.9;

const LOG_RADIUS: f32 = 1.1;
const LOG_RUNG_LENGTH: f32 = 2.6;

/// How thick ropes are, and how far a rope ladder's rungs sag.
const ROPE_RADIUS: f32 = 0.3;
const ROPE_SAG: f32 = 0.7;
const SLAT_RADIUS: f32 = 0.5;
const KNOT_RADIUS: f32 = 0.85;
const KNOT_SPACING: f32 = 4.5;
/// How far a knot reaches up and down from its middle.
const KNOT_HALF_HEIGHT: f32 = 0.3;
/// The curl of rope left on the floor: how far its middle is from the rope,
/// how tight it winds to, how many turns and pieces, and its height.
const CURL_RADIUS: f32 = 1.8;
const CURL_INNER_RADIUS: f32 = 0.5;
const CURL_TURNS: f32 = 0.875;
const CURL_PIECES: usize = 8;
const CURL_HEIGHT: f32 = 0.3;
/// A hanging rope's way over the edge of the hole: outwards from its side
/// and up from the floor, ending on the floor at a peg.
const ROPE_OVER_EDGE: [(f32, f32); 4] = [(0.0, 3.6), (0.4, 4.4), (1.2, 3.4), (1.6, 0.15)];
const ROPE_INSIDE: f32 = 0.7;
const PEG_HEIGHT: f32 = 2.2;
const PEG_RADIUS: f32 = 0.4;

/// Rope is the wood's picture made paler (vertex colours multiply it).
const ROPE_TINT: Vec3 = Vec3::new(2.8, 2.35, 1.7);
/// A spot on the wood's picture away from the dark lines, for rope.
const ROPE_UV: [f32; 2] = [0.5, 2.5 / 32.0];
/// One plank of the wood's picture, for bars, which run along its grain:
/// from `PLANK_U.0`, and across `PLANK_V`.
const PLANK_U: (f32, f32) = (12.1 / 32.0, 31.9 / 32.0);
const PLANK_V: (f32, f32) = (1.1 / 32.0, 3.9 / 32.0);

/// How much darker the outer sides of steps are than their treads, and how
/// bright their fronts are compared with their treads.
const RISER_SHADE: f32 = 0.6;
const FRONT_SHADE: f32 = 0.45;
/// Treads are the floor's picture made brighter, to stand out from the floor.
const TREAD_TINT: f32 = 1.35;
/// The circle of floor under a ladder going up, the size of the hole it comes
/// through, is the floor's picture made this much brighter, to show it is a way out.
const LADDER_LIGHT_TINT: f32 = 1.6;

/// Inside the hole: how bright the side is at the top and the bottom, and how
/// bright the model is at the bottom (it is full bright at the floor). Vertex
/// colours are linear, so these look much brighter on screen than they read:
/// 0.03 shows as about 20% of full.
const RIM_SHADE: f32 = 0.6;
const DEEPEST_SHADE: f32 = 0.02;
const MODEL_DEEPEST_SHADE: f32 = 0.03;

/// The shape of a hole, centred on its cell.
#[derive(Clone, Copy)]
enum Hole {
    Round,
    Square,
}

/// Draws a staircase on its cell, floor included. `lit` is the colour at the
/// stairs' cell and `floor_colour` the colour at a point on the floor (light
/// with the ambient, or memory's grey). If `through` is given, what stands
/// above the floor goes there instead of `meshes`, to be drawn see-through.
/// `floor` is the surface of the cell they stand on.
pub fn build_stairs(
    meshes: &mut [Builder],
    through: Option<&mut [Builder]>,
    stairs: &SeenStairs,
    floor: Surface,
    lit: Vec3,
    floor_colour: &impl Fn(Vec3) -> Vec3,
) {
    let centre = Vec3::new(
        stairs.x as f32 * CELL_SIZE,
        0.0,
        stairs.y as f32 * CELL_SIZE,
    );
    let (dx, dy) = stairs.descends;
    let descends = Vec3::new(dx as f32, 0.0, dy as f32);
    let down = stairs.direction == StairsDirection::Down;
    if down {
        let hole = match stairs.style {
            StairsStyle::Turning => Hole::Square,
            _ => Hole::Round,
        };
        floor_with_hole(meshes, floor, centre, hole, floor_colour);
        shaft(meshes, centre, hole, lit);
    } else if matches!(stairs.style, StairsStyle::Spiral | StairsStyle::Turning) {
        floor_whole(meshes, floor, centre, floor_colour);
    } else {
        floor_with_hole(meshes, floor, centre, Hole::Round, floor_colour);
        light_under_ladder(meshes, floor, centre, floor_colour);
    }
    let mut model = Model {
        meshes,
        through,
        lit,
        base: centre - Vec3::Y * if down { STAIRS_DOWN_DEPTH } else { 0.0 },
        height: match (down, stairs.style) {
            (true, _) => STAIRS_DOWN_DEPTH,
            (false, StairsStyle::Turning) => TURNING_UP_HEIGHT,
            (false, _) => STAIRS_UP_HEIGHT,
        },
        ceiling: if down { f32::INFINITY } else { STAIRS_UP_TOP },
        spacing: if down { 1.0 } else { UP_SPACING },
        descends,
        across: Vec3::Y.cross(descends),
    };
    match stairs.style {
        StairsStyle::Spiral => model.spiral(down),
        // Both ends open onto the floor beside them, the side you walk on
        // from, so going down that is the other way from going up.
        StairsStyle::Turning => model.turning(if down { -descends } else { descends }, down),
        StairsStyle::Ladder => model.ladder(down),
        StairsStyle::LogLadder => model.log_ladder(down),
        StairsStyle::RopeLadder => model.rope_ladder(down, false),
        StairsStyle::SlatLadder => model.rope_ladder(down, true),
        StairsStyle::KnottedRope => model.knotted_rope(down),
    }
}

/// Whether stairs stand between the camera and any viewer's token (or on the
/// same cell), so they are drawn see-through, like a wall in the way.
pub fn hides_viewer(stairs: &SeenStairs, views: &[View], camera: Vec3) -> bool {
    let top = match (stairs.direction, stairs.style) {
        (StairsDirection::Up, _) => STAIRS_UP_TOP,
        (_, StairsStyle::Spiral | StairsStyle::Turning) => RAIL_HEIGHT,
        _ => LADDER_ABOVE_FLOOR,
    };
    let half = CELL_SIZE / 2.0;
    let centre = Vec3::new(stairs.x as f32, 0.0, stairs.y as f32) * CELL_SIZE;
    let low = centre - Vec3::new(half, 0.0, half);
    let high = centre + Vec3::new(half, top, half);
    views.iter().any(|view| {
        let foot = Vec3::new(view.cell.0 as f32, 0.0, view.cell.1 as f32) * CELL_SIZE;
        let head = foot + Vec3::Y * VIEWER_HEIGHT;
        segment_hits_box(camera, foot, low, high) || segment_hits_box(camera, head, low, high)
    })
}

/// Whether the straight line from `from` to `to` passes through a box.
fn segment_hits_box(from: Vec3, to: Vec3, low: Vec3, high: Vec3) -> bool {
    let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
    let way = to - from;
    for axis in 0..3 {
        if way[axis].abs() < 1e-6 {
            if from[axis] < low[axis] || from[axis] > high[axis] {
                return false;
            }
            continue;
        }
        let (a, b) = (
            (low[axis] - from[axis]) / way[axis],
            (high[axis] - from[axis]) / way[axis],
        );
        enter = enter.max(a.min(b));
        leave = leave.min(a.max(b));
    }
    enter <= leave
}

/// A point on the cell's floor to picture coordinates.
fn floor_uv(point: Vec3, centre: Vec3) -> [f32; 2] {
    let at = (point - centre) / CELL_SIZE + Vec3::splat(0.5);
    [at.x.clamp(0.0, 1.0), at.z.clamp(0.0, 1.0)]
}

/// The floor of a whole cell.
fn floor_whole(
    meshes: &mut [Builder],
    floor: Surface,
    centre: Vec3,
    colour: &impl Fn(Vec3) -> Vec3,
) {
    let half = CELL_SIZE / 2.0;
    let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .map(|(x, z)| centre + Vec3::new(x * half, 0.0, z * half));
    meshes[floor as usize].free(
        corners,
        Vec3::Y,
        corners.map(|p| floor_uv(p, centre)),
        corners.map(colour),
    );
}

/// The corners of a hole, going round, as offsets from its centre.
fn hole_outline(hole: Hole) -> Vec<Vec3> {
    match hole {
        Hole::Round => (0..ROUND_SIDES)
            .map(|i| {
                let angle = i as f32 / ROUND_SIDES as f32 * TAU;
                Vec3::new(angle.cos(), 0.0, angle.sin()) * ROUND_RADIUS
            })
            .collect(),
        Hole::Square => [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
            .map(|(x, z)| Vec3::new(x, 0.0, z) * SQUARE_HALF)
            .to_vec(),
    }
}

/// The floor of a cell with a hole in it: pieces from each side of the hole
/// out to the edge of the cell.
fn floor_with_hole(
    meshes: &mut [Builder],
    floor: Surface,
    centre: Vec3,
    hole: Hole,
    colour: &impl Fn(Vec3) -> Vec3,
) {
    let half = CELL_SIZE / 2.0;
    // Straight out from the hole's corner to the edge of the cell.
    let to_edge = |p: Vec3| p / p.x.abs().max(p.z.abs()) * half;
    let outline = hole_outline(hole);
    for (i, &a) in outline.iter().enumerate() {
        let b = outline[(i + 1) % outline.len()];
        let corners = [a, b, to_edge(b), to_edge(a)].map(|p| centre + p);
        meshes[floor as usize].free(
            corners,
            Vec3::Y,
            corners.map(|p| floor_uv(p, centre)),
            corners.map(colour),
        );
    }
}

/// Fills the round hole `floor_with_hole` leaves with brighter floor.
fn light_under_ladder(
    meshes: &mut [Builder],
    floor: Surface,
    centre: Vec3,
    colour: &impl Fn(Vec3) -> Vec3,
) {
    let outline = hole_outline(Hole::Round);
    for (i, &a) in outline.iter().enumerate() {
        let b = outline[(i + 1) % outline.len()];
        // A triangle, as a quad with two corners the same.
        let corners = [centre, centre + a, centre + b, centre + b];
        meshes[floor as usize].free(
            corners,
            Vec3::Y,
            corners.map(|p| floor_uv(p, centre)),
            corners.map(|p| colour(p) * LADDER_LIGHT_TINT),
        );
    }
}

/// The side of a hole, in the wall's picture and darker the deeper (only its
/// inside: the near part faces away from the camera, so doesn't hide the
/// model), and a solid black base, so nothing behind can show through.
fn shaft(meshes: &mut [Builder], centre: Vec3, hole: Hole, lit: Vec3) {
    let depth = Vec3::Y * STAIRS_DOWN_DEPTH;
    let outline = hole_outline(hole);
    let mut along = 0.0;
    for (i, &a) in outline.iter().enumerate() {
        let b = outline[(i + 1) % outline.len()];
        let width = a.distance(b) / CELL_SIZE;
        let (a, b) = (centre + a, centre + b);
        let middle = (a + b) / 2.0;
        let normal = (centre - middle).normalize();
        let (u0, u1) = (along, (along + width).min(1.0));
        let v = STAIRS_DOWN_DEPTH / CELL_SIZE;
        let top = lit * RIM_SHADE;
        let bottom = lit * DEEPEST_SHADE;
        meshes[Surface::Wall as usize].free(
            [a, b, b - depth, a - depth],
            normal,
            [[u0, 0.0], [u1, 0.0], [u1, v], [u0, v]],
            [top, top, bottom, bottom],
        );
        along = (along + width).fract();
        // A slice of the base, from the middle.
        let base = centre - depth;
        let corners = [base, a - depth, b - depth, base];
        meshes[Surface::Floor as usize].free(
            corners,
            Vec3::Y,
            corners.map(|p| floor_uv(p, centre)),
            [Vec3::ZERO; 4],
        );
    }
}

/// How bright a face is, from the way it faces: tops brightest, then the
/// south and east, undersides and the north and west darker.
fn facing_shade(normal: Vec3) -> f32 {
    (0.62 + 0.38 * normal.y + 0.14 * normal.z + 0.07 * normal.x).clamp(0.3, 1.0)
}

/// The tint that makes the front of a step, facing `normal`, a fixed amount
/// darker than its tread, whichever way it faces.
fn front_tint(normal: Vec3) -> Vec3 {
    Vec3::splat(TREAD_TINT * FRONT_SHADE / facing_shade(normal))
}

/// A model being built on its cell: `base` is the middle of its bottom,
/// `descends` and `across` the two ways along the floor.
struct Model<'a> {
    meshes: &'a mut [Builder],
    /// Where what stands above the floor goes, when see-through.
    through: Option<&'a mut [Builder]>,
    lit: Vec3,
    base: Vec3,
    /// How high it would be, and the height it is cut off at.
    height: f32,
    ceiling: f32,
    /// How far apart rungs and knots are, compared with going down.
    spacing: f32,
    descends: Vec3,
    across: Vec3,
}

impl Model<'_> {
    /// The colour at a point: darker the deeper below the floor.
    fn colour_at(&self, point: Vec3) -> Vec3 {
        let depth = (-point.y / STAIRS_DOWN_DEPTH).clamp(0.0, 1.0);
        self.lit * (1.0 + (MODEL_DEEPEST_SHADE - 1.0) * depth)
    }

    /// A point in the model, `(along descends, height, along across)`.
    fn at(&self, (along, up, side): (f32, f32, f32)) -> Vec3 {
        self.base + self.descends * along + Vec3::Y * up + self.across * side
    }

    /// A face with four corners going round it, shaded by the way it faces.
    /// Its picture keeps its scale; flat ones line up with the cell's.
    fn face(&mut self, surface: Surface, corners: [Vec3; 4], normal: Vec3, tint: Vec3) {
        let uvs = if normal.y.abs() > 0.9 {
            let cell = self.base * Vec3::new(1.0, 0.0, 1.0);
            corners.map(|p| floor_uv(p * Vec3::new(1.0, 0.0, 1.0), cell))
        } else {
            let u = (corners[1].distance(corners[0]) / CELL_SIZE).min(1.0);
            let v = (corners[2].distance(corners[1]) / CELL_SIZE).min(1.0);
            [[0.0, 0.0], [u, 0.0], [u, v], [0.0, v]]
        };
        self.face_with(surface, corners, normal, tint, uvs);
    }

    /// A face as `face`, with its picture coordinates given.
    fn face_with(
        &mut self,
        surface: Surface,
        corners: [Vec3; 4],
        normal: Vec3,
        tint: Vec3,
        uvs: [[f32; 2]; 4],
    ) {
        let shade = facing_shade(normal);
        let colours = corners.map(|p| self.colour_at(p) * shade * tint);
        let target = match self.through.as_deref_mut() {
            Some(through) if corners.iter().any(|p| p.y > 0.01) => through,
            _ => &mut *self.meshes,
        };
        target[surface as usize].free(corners, normal, uvs, colours);
    }

    /// A bar from `from` to `to` with `sides` flat sides and a cap at its
    /// upper end, cut off at the ceiling.
    fn bar(
        &mut self,
        surface: Surface,
        (from, to): (Vec3, Vec3),
        radius: f32,
        sides: usize,
        tint: Vec3,
    ) {
        if from.y.min(to.y) >= self.ceiling {
            return;
        }
        let cut = |p: Vec3, other: Vec3| {
            if p.y > self.ceiling {
                other + (p - other) * (self.ceiling - other.y) / (p.y - other.y)
            } else {
                p
            }
        };
        let (from, to) = (cut(from, to), cut(to, from));
        self.whole_bar(surface, (from, to), radius, sides, tint);
    }

    /// A bar as `bar`, not cut off (handrails).
    fn whole_bar(
        &mut self,
        surface: Surface,
        (from, to): (Vec3, Vec3),
        radius: f32,
        sides: usize,
        tint: Vec3,
    ) {
        let way = (to - from).normalize_or_zero();
        if way == Vec3::ZERO {
            return;
        }
        let helper = if way.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
        let right = way.cross(helper).normalize();
        let up = right.cross(way);
        let spoke = |i: usize| {
            let angle = i as f32 / sides as f32 * TAU;
            right * angle.cos() + up * angle.sin()
        };
        for i in 0..sides {
            let (a, b) = (spoke(i), spoke(i + 1));
            let normal = (a + b).normalize();
            let corners = [
                from + a * radius,
                from + b * radius,
                to + b * radius,
                to + a * radius,
            ];
            // Rope is one colour; wood runs along a plank of its picture.
            let uvs = if tint == ROPE_TINT {
                [ROPE_UV; 4]
            } else if surface == Surface::Door {
                let (u0, v0, v1) = (PLANK_U.0, PLANK_V.0, PLANK_V.1);
                let u1 = (u0 + from.distance(to) / CELL_SIZE).min(PLANK_U.1);
                [[u0, v0], [u0, v1], [u1, v1], [u1, v0]]
            } else {
                let across = a.distance(b) * radius / CELL_SIZE;
                let up = (from.distance(to) / CELL_SIZE).min(1.0);
                [[0.0, 0.0], [across, 0.0], [across, up], [0.0, up]]
            };
            self.face_with(surface, corners, normal, tint, uvs);
        }
        let (top, cap) = if to.y >= from.y {
            (to, way)
        } else {
            (from, -way)
        };
        for i in 0..sides {
            let (a, b) = (top + spoke(i) * radius, top + spoke(i + 1) * radius);
            let uv = if tint == ROPE_TINT {
                ROPE_UV
            } else {
                [PLANK_U.0, PLANK_V.0]
            };
            self.face_with(surface, [top, a, b, top], cap, tint, [uv; 4]);
        }
    }

    /// A step of solid stairs: its tread and front (the side facing `front`)
    /// in the floor's picture, its other sides, darker, in the wall's.
    fn steps_block(&mut self, top: Vec3, half: (f32, f32), bottom: f32, front: Vec3) {
        let top = Vec3::new(top.x, top.y.min(self.ceiling), top.z);
        let (d, a) = (self.descends * half.0, self.across * half.1);
        let corners = [top - d - a, top + d - a, top + d + a, top - d + a];
        self.face(Surface::Floor, corners, Vec3::Y, Vec3::splat(TREAD_TINT));
        for (i, normal) in [-self.across, self.descends, self.across, -self.descends]
            .into_iter()
            .enumerate()
        {
            let (p, q) = (corners[i], corners[(i + 1) % 4]);
            let down = |p: Vec3| Vec3::new(p.x, bottom, p.z);
            let side = [p, q, down(q), down(p)];
            if normal == front {
                self.face(Surface::Floor, side, normal, front_tint(normal));
            } else {
                self.face(Surface::Wall, side, normal, Vec3::splat(RISER_SHADE));
            }
        }
        let under = corners.map(|p| Vec3::new(p.x, bottom, p.z));
        self.face(Surface::Wall, under, Vec3::NEG_Y, Vec3::splat(RISER_SHADE));
    }

    /// A rail along `points`, with a second bar half way down to what it
    /// stands on.
    fn rail(&mut self, points: &[Vec3]) {
        let middle = Vec3::Y * RAIL_HEIGHT / 2.0;
        for pair in points.windows(2) {
            let bar = (pair[0], pair[1]);
            self.whole_bar(Surface::Door, bar, RAIL_RADIUS, 4, Vec3::ONE);
            let bar = (pair[0] - middle, pair[1] - middle);
            self.whole_bar(Surface::Door, bar, RAIL_RADIUS, 4, Vec3::ONE);
        }
    }

    /// A post from a step up to the rail above it.
    fn post(&mut self, foot: Vec3) {
        let top = foot + Vec3::Y * RAIL_HEIGHT;
        self.whole_bar(Surface::Door, (foot, top), POST_RADIUS, 4, Vec3::ONE);
    }

    /// Winds up round a column, from near the side `descends` points at to
    /// the opposite side, where the floor is at the top. Going down, the top
    /// step is a landing of twice the angle, centred on that side, and the
    /// only rail runs round the top of the hole beside the landing, then across
    /// its far edge to the column. Going up, it has no rails.
    fn spiral(&mut self, down: bool) {
        let radius = ROUND_RADIUS - SPIRAL_GAP;
        let rise = self.height / SPIRAL_STEPS as f32;
        let turned = SPIRAL_TURNS * TAU;
        let (step_angle, first) = if down {
            let angle = turned / (SPIRAL_STEPS + 1) as f32;
            (angle, turned + angle - (SPIRAL_STEPS + 1) as f32 * angle)
        } else {
            (turned / SPIRAL_STEPS as f32, 0.0)
        };
        let (base, descends, across) = (self.base, self.descends, self.across);
        let way = move |angle: f32| descends * angle.cos() + across * angle.sin();
        let point = move |angle: f32, r: f32, up: f32| base + way(angle) * r + Vec3::Y * up;
        let wall_side = RISER_SHADE * Vec3::ONE;
        for step in 0..SPIRAL_STEPS {
            let a = first + step as f32 * step_angle;
            let landing = down && step + 1 == SPIRAL_STEPS;
            let b = a + step_angle * if landing { 2.0 } else { 1.0 };
            let top = (step + 1) as f32 * rise;
            if top > self.ceiling + 0.01 {
                break;
            }
            let low = (top - rise - SPIRAL_STEP_OVERLAP).max(0.0);
            let pieces = SPIRAL_STEP_PIECES * if landing { 2 } else { 1 };
            for piece in 0..pieces {
                let (from, to) = (
                    a + (b - a) * piece as f32 / pieces as f32,
                    a + (b - a) * (piece + 1) as f32 / pieces as f32,
                );
                let tread = [
                    point(from, COLUMN_RADIUS, top),
                    point(from, radius, top),
                    point(to, radius, top),
                    point(to, COLUMN_RADIUS, top),
                ];
                let edge = [
                    point(from, radius, top),
                    point(to, radius, top),
                    point(to, radius, low),
                    point(from, radius, low),
                ];
                let outwards = way((from + to) / 2.0);
                self.face(Surface::Floor, tread, Vec3::Y, Vec3::splat(TREAD_TINT));
                self.face(Surface::Wall, edge, outwards, wall_side);
                let under = tread.map(|p| Vec3::new(p.x, base.y + low, p.z));
                self.face(Surface::Wall, under, Vec3::NEG_Y, wall_side);
            }
            // The front of the step, facing back the way it winds.
            let front = [
                point(a, COLUMN_RADIUS, top),
                point(a, radius, top),
                point(a, radius, low),
                point(a, COLUMN_RADIUS, low),
            ];
            let backwards = way(a - PI / 2.0);
            self.face(Surface::Floor, front, backwards, front_tint(backwards));
            // The back of the step, facing on the way it winds.
            let back = [
                point(b, COLUMN_RADIUS, top),
                point(b, radius, top),
                point(b, radius, low),
                point(b, COLUMN_RADIUS, low),
            ];
            self.face(Surface::Wall, back, way(b + PI / 2.0), wall_side);
        }

        let high = self.height + RAIL_HEIGHT;
        if down {
            let (start, end) = (turned + step_angle, turned - step_angle + TAU);
            let rim_radius = ROUND_RADIUS + RIM_RAIL_OUTSIDE;
            let rim = |piece: usize| {
                let along = piece as f32 / RIM_RAIL_PIECES as f32;
                point(start + (end - start) * along, rim_radius, high)
            };
            let around: Vec<Vec3> = (0..=RIM_RAIL_PIECES).map(rim).collect();
            self.rail(&around);
            for piece in (0..=RIM_RAIL_PIECES).step_by(RIM_POST_EVERY) {
                self.post(rim(piece) - Vec3::Y * RAIL_HEIGHT);
            }
            self.rail(&[rim(0), point(start, COLUMN_RADIUS, high)]);
        }
        let column = (self.at((0.0, 0.0, 0.0)), self.at((0.0, high, 0.0)));
        self.bar(Surface::Wall, column, COLUMN_RADIUS, 12, wall_side);
    }

    /// Two flights side by side, solid down to the ground, turning back at a
    /// landing: both ends are on the side `open` points at (`descends` or
    /// the opposite). Going up, it has no rails; going down, its only rail
    /// runs round the top of the hole, leaving the way onto the top step open.
    fn turning(&mut self, open: Vec3, down: bool) {
        // Steps are placed with the model's axes, so flip them to match the
        // way from the open side to the landing.
        let flip = if open == self.descends { -1.0 } else { 1.0 };
        let half = SQUARE_HALF;
        let flight_half = half / 2.0;
        let run = (2.0 * half - LANDING_DEPTH) / FLIGHT_STEPS as f32;
        let levels = 2 * FLIGHT_STEPS + 1;
        let rise = self.height / levels as f32;
        let bottom = self.base.y;
        // `along` is 0 at the open side, 2 × half at the back.
        let spot = |along: f32, up: f32, side: f32| (flip * (along - half), up, side);

        for step in 0..FLIGHT_STEPS {
            let along = (step as f32 + 0.5) * run;
            // First flight on one side, climbing away from the open side.
            let up = (step + 1) as f32 * rise;
            let top = self.at(spot(along, up, flight_half));
            self.steps_block(top, (run / 2.0, flight_half), bottom, open);
            // Second flight on the other, climbing back.
            let up = (levels - step) as f32 * rise;
            let top = self.at(spot(along, up, -flight_half));
            self.steps_block(top, (run / 2.0, flight_half), bottom, -open);
        }
        let landing = (FLIGHT_STEPS + 1) as f32 * rise;
        let top = self.at(spot(2.0 * half - LANDING_DEPTH / 2.0, landing, 0.0));
        self.steps_block(top, (LANDING_DEPTH / 2.0, half), bottom, open);

        let high = RAIL_HEIGHT;
        let top_rail = self.height + high;
        if down {
            let rim = half + RIM_RAIL_OUTSIDE;
            let (near, far) = (-RIM_RAIL_OUTSIDE, 2.0 * half + RIM_RAIL_OUTSIDE);
            let around = [
                spot(near, top_rail, 0.0),
                spot(near, top_rail, rim),
                spot(far, top_rail, rim),
                spot(far, top_rail, -rim),
                spot(near, top_rail, -rim),
            ]
            .map(|p| self.at(p));
            self.rail(&around);
            for point in around {
                self.post(point - Vec3::Y * high);
            }
        }
    }

    /// The height of a leaning ladder's top: it comes up above the floor when
    /// going down.
    fn lean_top(&self, down: bool) -> f32 {
        self.height + if down { LADDER_ABOVE_FLOOR } else { 0.0 }
    }

    /// A point on the line of a leaning ladder, from the middle of the bottom
    /// to the edge of the hole, on the side opposite `descends` (where the
    /// floor is at the top), at a height.
    fn lean(&self, height: f32) -> f32 {
        -LEAN_REACH * height / self.height
    }

    /// A ladder leaning from the middle of the bottom to the edge of the hole.
    fn ladder(&mut self, down: bool) {
        let top = self.lean_top(down);
        for side in [-LADDER_HALF_WIDTH, LADDER_HALF_WIDTH] {
            let pole = (
                self.at((0.0, 0.0, side)),
                self.at((self.lean(top), top, side)),
            );
            self.bar(Surface::Door, pole, LADDER_POLE_RADIUS, 4, Vec3::ONE);
        }
        let mut up = self.spacing * RUNG_SPACING;
        while up < top - self.spacing * RUNG_SPACING / 2.0 {
            let along = self.lean(up);
            let rung = (
                self.at((along, up, -LADDER_HALF_WIDTH)),
                self.at((along, up, LADDER_HALF_WIDTH)),
            );
            self.bar(Surface::Door, rung, LADDER_RUNG_RADIUS, 4, Vec3::ONE);
            up += self.spacing * RUNG_SPACING;
        }
    }

    /// A thick pole leaning like a ladder, with rungs out of alternate sides.
    fn log_ladder(&mut self, down: bool) {
        let top = self.lean_top(down);
        let pole = (
            self.at((0.0, 0.0, 0.0)),
            self.at((self.lean(top), top, 0.0)),
        );
        self.bar(Surface::Door, pole, LOG_RADIUS, 8, Vec3::ONE);
        let mut up = self.spacing * RUNG_SPACING / 2.0;
        let mut side = 1.0;
        while up < top - self.spacing * RUNG_SPACING / 2.0 {
            let along = self.lean(up);
            let rung = (
                self.at((along, up, 0.0)),
                self.at((along, up, side * (LOG_RADIUS + LOG_RUNG_LENGTH))),
            );
            self.bar(Surface::Door, rung, LADDER_RUNG_RADIUS, 4, Vec3::ONE);
            up += self.spacing * RUNG_SPACING / 2.0;
            side = -side;
        }
    }

    /// A rope hanging straight at the side of the hole, `side` across: going
    /// down it comes up over the edge and is tied to a peg in the floor.
    fn hanging_rope(&mut self, side: f32, down: bool) {
        let along = -(ROUND_RADIUS - ROPE_INSIDE);
        let top = self.height;
        let mut points = vec![self.at((along, CURL_HEIGHT, side))];
        if down {
            for (out, up) in ROPE_OVER_EDGE {
                points.push(self.at((-ROUND_RADIUS - out, top + up, side)));
            }
            let [.., (out, _)] = ROPE_OVER_EDGE;
            let foot = self.at((-ROUND_RADIUS - out, top, side));
            let peg = (foot, foot + Vec3::Y * PEG_HEIGHT);
            self.bar(Surface::Door, peg, PEG_RADIUS, 6, Vec3::ONE);
        } else {
            points.push(self.at((along, top, side)));
        }
        for pair in points.windows(2) {
            let bar = (pair[0], pair[1]);
            self.bar(Surface::Door, bar, ROPE_RADIUS, 4, ROPE_TINT);
        }
    }

    /// Two ropes hanging straight, joined by sagging rope or wooden slats.
    fn rope_ladder(&mut self, down: bool, slats: bool) {
        let along = -(ROUND_RADIUS - ROPE_INSIDE);
        let width = LADDER_HALF_WIDTH;
        self.hanging_rope(-width, down);
        self.hanging_rope(width, down);
        let top = self.height + if down { ROPE_OVER_EDGE[0].1 } else { 0.0 };
        let mut up = self.spacing * RUNG_SPACING;
        while up < top - self.spacing * RUNG_SPACING / 3.0 {
            if slats {
                let slat = (
                    self.at((along, up, -width - 0.3)),
                    self.at((along, up, width + 0.3)),
                );
                self.bar(Surface::Door, slat, SLAT_RADIUS, 4, Vec3::ONE);
            } else {
                let middle = self.at((along, up - ROPE_SAG, 0.0));
                let sag = [
                    self.at((along, up, -width)),
                    self.at((along, up - ROPE_SAG * 0.75, -width / 2.0)),
                    middle,
                    self.at((along, up - ROPE_SAG * 0.75, width / 2.0)),
                    self.at((along, up, width)),
                ];
                for pair in sag.windows(2) {
                    let bar = (pair[0], pair[1]);
                    self.bar(Surface::Door, bar, ROPE_RADIUS * 0.8, 4, ROPE_TINT);
                }
            }
            up += self.spacing * RUNG_SPACING;
        }
    }

    /// One rope hanging straight, with knots and a curl on the floor.
    fn knotted_rope(&mut self, down: bool) {
        let along = -(ROUND_RADIUS - ROPE_INSIDE);
        self.hanging_rope(0.0, down);
        let top = self.height;
        let mut up = self.spacing * KNOT_SPACING;
        while up < top + if down { 1.0 } else { 0.0 } {
            let knot = (
                self.at((along, up - KNOT_HALF_HEIGHT, 0.0)),
                self.at((along, up + KNOT_HALF_HEIGHT, 0.0)),
            );
            self.bar(Surface::Door, knot, KNOT_RADIUS, 6, ROPE_TINT);
            up += self.spacing * KNOT_SPACING;
        }
        // A curl on the floor, winding in from the foot of the rope.
        let centre = along + CURL_RADIUS;
        let curl: Vec<Vec3> = (0..=CURL_PIECES)
            .map(|piece| {
                let done = piece as f32 / CURL_PIECES as f32;
                let r = CURL_RADIUS + (CURL_INNER_RADIUS - CURL_RADIUS) * done;
                let angle = PI + done * CURL_TURNS * TAU;
                self.at((centre + angle.cos() * r, CURL_HEIGHT, angle.sin() * r))
            })
            .collect();
        for pair in curl.windows(2) {
            let bar = (pair[0], pair[1]);
            self.bar(Surface::Door, bar, ROPE_RADIUS, 4, ROPE_TINT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(direction: StairsDirection, style: StairsStyle) -> Vec<Builder> {
        let mut meshes: Vec<Builder> = Surface::ALL.iter().map(|_| Builder::default()).collect();
        let stairs = SeenStairs {
            x: 0,
            y: 0,
            direction,
            descends: (0, 1),
            style,
        };
        build_stairs(
            &mut meshes,
            None,
            &stairs,
            Surface::Floor,
            Vec3::ONE,
            &|_| Vec3::ONE,
        );
        meshes
    }

    fn all_points(meshes: &[Builder]) -> impl Iterator<Item = Vec3> + '_ {
        meshes
            .iter()
            .flat_map(|mesh| mesh.positions.iter().map(|&p| Vec3::from_array(p)))
    }

    /// Ladders going up have a brighter circle of floor as wide as the hole
    /// above, and the floor still covers the whole cell; stairs have no circle.
    #[test]
    fn only_ladders_going_up_have_light_under_them() {
        let half = CELL_SIZE / 2.0;
        for style in StairsStyle::ALL {
            let mut meshes: Vec<Builder> =
                Surface::ALL.iter().map(|_| Builder::default()).collect();
            let stairs = SeenStairs {
                x: 0,
                y: 0,
                direction: StairsDirection::Up,
                descends: (0, 1),
                style,
            };
            // Only the floor comes straight from `floor_colour`.
            build_stairs(
                &mut meshes,
                None,
                &stairs,
                Surface::Floor,
                Vec3::ONE,
                &|_| Vec3::ONE,
            );
            let floor = &meshes[Surface::Floor as usize];
            let flat = || {
                floor
                    .positions
                    .iter()
                    .zip(&floor.colours)
                    .filter(|(p, _)| p[1] == 0.0)
            };
            let lit: Vec<_> = flat().filter(|(_, c)| c[0] == LADDER_LIGHT_TINT).collect();
            if matches!(style, StairsStyle::Spiral | StairsStyle::Turning) {
                assert!(lit.is_empty(), "{style:?}");
                continue;
            }
            let reach = lit
                .iter()
                .map(|(p, _)| Vec3::from_array(**p).length())
                .fold(0.0, f32::max);
            assert!((reach - ROUND_RADIUS).abs() < 0.001, "{style:?}: {reach}");
            let furthest = flat()
                .map(|(p, _)| p[0].abs().max(p[2].abs()))
                .fold(0.0, f32::max);
            assert!((furthest - half).abs() < 0.001, "{style:?}: {furthest}");
        }
    }

    /// Every model leaves a border of floor round it inside its cell, apart
    /// from the floor itself; a rope's peg may stand on that border.
    #[test]
    fn models_stay_inside_their_cell() {
        for style in StairsStyle::ALL {
            for direction in [StairsDirection::Up, StairsDirection::Down] {
                let meshes = build(direction, style);
                for p in all_points(&meshes[Surface::Door as usize..=Surface::Door as usize]) {
                    assert!(p.x.abs() < CELL_SIZE / 2.0, "{style:?}: {p}");
                    assert!(p.z.abs() < CELL_SIZE / 2.0, "{style:?}: {p}");
                }
                for p in all_points(&meshes[Surface::Wall as usize..=Surface::Wall as usize]) {
                    assert!(p.x.abs() < CELL_SIZE * 0.45, "{style:?}: {p}");
                    assert!(p.z.abs() < CELL_SIZE * 0.45, "{style:?}: {p}");
                }
            }
        }
    }

    /// Going down, the model is below the floor, with only rails and the tops
    /// of ladders above it, and the hole has a black base.
    #[test]
    fn stairs_down_go_below_the_floor_with_a_black_base() {
        for style in StairsStyle::ALL {
            let meshes = build(StairsDirection::Down, style);
            let lowest = all_points(&meshes).map(|p| p.y).fold(f32::MAX, f32::min);
            // The corner of a leaning pole's foot dips a little under the base.
            assert!((lowest + STAIRS_DOWN_DEPTH).abs() < 0.6, "{style:?}");
            let highest = all_points(&meshes).map(|p| p.y).fold(f32::MIN, f32::max);
            assert!(highest > 0.5, "{style:?} doesn't come above the floor");

            let floor = &meshes[Surface::Floor as usize];
            let base = floor
                .positions
                .iter()
                .zip(&floor.colours)
                // The middle of the base, which step fronts don't reach.
                .filter(|(p, _)| p[1] < 0.01 - STAIRS_DOWN_DEPTH && p[0] == 0.0 && p[2] == 0.0)
                .map(|(_, colour)| colour);
            let mut found = false;
            for colour in base {
                found = true;
                assert_eq!(colour[..3], [0.0, 0.0, 0.0], "{style:?}");
            }
            assert!(found, "{style:?} has no base");
        }
    }

    /// Stairs are see-through for a viewer on them or behind them from the
    /// camera, not for one in front of them or off to the side.
    #[test]
    fn stairs_hide_viewers_on_them_or_behind_them() {
        use shared::dungeon::{Level, Sight};
        let level = Level::test_room();
        let stairs = SeenStairs {
            x: 0,
            y: 0,
            direction: StairsDirection::Up,
            descends: (0, 1),
            style: StairsStyle::Spiral,
        };
        // South of the stairs, looking north.
        let camera = Vec3::new(0.0, 200.0, 200.0);
        let at = |x, y| {
            hides_viewer(
                &stairs,
                &[View {
                    cell: (x, y),
                    sight: Sight::new(&level),
                }],
                camera,
            )
        };
        assert!(at(0, 0));
        assert!(at(0, -1));
        assert!(!at(0, 1));
        assert!(!at(3, -1));
    }

    /// See-through stairs keep their floor solid.
    #[test]
    fn only_what_stands_above_the_floor_goes_see_through() {
        let mut meshes: Vec<Builder> = Surface::ALL.iter().map(|_| Builder::default()).collect();
        let mut through: Vec<Builder> = Surface::ALL.iter().map(|_| Builder::default()).collect();
        let stairs = SeenStairs {
            x: 0,
            y: 0,
            direction: StairsDirection::Down,
            descends: (0, 1),
            style: StairsStyle::Spiral,
        };
        let colour = |_| Vec3::ONE;
        build_stairs(
            &mut meshes,
            Some(&mut through),
            &stairs,
            Surface::Floor,
            Vec3::ONE,
            &colour,
        );
        assert!(all_points(&meshes).all(|p| p.y <= 0.01));
        assert!(all_points(&through).any(|p| p.y > 0.01));
    }

    /// Going up, models are cut off at a tile and a half, and reach it, and
    /// stairs have no handrails.
    #[test]
    fn stairs_up_stop_at_their_top_without_rails() {
        for style in StairsStyle::ALL {
            let meshes = build(StairsDirection::Up, style);
            let highest = all_points(&meshes).map(|p| p.y).fold(f32::MIN, f32::max);
            assert!(
                (highest - STAIRS_UP_TOP).abs() < 0.5,
                "{style:?}: {highest}"
            );
            if matches!(style, StairsStyle::Spiral | StairsStyle::Turning) {
                assert!(
                    meshes[Surface::Door as usize].indices.is_empty(),
                    "{style:?}"
                );
            }
        }
    }

    /// Every step of a spiral has a back and a bottom.
    #[test]
    fn spiral_steps_are_closed() {
        let meshes = build(StairsDirection::Up, StairsStyle::Spiral);
        let wall = &meshes[Surface::Wall as usize];
        // Each step's edge, bottom and back; the column adds more.
        let quads = wall.indices.len() / 6;
        assert!(quads >= 6 * (2 * SPIRAL_STEP_PIECES + 1), "{quads}");
    }

    /// Going up, nothing is below the floor.
    #[test]
    fn stairs_up_stand_on_the_floor() {
        for style in StairsStyle::ALL {
            let meshes = build(StairsDirection::Up, style);
            let lowest = all_points(&meshes).map(|p| p.y).fold(f32::MAX, f32::min);
            assert!(lowest >= -0.5, "{style:?}: {lowest}");
        }
    }
}
