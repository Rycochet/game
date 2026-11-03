//! A hedge along one side of a cell: boxes and arches.

use bevy::prelude::*;

use super::super::walls::{Layer, Layers, Scene};
use super::{
    ARCH_HALF, ARCH_HEIGHT, HALF_CELL, HEDGE_INSET, HEDGE_SIDE_SHADE, HEDGE_THICK, HEDGE_TILE,
    atlas_uv, emit, layer_colour,
};

/// A hedge along one side of a cell.
pub(super) struct Hedge {
    pub(super) centre: Vec3,
    pub(super) out: Vec3,
    pub(super) along: Vec3,
    pub(super) layer: Layer,
}

impl Hedge {
    /// A box from `low` to `high` along the side, `up` high.
    pub(super) fn stretch(&self, layers: &mut Layers, scene: &Scene, low: f32, high: f32, up: f32) {
        if low < high {
            self.solid(layers, scene, (low, high), (0.0, up));
        }
    }

    /// Two posts and a stepped top, all of hedge.
    pub(super) fn arch(&self, layers: &mut Layers, scene: &Scene) {
        let (inner, outer) = (ARCH_HALF, ARCH_HALF + HEDGE_THICK);
        self.solid(layers, scene, (-outer, -inner), (0.0, ARCH_HEIGHT));
        self.solid(layers, scene, (inner, outer), (0.0, ARCH_HEIGHT));
        let beam = ARCH_HEIGHT - HEDGE_THICK;
        self.solid(layers, scene, (-outer, outer), (beam, ARCH_HEIGHT));
        // Corners stepping in, to look rounded.
        let step = ARCH_HALF / 3.0;
        let low = beam - HEDGE_THICK;
        self.solid(layers, scene, (-inner, -inner + step), (low, beam));
        self.solid(layers, scene, (inner - step, inner), (low, beam));
    }

    /// A box of hedge, `(low, high)` along the side and up.
    fn solid(&self, layers: &mut Layers, scene: &Scene, along: (f32, f32), up: (f32, f32)) {
        let far = HALF_CELL - HEDGE_INSET;
        let near = far - HEDGE_THICK;
        let at = |a: f32, o: f32, h: f32| self.centre + self.along * a + self.out * o + Vec3::Y * h;
        let shade = layer_colour(self.layer);
        let side = shade * HEDGE_SIDE_SHADE;
        let uv = |u: f32, v: f32| atlas_uv(HEDGE_TILE, u, v);
        let uvs = [uv(0.1, 0.5), uv(0.9, 0.5), uv(0.9, 0.1), uv(0.1, 0.1)];
        let face = |layers: &mut Layers, corners: [Vec3; 4], normal: Vec3, colour: Vec3| {
            emit(layers, scene, self.layer, corners, normal, uvs, [colour; 4]);
        };
        let (a0, a1) = along;
        let (h0, h1) = up;
        face(
            layers,
            [
                at(a0, near, h1),
                at(a1, near, h1),
                at(a1, far, h1),
                at(a0, far, h1),
            ],
            Vec3::Y,
            shade,
        );
        face(
            layers,
            [
                at(a0, far, h0),
                at(a1, far, h0),
                at(a1, far, h1),
                at(a0, far, h1),
            ],
            self.out,
            side,
        );
        face(
            layers,
            [
                at(a0, near, h0),
                at(a1, near, h0),
                at(a1, near, h1),
                at(a0, near, h1),
            ],
            -self.out,
            side,
        );
        face(
            layers,
            [
                at(a1, near, h0),
                at(a1, far, h0),
                at(a1, far, h1),
                at(a1, near, h1),
            ],
            self.along,
            side,
        );
        face(
            layers,
            [
                at(a0, near, h0),
                at(a0, far, h0),
                at(a0, far, h1),
                at(a0, near, h1),
            ],
            -self.along,
            side,
        );
    }
}
