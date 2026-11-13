//! Markings (`shared::sheet::Marks`): the skin's triangles split into those in the
//! coat's colour and those in the markings', by where each is on the body: under the
//! trunk's middle for a pale belly, parts of a measured head for a face, and patches
//! from the markings' own number laid over the trunk, neck, head and tops of the legs.
//! Triangles are halved along each side first, so the edges are less jagged.

use bevy::prelude::*;
use shared::body::{HEAD_RINGS, Layout, MAX_BONES};
use shared::sheet::{Face, Marks};

use super::super::model::{BodyPart, Bones};
use super::{HeadAxes, Triangle, point, trunk};

/// The skin's triangles in the coat's colour, and those in the markings'.
pub(in crate::item_art) fn split(
    layout: &Layout,
    skin: Vec<Triangle>,
    marks: Option<Marks>,
) -> (Vec<Triangle>, Vec<Triangle>) {
    let Some(marks) = marks else {
        return (skin, Vec::new());
    };
    let body = Body::of(layout);
    halved(skin)
        .into_iter()
        .partition(|triangle| !body.marked(&marks, (triangle[0] + triangle[1] + triangle[2]) / 3.0))
}

/// Each triangle as four, by the middles of its sides.
fn halved(triangles: Vec<Triangle>) -> Vec<Triangle> {
    triangles
        .into_iter()
        .flat_map(|[a, b, c]| {
            let (ab, bc, ca) = ((a + b) / 2.0, (b + c) / 2.0, (c + a) / 2.0);
            [[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]]
        })
        .collect()
}

/// What telling where a point is takes.
struct Body<'a> {
    layout: &'a Layout,
    bones: Bones,
    /// How big a patch is, in millimetres: a share of the trunk's length.
    patch: f32,
}

/// How much of the trunk's length a patch's noise repeats over.
const PATCH_SHARE: f32 = 0.3;
/// How much of the patched parts is marked, from 0 (all) to 1 (none).
const PATCH_LEVEL: f32 = 0.52;

impl<'a> Body<'a> {
    fn of(layout: &'a Layout) -> Self {
        let rings = &layout.spine.rings;
        let length = (rings[rings.len() - 1].along - rings[0].along) as f32;
        Body {
            layout,
            bones: Bones::of(layout),
            patch: (length * PATCH_SHARE).max(60.0),
        }
    }

    /// Whether a point of the skin is in the markings' colour.
    fn marked(&self, marks: &Marks, at: Vec3) -> bool {
        let part = self.bones.part_at(at);
        let face = match part {
            BodyPart::HEAD => place_on_head(self.layout, at),
            _ => None,
        };
        // Ears and horns stand out of the head's outline, and stay the coat's.
        let on_head = face.filter(|place| place.inside);
        if part == BodyPart::HEAD && on_head.is_none() {
            return false;
        }
        let belly = marks.belly
            && match part {
                BodyPart::TRUNK => trunk::underside(self.layout, at) > 0.15,
                BodyPart::NECK => self.under_neck(at),
                BodyPart::HEAD => on_head.is_some_and(|place| place.up < -0.25),
                _ => false,
            };
        let face = on_head.is_some_and(|place| match marks.face {
            Face::Bare => false,
            Face::Mask => place.along > 0.2 && place.up < 0.3,
            Face::Blaze => place.along > 0.25 && place.up > 0.2 && place.across.abs() < 0.35,
            Face::Whole => true,
        });
        let patched = marks
            .patches
            .is_some_and(|layout| self.patched(part) && self.patch_at(layout, at));
        belly || face || patched
    }

    /// Whether a point is on the underside of the neck.
    fn under_neck(&self, at: Vec3) -> bool {
        let root = trunk::neck_root(self.layout);
        let chin = point(self.layout.chin());
        let up_neck = (chin - root).normalize_or(Vec3::Y);
        let under = Vec3::X.cross(up_neck).normalize_or(Vec3::NEG_Y);
        (at - root).dot(under) > self.layout.neck.radius as f32 * 0.3
    }

    /// Whether patches go on this part: the trunk, neck and head, and the top two bones
    /// of each limb.
    fn patched(&self, part: u32) -> bool {
        let part = part % BodyPart::UPPER;
        part <= BodyPart::HEAD || (part - 3) as usize % MAX_BONES < 2
    }

    /// Whether this point is in a patch of this layout.
    fn patch_at(&self, layout: u8, at: Vec3) -> bool {
        noise(at / self.patch, u32::from(layout)) > PATCH_LEVEL
    }
}

/// Where a point is on a measured head, each in shares: from the back of the skull
/// (0) to the nose (1), up from the middle of its ring and across in the ring's half
/// sizes, and whether it is within the head's outline (not on an ear or a horn).
#[derive(Clone, Copy)]
struct Place {
    along: f32,
    up: f32,
    across: f32,
    inside: bool,
}

fn place_on_head(layout: &Layout, at: Vec3) -> Option<Place> {
    let shape = layout.head.shape?;
    let (along, up, across) = HeadAxes::of(layout, &shape).local(at);
    let ring = trunk::ring_at(&shape.rings, along);
    let deep = ((ring.belly + ring.back) as f32 / 2.0).max(1.0);
    let up = (up - (ring.back - ring.belly) as f32 / 2.0) / deep;
    let across = across / (ring.half as f32).max(1.0);
    let (back, nose) = (
        shape.rings[0].along as f32,
        shape.rings[HEAD_RINGS - 1].along as f32,
    );
    Some(Place {
        along: (along - back) / (nose - back).max(1.0),
        up,
        across,
        inside: up * up + across * across <= 1.5 && along >= back - 10.0 && along <= nose + 10.0,
    })
}

/// Smooth noise from 0 to 1, changing over about one unit, different for each seed.
fn noise(at: Vec3, seed: u32) -> f32 {
    let base = at.floor();
    let t = at - base;
    let smooth = t * t * (Vec3::splat(3.0) - 2.0 * t);
    let corner = |x: i32, y: i32, z: i32| {
        hashed(
            [base.x as i32 + x, base.y as i32 + y, base.z as i32 + z],
            seed,
        )
    };
    let along_x = |y, z| corner(0, y, z) + (corner(1, y, z) - corner(0, y, z)) * smooth.x;
    let along_y = |z| along_x(0, z) + (along_x(1, z) - along_x(0, z)) * smooth.y;
    along_y(0) + (along_y(1) - along_y(0)) * smooth.z
}

/// A number from 0 to 1 for a corner of the noise's grid.
fn hashed([x, y, z]: [i32; 3], seed: u32) -> f32 {
    let mut h = seed.wrapping_mul(0x9e37_79b9)
        ^ (x as u32).wrapping_mul(0x85eb_ca6b)
        ^ (y as u32).wrapping_mul(0xc2b2_ae35)
        ^ (z as u32).wrapping_mul(0x27d4_eb2f);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h & 0xffff) as f32 / 65535.0
}

#[cfg(test)]
mod tests;
