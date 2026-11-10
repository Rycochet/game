//! Underclothes: shirts, braies, breast bands, loincloths, hose, socks, petticoats,
//! linen coifs and wimples. Each is built in millimetres in place on a body frame, so
//! its picture and a dressed character use the same parts. Alone it is drawn as loose
//! as it is cut; worn it keeps close (`Fit::Tucked`), so it never pokes through the
//! clothes and armour over it.

mod colour;
mod head;
mod legs;
mod lower;
mod shape;
mod top;

#[cfg(test)]
mod tests;

pub(in crate::item_art) use self::colour::material_of;
pub(in crate::item_art) use self::shape::Fit;
use bevy::prelude::*;
use shared::body::Frame;
use shared::creature::CreatureKind;
use shared::item::Quality;
use shared::underclothes::{Decoration, Garment, UnderForm};

use self::shape::Cloth;
use super::armour::{Model, Stuff};
use crate::item_art::body::mesh_of;
use crate::item_art::{CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, VIEW_WIDTH};

/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Turned a little to its right and seen a little from above; socks from higher up,
/// so the tops of the feet show.
const TURN: f32 = 0.45;
const TILT: f32 = 0.12;
const TILT_FEET: f32 = 0.55;

/// What can be seen of a piece of underclothing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnderLook {
    pub(in crate::item_art) form: UnderForm,
    /// What it takes up, wide and long, not turned.
    pub(in crate::item_art) cells: (u8, u8),
}

impl UnderLook {
    /// A piece taking up these cells.
    pub fn new(form: UnderForm, cells: (u8, u8)) -> Self {
        Self { form, cells }
    }
}

/// The piece on this frame, as cut (`Fit::Alone`) or worn under others.
pub(in crate::item_art) fn build(form: &UnderForm, frame: &Frame, fit: Fit) -> Model {
    let mut model = Model::default();
    let cloth = Cloth { frame, fit };
    match &form.garment {
        Garment::Shirt(shirt) => cloth.shirt(&mut model, shirt),
        Garment::Braies(braies) => cloth.braies(&mut model, braies),
        Garment::Band(band) => cloth.band(&mut model, band),
        Garment::Loincloth(loin) => cloth.loincloth(&mut model, loin),
        Garment::Hose(hose) => cloth.hose(&mut model, hose),
        Garment::Socks(socks) => cloth.socks(&mut model, socks),
        Garment::Petticoat(skirt) => cloth.petticoat(&mut model, skirt),
        Garment::Coif(coif) => cloth.coif(&mut model, coif),
        Garment::Wimple(wimple) => cloth.wimple(&mut model, wimple),
    }
    decorate(&mut model, form);
    model
}

/// Embroidery and lace show as the edges' lines drawn in thread; a caul's net is gold.
fn decorate(model: &mut Model, form: &UnderForm) {
    let thread = match form.decoration {
        Decoration::Plain => return,
        Decoration::Lace => Stuff::Trim,
        Decoration::Embroidered | Decoration::GoldThread => Stuff::Field,
    };
    for (stuff, _) in &mut model.parts {
        if *stuff == Stuff::Shade {
            *stuff = thread;
        }
    }
}

/// The piece built on an ordinary body for its picture, turned and tipped back, then
/// fitted to its cells.
fn fitted(look: &UnderLook) -> Model {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    let mut picture = build(&look.form, &frame, Fit::Alone);
    let tilt = match look.form.garment {
        Garment::Socks(_) => TILT_FEET,
        _ => TILT,
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

/// A piece of underclothing filling its picture.
pub(in crate::item_art) fn under_parts(
    look: &UnderLook,
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
