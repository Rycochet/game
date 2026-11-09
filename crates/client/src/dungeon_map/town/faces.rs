//! Upright faces of walls, the layer each goes in, and the ground under walls.

use bevy::prelude::*;

use super::super::mesh::{Surface, WallFace, faces_camera};
use super::super::walls::{Layer, Layers, Scene};
use super::super::{CELL_SIZE, WALL_HEIGHT};
use super::{Opening, SIDEWAYS_SHADE, atlas_uv, cell_centre, emit, layer_colour, light_at};

/// A face of a wall (or a door or window on it), upright, running along `x` or
/// `z`, facing `normal`, from `(a, ..)` to `(b, ..)` along it.
#[derive(Clone, Debug)]
pub(super) struct Piece {
    /// A point on the bottom of the face at one end, and at the other.
    pub(super) from: Vec3,
    pub(super) to: Vec3,
    pub(super) normal: Vec3,
    /// Picture coordinates across, at each end.
    pub(super) across: (f32, f32),
}

/// Which layer a face goes in, or none: seen and facing the camera it is
/// solid; seen and facing away, see-through; unseen and facing it, memory.
pub(super) fn face_layer(
    scene: &Scene,
    own: (i64, i64),
    outer: (i64, i64),
    piece: &Piece,
    hides: bool,
) -> Option<Layer> {
    let level = scene.level;
    let middle = (piece.from + piece.to) / 2.0;
    let in_front = scene.views.iter().any(|view| {
        let viewer = cell_centre(view.cell.0, view.cell.1);
        piece.normal.dot(viewer - middle) >= 0.0
            && if level.is_open(outer.0, outer.1) {
                view.sight.is_seen(outer.0, outer.1)
            } else {
                view.sight.is_seen(own.0, own.1)
            }
    });
    let facing = faces_camera(middle, piece.normal, scene.camera);
    match (in_front, facing) {
        _ if hides && (in_front || facing) => Some(Layer::SeeThrough),
        (true, true) => Some(Layer::Solid),
        (true, false) => Some(Layer::SeeThrough),
        (false, true) => Some(Layer::Memory),
        (false, false) => None,
    }
}

pub(super) fn side_shade(normal: Vec3) -> f32 {
    if normal.x != 0.0 { SIDEWAYS_SHADE } else { 1.0 }
}

/// A face from the ground up to `height`, in `tile`.
pub(super) fn upright(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    piece: &Piece,
    (low, high): (f32, f32),
    tile: usize,
) {
    let colour = layer_colour(layer) * side_shade(piece.normal);
    let (v_low, v_high) = (1.0 - low / WALL_HEIGHT, 1.0 - high / WALL_HEIGHT);
    emit(
        layers,
        scene,
        layer,
        [
            piece.from + Vec3::Y * low,
            piece.to + Vec3::Y * low,
            piece.to + Vec3::Y * high,
            piece.from + Vec3::Y * high,
        ],
        piece.normal,
        [
            atlas_uv(tile, piece.across.0, v_low),
            atlas_uv(tile, piece.across.1, v_low),
            atlas_uv(tile, piece.across.1, v_high),
            atlas_uv(tile, piece.across.0, v_high),
        ],
        [colour; 4],
    );
}

/// A wall's face: goes into whichever layer it belongs in, round any door or
/// window `opening` on its cell.
#[allow(clippy::too_many_arguments)]
pub(super) fn wall_face(
    layers: &mut Layers,
    scene: &Scene,
    (own, outer): ((i64, i64), (i64, i64)),
    piece: Piece,
    tile: usize,
    hides: bool,
    opening: Option<&Opening>,
) {
    let Some(layer) = face_layer(scene, own, outer, &piece, hides) else {
        return;
    };
    match opening {
        Some(opening) => {
            for (part, heights) in opening.around(&piece, own) {
                upright(layers, scene, layer, &part, heights, tile);
            }
        }
        None => upright(layers, scene, layer, &piece, (0.0, WALL_HEIGHT), tile),
    }
    if layer == Layer::Solid {
        let middle = (piece.from + piece.to) / 2.0;
        layers.solid_faces.push(WallFace {
            centre: middle + Vec3::Y * WALL_HEIGHT / 2.0,
            normal: piece.normal,
            half_across: (piece.to - piece.from).length() / 2.0,
            half_up: WALL_HEIGHT / 2.0,
        });
    }
}

/// The ground under a wall's outer half is always the land's own, whatever
/// the cell beyond it is: it shows as seen if that cell is.
pub(super) fn strip(
    layers: &mut Layers,
    scene: &Scene,
    centre: Vec3,
    patch: Rect,
    neighbours: &[(i64, i64)],
) {
    let seen = neighbours.iter().any(|&(x, y)| scene.sight.is_seen(x, y));
    let layer = if seen { Layer::Solid } else { Layer::Memory };
    ground_patch(layers, scene, layer, centre, patch);
}

/// A piece of the land's own ground.
pub(super) fn ground_patch(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    centre: Vec3,
    patch: Rect,
) {
    let corners = [
        (patch.min.x, patch.min.y),
        (patch.max.x, patch.min.y),
        (patch.max.x, patch.max.y),
        (patch.min.x, patch.max.y),
    ];
    let positions = corners.map(|(east, south)| centre + Vec3::new(east, 0.0, south));
    let colours = positions.map(|at| {
        if layer == Layer::Memory {
            layer_colour(layer)
        } else {
            light_at(scene, at)
        }
    });
    let builder = layers.builder(layer, Surface::Ground);
    builder.free(
        positions,
        Vec3::Y,
        corners.map(|(east, south)| [east / CELL_SIZE + 0.5, south / CELL_SIZE + 0.5]),
        colours,
    );
}
