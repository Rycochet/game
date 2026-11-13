//! Which part of the body each point of a model moves with, for animating it later:
//! the bone it is nearest, the trunk, neck, head or a limb's bone.

use bevy::mesh::{MeshVertexAttribute, VertexFormat};
use bevy::prelude::*;
use shared::body::{Layout, MAX_BONES};

use super::super::body::{HeadAxes, neck_root};

/// On every vertex of a creature model's parts: the `BodyPart` it moves with.
pub const ATTRIBUTE_BODY_PART: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_BodyPart", 0x484f_5701, VertexFormat::Uint32);

/// A part of the body, as a number on each vertex: `TRUNK`, `NECK`, `HEAD`, then each
/// limb's bones in turn (`limb`), and the same again past `UPPER` for a taur's upper
/// body.
pub struct BodyPart;

impl BodyPart {
    pub const TRUNK: u32 = 0;
    pub const NECK: u32 = 1;
    pub const HEAD: u32 = 2;
    /// Added for a taur's upper body.
    pub const UPPER: u32 = 64;

    /// Bone `bone` (from where it hangs) of the limb in slot `slot` of its layout.
    pub fn limb(slot: usize, bone: usize) -> u32 {
        (3 + slot * MAX_BONES + bone) as u32
    }
}

/// A body's bones as lines between points, in millimetres.
pub(in crate::item_art) struct Bones(Vec<(u32, Vec3, Vec3)>);

fn point(mm: [i32; 3]) -> Vec3 {
    Vec3::new(mm[0] as f32, mm[1] as f32, mm[2] as f32)
}

impl Bones {
    /// The bones of this body, and of a taur's upper body.
    pub(in crate::item_art) fn of(layout: &Layout) -> Self {
        let mut bones = Vec::new();
        add(&mut bones, layout, 0);
        if let Some(upper) = layout.upper_body() {
            add(&mut bones, &upper, BodyPart::UPPER);
        }
        Self(bones)
    }

    /// The part a point moves with: the bone nearest it.
    pub(in crate::item_art) fn part_at(&self, at: Vec3) -> u32 {
        let distance = |(_, from, to): &(u32, Vec3, Vec3)| {
            let along = *to - *from;
            let share = (at - *from).dot(along) / along.length_squared().max(1.0);
            at.distance(*from + along * share.clamp(0.0, 1.0))
        };
        self.0
            .iter()
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
            .map_or(BodyPart::TRUNK, |bone| bone.0)
    }
}

/// One body's bones, numbered from `first`.
fn add(bones: &mut Vec<(u32, Vec3, Vec3)>, layout: &Layout, first: u32) {
    let spine = &layout.spine;
    // Where the neck is drawn from, which a measured beast's trunk climbs to.
    let neck_base = neck_root(layout);
    bones.push((first + BodyPart::TRUNK, point(spine.at(0)), neck_base));
    if layout.head.length > 0 {
        let chin = point(layout.chin());
        bones.push((first + BodyPart::NECK, neck_base, chin));
        // Up to the crown, or along a measured head to its nose.
        let end = match layout.head.shape {
            Some(shape) => {
                let nose = shape.rings[shape.rings.len() - 1];
                HeadAxes::of(layout, &shape).at(nose.along as f32, 0.0, 0.0)
            }
            None => chin + Vec3::Y * layout.head.length as f32,
        };
        bones.push((first + BodyPart::HEAD, chin, end));
    }
    for (slot, limb) in layout.limbs.iter().enumerate() {
        let Some(limb) = limb else {
            continue;
        };
        let joints = layout.joints(limb);
        for (bone, pair) in joints.windows(2).enumerate() {
            let part = first + BodyPart::limb(slot, bone);
            bones.push((part, point(pair[0]), point(pair[1])));
        }
    }
}
