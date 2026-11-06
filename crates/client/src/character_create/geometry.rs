//! Where the three triangles and their charts are drawn, in the chooser's own
//! units: a square from 0 to 1, y down, turning round its middle. Only for
//! showing and pointing: the build itself is integers (`shared::sheet::Build`).

use bevy::math::Vec2;
use shared::sheet::{Build, Dot, Row, TURN_STEPS, WEIGHT_TOTAL};

/// The middle the triangles turn round.
pub(super) const MIDDLE: Vec2 = Vec2::new(0.5, 0.5);
/// How far each triangle's middle is from it.
const APART: f32 = 0.24;
/// How far each triangle's corners are from its own middle.
pub(super) const REACH: f32 = 0.18;
/// The value a chart's axis reaches its corner at.
pub(super) const SHOWN_MOST: f32 = 250.0;

/// The direction this many degrees clockwise from straight up.
fn towards(degrees: f32) -> Vec2 {
    let radians = degrees.to_radians();
    Vec2::new(radians.sin(), -radians.cos())
}

/// How far a row is turned from the top, in degrees clockwise, as the shared
/// rules count it (Body at the top at 0, then a third of a turn each).
fn row_angle(row: Row, turn: u16) -> f32 {
    turn as f32 + row as usize as f32 * TURN_STEPS as f32 / 3.0
}

/// A row's triangle's middle.
pub(super) fn centre(row: Row, turn: u16) -> Vec2 {
    MIDDLE + towards(row_angle(row, turn)) * APART
}

/// A row's triangle's corners, in its column order: power at the top, finesse at
/// the lower right, resistance at the lower left. They stay that way up however
/// the triangles are turned, so each attribute keeps its place.
pub(super) fn corners(row: Row, turn: u16) -> [Vec2; 3] {
    let middle = centre(row, turn);
    [0.0, 120.0, 240.0].map(|angle| middle + towards(angle) * REACH)
}

/// The angles of the gaps between the triangles, in degrees clockwise from up:
/// a sixth of a turn from each.
pub(super) fn gaps(turn: u16) -> [f32; 3] {
    Row::ALL.map(|row| row_angle(row, turn) + TURN_STEPS as f32 / 6.0)
}

/// The point this far from the middle at this angle.
pub(super) fn round_middle(degrees: f32, radius: f32) -> Vec2 {
    MIDDLE + towards(degrees) * radius
}

/// Where a dot sits in its triangle.
pub(super) fn dot_place(row: Row, build: &Build) -> Vec2 {
    let [a, b, c] = corners(row, build.turn);
    let [wa, wb, wc] = build
        .dot(row)
        .0
        .map(|weight| weight as f32 / WEIGHT_TOTAL as f32);
    a * wa + b * wb + c * wc
}

/// The weights of a point in a triangle, as near as it can be while inside it.
pub(super) fn dot_at(point: Vec2, [a, b, c]: [Vec2; 3]) -> Dot {
    let area = (b - a).perp_dot(c - a);
    let raw = [
        (b - point).perp_dot(c - point) / area,
        (c - point).perp_dot(a - point) / area,
        (a - point).perp_dot(b - point) / area,
    ]
    .map(|weight| weight.max(0.0));
    let total: f32 = raw.iter().sum::<f32>().max(f32::EPSILON);
    let first = (raw[0] / total * WEIGHT_TOTAL as f32).round() as u16;
    let second = ((raw[1] / total * WEIGHT_TOTAL as f32).round() as u16).min(WEIGHT_TOTAL - first);
    Dot([first, second, WEIGHT_TOTAL - first - second])
}

/// Whether a point is in a triangle, or within `margin` of it.
pub(super) fn near_triangle(point: Vec2, corners: [Vec2; 3], margin: f32) -> bool {
    distance_to_shape(point, &corners) <= margin
}

/// The angle of a point round the middle, in degrees clockwise from up.
pub(super) fn angle_round_middle(point: Vec2) -> f32 {
    let from = point - MIDDLE;
    from.x.atan2(-from.y).to_degrees()
}

/// A turn moved on by this many degrees, kept within one turn.
pub(super) fn turned_by(turn: u16, degrees: f32) -> u16 {
    let steps = TURN_STEPS as i32;
    (turn as i32 + degrees.round() as i32).rem_euclid(steps) as u16
}

/// The points of a chart along a triangle's axes, for these three values.
pub(super) fn chart(row: Row, turn: u16, values: [u16; 3]) -> [Vec2; 3] {
    let middle = centre(row, turn);
    let corners = corners(row, turn);
    let mut points = corners;
    for (point, (corner, value)) in points.iter_mut().zip(corners.into_iter().zip(values)) {
        let share = (value as f32 / SHOWN_MOST).min(1.0);
        *point = middle + (corner - middle) * share;
    }
    points
}

/// How far a point is outside a convex shape of up to three points (a point, a
/// line or a triangle), negative inside.
pub(super) fn distance_to_shape(point: Vec2, shape: &[Vec2; 3]) -> f32 {
    let edges = [
        (shape[0], shape[1]),
        (shape[1], shape[2]),
        (shape[2], shape[0]),
    ];
    let outside = edges
        .iter()
        .map(|&(from, to)| distance_to_line(point, from, to))
        .fold(f32::MAX, f32::min);
    let area = (shape[1] - shape[0]).perp_dot(shape[2] - shape[0]);
    let inside = area.abs() > f32::EPSILON
        && edges
            .iter()
            .all(|&(from, to)| (to - from).perp_dot(point - from) * area >= 0.0);
    if inside { -outside } else { outside }
}

/// How far a point is from the line from `from` to `to`.
pub(super) fn distance_to_line(point: Vec2, from: Vec2, to: Vec2) -> f32 {
    let along = to - from;
    let share = if along.length_squared() > 0.0 {
        ((point - from).dot(along) / along.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    point.distance(from + along * share)
}

#[cfg(test)]
mod tests;
