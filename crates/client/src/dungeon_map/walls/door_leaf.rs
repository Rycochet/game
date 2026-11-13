//! Doors' leaves, open in their frames, and the lines doors and secret doors
//! are known by through what hides them (owner, 2026-11-09).
//!
//! A leaf is hinged at one side of the hole (from a hash of the cell, only for
//! looks), on the face of the wall it opens to, and drawn as the door hangs
//! (`LeafPose`, from `Level::swing`, or part way there while it swings:
//! `door_swing`).

use bevy::prelude::*;
use shared::dungeon::Cell;

use super::super::mesh::Surface;
use super::super::outlined::{OUTLINE_WIDTH, Prism, add_face, grow_convex};
use super::gate_leaf::gate_hinge;
use super::leaf_pose::{LeafPose, askew};
use super::{
    FRAME_SHADE, HALF_CELL, Layer, Layers, OPENING_HALF, Opening, Scene, Tile, half_thickness,
    heights,
};

/// A leaf's thickness, and the gap left round it in the hole.
const LEAF_THICKNESS: f32 = 1.0;
const LEAF_GAP: f32 = 0.3;

/// How wide the lines round a hole are, shown through what hides it, and how
/// far in front of the wall they stand.
const RIM_WIDTH: f32 = 0.6;
const RIM_OUT: f32 = 0.05;

/// The black line round a secret door: wider than an outline, to stand out
/// from the black lines between a wall's stones.
const CRACK_WIDTH: f32 = 1.0;

/// Two numbers, each 1 or -1, the same every time for a cell.
pub(super) fn sides_of(x: i64, y: i64) -> (f32, f32) {
    let mixed = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let sign = |bit: u64| if mixed >> bit & 1 == 0 { 1.0 } else { -1.0 };
    (sign(33), sign(47))
}

