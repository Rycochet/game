//! Blocks of wood: posts, bars and rails.

use bevy::prelude::*;

use super::super::town::{self, FENCE_TILE};
use super::super::walls::{Layer, Layers, Scene};
use super::{POST_HALF, RAIL_HALF, RAIL_HEIGHTS, WOOD_SIDE_SHADE};

/// A block of wood. Its bottom isn't drawn.
pub(super) fn wood(layers: &mut Layers, scene: &Scene, layer: Layer, low: Vec3, high: Vec3) {
    let uvs = [(0.1, 0.5), (0.9, 0.5), (0.9, 0.1), (0.1, 0.1)];
    let uvs = uvs.map(|(u, v)| town::atlas_uv(FENCE_TILE, u, v));
    let top = town::layer_colour(layer);
    let side = top * WOOD_SIDE_SHADE;
    let at = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
    let (l, h) = (low, high);
    let faces = [
        (
            [
                at(l.x, h.y, l.z),
                at(h.x, h.y, l.z),
                at(h.x, h.y, h.z),
                at(l.x, h.y, h.z),
            ],
            Vec3::Y,
            top,
        ),
        (
            [
                at(l.x, l.y, h.z),
                at(h.x, l.y, h.z),
                at(h.x, h.y, h.z),
                at(l.x, h.y, h.z),
            ],
            Vec3::Z,
            side,
        ),
        (
            [
                at(l.x, l.y, l.z),
                at(h.x, l.y, l.z),
                at(h.x, h.y, l.z),
                at(l.x, h.y, l.z),
            ],
            Vec3::NEG_Z,
            side,
        ),
        (
            [
                at(h.x, l.y, l.z),
                at(h.x, l.y, h.z),
                at(h.x, h.y, h.z),
                at(h.x, h.y, l.z),
            ],
            Vec3::X,
            side,
        ),
        (
            [
                at(l.x, l.y, l.z),
                at(l.x, l.y, h.z),
                at(l.x, h.y, h.z),
                at(l.x, h.y, l.z),
            ],
            Vec3::NEG_X,
            side,
        ),
    ];
    for (corners, normal, colour) in faces {
        town::emit(layers, scene, layer, corners, normal, uvs, [colour; 4]);
    }
}

/// A post, standing `up` high at a point on the ground.
pub(super) fn post(layers: &mut Layers, scene: &Scene, layer: Layer, at: Vec3, up: f32) {
    let half = Vec3::new(POST_HALF, 0.0, POST_HALF);
    wood(layers, scene, layer, at - half, at + half + Vec3::Y * up);
}

/// A bar from one point to another along a grid direction, `half` thick each
/// way.
pub(super) fn bar(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    from: Vec3,
    to: Vec3,
    half: f32,
) {
    let margin = Vec3::splat(half);
    wood(
        layers,
        scene,
        layer,
        from.min(to) - margin,
        from.max(to) + margin,
    );
}

/// The two rails of a fence from one point to another on the ground.
pub(super) fn rails(layers: &mut Layers, scene: &Scene, layer: Layer, from: Vec3, to: Vec3) {
    for height in RAIL_HEIGHTS {
        let up = Vec3::Y * height;
        bar(layers, scene, layer, from + up, to + up, RAIL_HALF);
    }
}
