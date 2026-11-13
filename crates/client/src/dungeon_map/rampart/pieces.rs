//! A wall's pieces in one cell: its run to the cells of the wall beside it,
//! and a gate's post.

use bevy::prelude::*;
use shared::site::{WallLook, WallMaterial};

use super::super::town;
use super::super::walls::{Layer, Layers, Scene};
use super::{Build, HALF_CELL, block};
use crate::dungeon_map::{CELL_SIZE, WALL_HEIGHT};

/// The four ways out of a cell, as steps (`y` south).
const SIDES: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// Logs stand this far apart along a palisade, each a little taller or
/// shorter than the next by a hash of where it stands, with a point on top.
const LOG_SPACING: f32 = CELL_SIZE / 3.0;
const LOG_HEIGHTS: f32 = WALL_HEIGHT * 0.08;
const LOG_POINT: f32 = CELL_SIZE * 0.07;

/// A plank wall's posts: half their thickness, and how far they stand above
/// its boards.
const PLANK_POST: f32 = CELL_SIZE * 0.1;
const PLANK_POST_ABOVE: f32 = WALL_HEIGHT * 0.06;

/// Merlons on a stone wall: half their length, and their height.
const MERLON_HALF: f32 = CELL_SIZE * 0.16;
const MERLON_HEIGHT: f32 = WALL_HEIGHT * 0.18;

/// A gate's posts: from where the leaf is hinged to the edge of the cell,
/// standing this much above the wall.
const GATE_POST_FROM: f32 = CELL_SIZE * 0.41;
const GATE_POST_ABOVE: f32 = WALL_HEIGHT * 0.15;

fn layer_of(scene: &Scene, (x, y): (i64, i64)) -> Layer {
    town::flat_layer(scene, x, y)
}

/// A box round a line from `from` to `to` on the ground, `half` thick each
/// way, `height` high.
fn along(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    build: &Build,
    (from, to): (Vec3, Vec3),
    (half, height): (f32, f32),
) {
    let margin = Vec3::new(half, 0.0, half);
    let low = from.min(to) - margin;
    let high = from.max(to) + margin + Vec3::Y * height;
    block(layers, scene, layer, build, (low, high));
}

/// One cell of a wall's run, joined to the cells of the same wall beside it.
pub(super) fn build_run(layers: &mut Layers, scene: &Scene, wall: &WallLook, (x, y): (i64, i64)) {
    let layer = layer_of(scene, (x, y));
    let build = Build::of(wall.material);
    let centre = town::cell_centre(x, y);
    let joined: Vec<Vec3> = (SIDES.iter())
        .filter(|(dx, dy)| wall.has(x + dx, y + dy))
        .map(|&(dx, dy)| Vec3::new(dx as f32, 0.0, dy as f32))
        .collect();
    match wall.material {
        WallMaterial::Log => {
            let log = |layers: &mut Layers, at: Vec3| {
                let hash = (at.x * 7.3 + at.z * 3.1).sin().abs();
                let height = build.height - LOG_HEIGHTS * hash;
                along(layers, scene, layer, &build, (at, at), (build.half, height));
                let point = at + Vec3::Y * height;
                along(
                    layers,
                    scene,
                    layer,
                    &build,
                    (point, point),
                    (build.half / 2.0, LOG_POINT),
                );
            };
            log(layers, centre);
            for way in &joined {
                log(layers, centre + *way * LOG_SPACING);
            }
        }
        WallMaterial::Plank => {
            let post = build.height + PLANK_POST_ABOVE;
            along(
                layers,
                scene,
                layer,
                &build,
                (centre, centre),
                (PLANK_POST, post),
            );
            for way in &joined {
                let edge = centre + *way * HALF_CELL;
                along(
                    layers,
                    scene,
                    layer,
                    &build,
                    (centre, edge),
                    (build.half, build.height),
                );
            }
        }
        WallMaterial::Stone | WallMaterial::Block => {
            let (half, height) = (build.half, build.height);
            along(
                layers,
                scene,
                layer,
                &build,
                (centre, centre),
                (half, height),
            );
            for way in &joined {
                let edge = centre + *way * HALF_CELL;
                along(layers, scene, layer, &build, (centre, edge), (half, height));
            }
            let top = centre + Vec3::Y * height;
            let merlon = (MERLON_HALF.min(half), MERLON_HEIGHT);
            along(layers, scene, layer, &build, (top, top), merlon);
        }
    }
}

/// A gate's post, on the side of its door away from the other: from the leaf's
/// hinge to the wall beside it, a little higher than the wall.
pub(super) fn build_gate_post(
    layers: &mut Layers,
    scene: &Scene,
    wall: &WallLook,
    (x, y): (i64, i64),
) {
    let Some(gate) = (wall.gates.iter()).find(|gate| gate.cells().contains(&(x, y))) else {
        return;
    };
    let layer = layer_of(scene, (x, y));
    let build = Build::of(wall.material);
    let [first, _] = gate.cells();
    let outward = if (x, y) == first { -1.0 } else { 1.0 };
    let way = if gate.runs_east_west {
        Vec3::X
    } else {
        Vec3::Z
    };
    let centre = town::cell_centre(x, y);
    let from = centre + way * outward * GATE_POST_FROM;
    let to = centre + way * outward * HALF_CELL;
    let half = build.half.max(PLANK_POST);
    let height = build.height + GATE_POST_ABOVE;
    let middle = (from + to) / 2.0;
    let reach = (to - from).length() / 2.0;
    let margin = way * reach + (Vec3::ONE - way - Vec3::Y) * half;
    let low = middle - margin;
    let high = middle + margin + Vec3::Y * height;
    block(layers, scene, layer, &build, (low.min(high), low.max(high)));
}