impl Tile<'_> {
    /// A door's leaf, as it hangs, and the lines round its hole for the x-ray.
    pub(super) fn door_leaf(&self, layers: &mut Layers, layer: Layer, opening: &Opening) {
        let gate = gate_hinge(self.scene, (self.x, self.y));
        let (hinge_side, _) = sides_of(self.x, self.y);
        let hinge_side = gate.unwrap_or(hinge_side);
        // As it is drawn now: swinging, or as it hangs.
        let at = (self.x, self.y);
        let pose = self.scene.leaves.get(&at).copied();
        let pose = pose.unwrap_or_else(|| LeafPose::of(self.scene.level, at));
        let (turned, swing) = (pose.turned, pose.way);
        let shape = opening.shape;
        let hinge_a = hinge_side * OPENING_HALF;
        let hinge = self.centre
            + opening.along * hinge_a
            + opening.out * swing * half_thickness(shape, 0.0);
        // Along the leaf from the hinge, and through it.
        let (sin, cos) = turned.sin_cos();
        let way = -opening.along * hinge_side * cos + opening.out * swing * sin;
        let through = -opening.out * swing * cos - opening.along * hinge_side * sin;
        // The leaf's shape: across from the hinge, and up, a little inside
        // the hole.
        // A gate's leaf reaches to the middle of the gate, to meet the other.
        let width = match gate {
            Some(_) => OPENING_HALF + HALF_CELL - LEAF_GAP / 2.0,
            None => 2.0 * OPENING_HALF - LEAF_GAP,
        };
        let mut profile = vec![Vec2::new(0.0, 0.0), Vec2::new(width, 0.0)];
        if gate.is_some() {
            profile.push(Vec2::new(width, opening.top(0.0) - LEAF_GAP));
        }
        let mut tops: Vec<Vec2> = opening
            .columns()
            .into_iter()
            .filter(|a| a.abs() <= OPENING_HALF)
            .map(|a| {
                let across = ((a - hinge_a) * -hinge_side).clamp(0.0, width);
                Vec2::new(across, opening.top(a) - LEAF_GAP)
            })
            .collect();
        tops.sort_by(|p, q| q.x.total_cmp(&p.x));
        tops.dedup_by(|p, q| (p.x - q.x).abs() < 1e-3);
        profile.extend(tops);
        if pose.tilt > 0.0 {
            profile = askew(&profile, pose.tilt);
        }
        let place =
            |point: Vec2, depth: f32| hinge + way * point.x + Vec3::Y * point.y + through * depth;
        let leaf = Prism {
            profile: profile.clone(),
            depth: (0.0, LEAF_THICKNESS),
            place,
        };
        for (corners, normal) in leaf.faces() {
            let shade = FRAME_SHADE * (0.75 + 0.25 * normal.y.abs().max(normal.dot(way).abs()));
            let uv = |p: Vec3| {
                let local = p - hinge;
                [local.dot(way) / 24.0, 1.0 - local.y / 24.0]
            };
            let corners = corners.map(|p| (p, uv(p), shade));
            self.add(layers, layer, Surface::Door, normal, corners);
            add_face(&mut layers.outlined.fill, corners.map(|c| c.0), normal);
        }
        let hull = Prism {
            profile: grow_convex(&profile, OUTLINE_WIDTH),
            depth: (-OUTLINE_WIDTH, LEAF_THICKNESS + OUTLINE_WIDTH),
            place,
        };
        for (corners, normal) in hull.faces() {
            add_face(&mut layers.outlined.black, corners, normal);
            add_face(&mut layers.outlined.lines, corners, normal);
        }
        // A gate stands in a rampart, which has no hole to show the line of.
        let sides: &[f32] = if gate.is_some() { &[] } else { &[1.0, -1.0] };
        for &side in sides {
            self.rim(
                opening,
                side,
                (OPENING_HALF, OPENING_HALF + RIM_WIDTH),
                |corners, normal| {
                    add_face(&mut layers.outlined.lines, corners, normal);
                },
            );
        }
    }

    /// Strips round a door's hole on one face (`side`) of the wall, `range`
    /// from the middle of the cell (the sides) and as far above the top,
    /// following the wall's curve: each given to `add` with its normal.
    fn rim(
        &self,
        opening: &Opening,
        side: f32,
        (inner, outer): (f32, f32),
        mut add: impl FnMut([Vec3; 4], Vec3),
    ) {
        let shape = opening.shape;
        let normal = opening.out * side;
        let point = |a: f32, height: f32| {
            self.centre
                + opening.along * a
                + Vec3::Y * height
                + normal * (half_thickness(shape, height) + RIM_OUT)
        };
        let above = outer - inner;
        for sign in [1.0f32, -1.0] {
            let (a0, a1) = (sign * inner, sign * outer);
            let cuts = heights(shape, 0.0, opening.top(sign * OPENING_HALF) + above);
            for band in cuts.windows(2) {
                let corners = [
                    point(a0, band[0]),
                    point(a1, band[0]),
                    point(a1, band[1]),
                    point(a0, band[1]),
                ];
                add(corners, normal);
            }
        }
        let columns: Vec<f32> = opening
            .columns()
            .into_iter()
            .filter(|a| a.abs() <= OPENING_HALF)
            .collect();
        for pair in columns.windows(2) {
            let low = |a: f32| opening.top(a) + (inner - OPENING_HALF);
            let corners = [
                point(pair[0], low(pair[0])),
                point(pair[1], low(pair[1])),
                point(pair[1], low(pair[1]) + above),
                point(pair[0], low(pair[0]) + above),
            ];
            add(corners, normal);
        }
    }
}

/// A known secret door (only sent to truesight), drawn as wall: the black
/// line round where it would open, on the faces towards the camera, and
/// the same lines for the x-ray on both.
pub fn build_secret_door(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    if scene.level.cell(x, y) != Cell::SecretDoor {
        return;
    }
    let east_west = scene.level.opening_runs_east_west(x, y);
    let (along, out) = if east_west {
        (Vec3::X, Vec3::Z)
    } else {
        (Vec3::Z, Vec3::X)
    };
    let tile = Tile::new(scene, (x, y), [false; 4]);
    let opening = Opening {
        along,
        out,
        window: false,
        shape: scene.shape,
    };
    let layer = if scene.sight.is_seen(x, y) {
        Layer::Solid
    } else {
        Layer::Memory
    };
    let half = CRACK_WIDTH / 2.0;
    let range = (OPENING_HALF - half, OPENING_HALF + half);
    for side in [1.0, -1.0] {
        let towards = super::super::mesh::faces_camera(tile.centre, out * side, scene.camera);
        tile.rim(&opening, side, range, |corners, normal| {
            if towards {
                tile.add(
                    layers,
                    layer,
                    Surface::Wall,
                    normal,
                    corners.map(|p| (p, [0.0; 2], 0.0)),
                );
            }
            add_face(&mut layers.outlined.lines, corners, normal);
        });
    }
}
