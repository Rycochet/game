//! Footwear: shoes, boots, sandals, clogs and slippers. Each shoe is built in
//! millimetres in place on a body frame (`build_foot`), over foot armour's `Last`, so
//! sabatons built over it fit and a dressed character uses the same parts; the
//! picture shows the pair alone. The sole goes under the frame's ground.

mod adorn;
mod colour;
mod fasten;
mod shoe;
mod sole;
mod straps;
mod upper;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use shared::body::{Frame, Side};
use shared::creature::CreatureKind;
use shared::footwear::FootwearForm;
use shared::item::Quality;

use self::shoe::Shoe;
use super::armour::Model;
use super::armour::feet::Foot;
use crate::item_art::body::mesh_of;
use crate::item_art::{CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, VIEW_WIDTH};

/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Turned to its right and seen a little from above; low footwear from higher up, so
/// the tops of the feet show.
const TURN: f32 = 0.75;
const TILT_LOW: f32 = 0.55;
const TILT_BOOTS: f32 = 0.25;

/// What can be seen of a pair of footwear.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FootwearLook {
    pub(in crate::item_art) form: FootwearForm,
    /// What it takes up, wide and long, not turned.
    pub(in crate::item_art) cells: (u8, u8),
}

impl FootwearLook {
    /// A pair of footwear taking up these cells.
    pub fn new(form: FootwearForm, cells: (u8, u8)) -> Self {
        Self { form, cells }
    }
}

/// A shoe on one foot of this frame, as worn.
pub(in crate::item_art) fn build_foot(form: &FootwearForm, frame: &Frame, side: Side) -> Model {
    let mut model = Model::default();
    let foot = Foot { frame, side };
    let shoe = Shoe::new(form, &foot);
    sole::add_sole(&mut model, &shoe);
    if form.open() {
        straps::add_straps(&mut model, &shoe);
    } else {
        upper::add_upper(&mut model, &shoe);
        fasten::add_fastening(&mut model, &shoe);
    }
    adorn::add_decoration(&mut model, &shoe);
    model
}

/// A pair on both feet, as worn: for the dressed character.
pub(in crate::item_art) fn build(form: &FootwearForm, frame: &Frame) -> Model {
    let mut model = Model::default();
    for side in Side::BOTH {
        model.join(build_foot(form, frame, side));
    }
    model
}

/// The pair built on an ordinary body for its picture, turned and tipped back, then
/// fitted to its cells.
fn fitted(look: &FootwearLook) -> Model {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    let mut picture = build(&look.form, &frame);
    let tilt = if look.form.boot() {
        TILT_BOOTS
    } else {
        TILT_LOW
    };
    let view = Quat::from_rotation_x(tilt) * Quat::from_rotation_y(TURN);
    picture.map(|point| view * point);
    let (low, high) = picture.bounds();
    let (wide, long) = (look.cells.0 as f32, look.cells.1 as f32);
    let world_per_pixel = VIEW_WIDTH / (wide.max(long) * PIXELS_PER_CELL);
    let room = |cells: f32| (cells * PIXELS_PER_CELL - 2.0 * MARGIN_PIXELS) * world_per_pixel;
    let size = high - low;
    let scale = (room(long) / size.y).min(room(wide) / size.x);
    let middle = (low + high) / 2.0;
    picture.map(|point| (point - middle) * scale);
    picture
}

/// A pair of footwear filling its picture.
pub(in crate::item_art) fn footwear_parts(
    look: &FootwearLook,
    quality: Option<Quality>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    fitted(look)
        .parts
        .into_iter()
        .filter(|(_, triangles)| !triangles.is_empty())
        .map(|(stuff, triangles)| Part {
            mesh: meshes.add(mesh_of(&triangles)),
            material: materials.add(colour::material_of(stuff, &look.form, quality)),
            transform: Transform::from_rotation(camera.rotation),
        })
        .collect()
}
