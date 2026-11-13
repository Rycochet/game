//! A gate's leaf (`shared::door::gate_partner`): one of two doors side by side
//! in a wall round a site (`shared::site::WallLook`). The wall and its gate
//! posts are the rampart's (`rampart`); here only the floor and the leaf, hinged
//! at the gate's outer side so the two leaves meet in the middle, as one gate.

use bevy::prelude::*;
use shared::door::gate_partner;

use super::super::theme::Shape;
use super::{Layer, Layers, Opening, QUARTERS, Scene, Tile};

/// The floor and leaf of the gate's door at `(x, y)`, whose cell is remembered.
pub fn build_gate_leaf(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let east_west = scene.level.opening_runs_east_west(x, y);
    let joins = [!east_west, east_west, !east_west, east_west];
    let tile = Tile::new(scene, (x, y), joins);
    let layer = if scene.sight.is_seen(x, y) {
        Layer::Solid
    } else {
        Layer::Memory
    };
    for quarter in QUARTERS {
        tile.floor_quarter(layers, layer, quarter, &[], &[]);
    }
    let (along, out) = if east_west {
        (Vec3::X, Vec3::Z)
    } else {
        (Vec3::Z, Vec3::X)
    };
    let opening = Opening {
        along,
        out,
        window: false,
        shape: Shape::Built,
    };
    tile.door_leaf(layers, layer, &opening);
}

/// Which side of its cell (1 or -1 along the wall) a door's leaf is hinged
/// at, if it is a gate's: away from the other leaf.
pub(super) fn gate_hinge(scene: &Scene, (x, y): (i64, i64)) -> Option<f32> {
    let (px, py) = gate_partner(scene.level, x, y)?;
    Some(if px > x || py > y { -1.0 } else { 1.0 })
}
