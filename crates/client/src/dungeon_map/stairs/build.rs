//! Drawing a staircase on its cell, floor included.

use bevy::prelude::*;
use shared::protocol::SeenStairs;
use shared::site::{StairsDirection, StairsStyle};

use super::super::CELL_SIZE;
use super::super::mesh::{Builder, Surface};
use super::{
    Model, STAIRS_DOWN_DEPTH, STAIRS_UP_HEIGHT, STAIRS_UP_TOP, TURNING_UP_HEIGHT, UP_SPACING,
    floor_whole, floor_with_hole, light_under_ladder, shaft,
};

/// The shape of a hole, centred on its cell.
#[derive(Clone, Copy)]
pub(super) enum Hole {
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
