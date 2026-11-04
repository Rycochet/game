//! Hand pieces: gauntlets of every construction, demi-gauntlets, tekkō and padded
//! gloves. Each hand is built in millimetres in place on a body frame, hanging from the
//! wrist with the palm towards the thigh and the thumb forwards, as the bare body draws
//! it; its picture lays both hands down side by side. A cuff sits 28 mm out from the
//! forearm, over the end of any vambrace (agreed with the arms' pieces).

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::hands::{Fingers, Hand, HandsForm, Knuckles};
use shared::body::{Frame, Side};
use shared::creature::CreatureKind;
use shared::item::Quality;
use shared::material::{Category, Material};

use super::{Model, Stuff, colour_of, darker, matte};
use crate::item_art::body::mesh_of;
use crate::item_art::ring::metal_material;
use crate::item_art::{CAMERA_AT, CAMERA_UP, HandsLook, OUTLINE_PIXELS, Part, VIEW_WIDTH};

// Split into files (`cargo xtask lint` keeps files small).
mod arc;
mod back;
mod cuff;
mod extras;
mod glove;
mod look;
mod rings;
mod shape;

use arc::{Arc, add_arc, add_arc_bumped, ellipse};
use back::{add_back, add_lames};
use cuff::add_cuff;
use extras::{add_extras, add_scales, add_tekko};
use glove::{add_glove, add_rivets};
use look::{flute, look_of, styled};
use rings::add_rings;
use shape::{Axes, Shape, across};

/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Tipped back a little, so the plates' sides show.
const TILT: f32 = 0.45;
/// Between the two hands in a picture.
const PICTURE_GAP: f32 = 12.0;
/// How far a cuff is out from the forearm's skin: past a vambrace's 20 mm.
const CUFF_CLEAR: f32 = 28.0;
/// Points round a cuff, and across a plate.
const ROUND: usize = 20;
const ACROSS: usize = 12;
/// How far each lame stands out over the next, so its edge shows.
const LAME_STEP: f32 = 1.4;

/// One hand of a pair, in place on its frame.
pub(in crate::item_art) fn build_hand(
    form: &HandsForm,
    frame: &Frame,
    side: Side,
    digits: u8,
) -> Model {
    let hand = Hand::of(frame, side, digits);
    let axes = Axes::of(&hand);
    let shape = Shape::of(&hand, &axes);
    let mut model = Model::default();
    let mitten = form.fingers == Fingers::Mitten;
    match form.construction {
        Construction::Padded => {
            add_glove(
                &mut model,
                &shape,
                &axes,
                false,
                4.0,
                Stuff::Main,
                Some(14.0),
            );
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
        }
        Construction::Mail => {
            add_glove(
                &mut model,
                &shape,
                &axes,
                mitten,
                2.5,
                Stuff::Main,
                Some(6.0),
            );
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
        }
        Construction::RingMail => {
            add_glove(&mut model, &shape, &axes, mitten, 2.0, Stuff::Backing, None);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Backing);
            add_rings(&mut model, &shape, &axes, mitten, 2.0);
        }
        Construction::Leather => {
            add_glove(&mut model, &shape, &axes, mitten, 2.0, Stuff::Main, None);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
            if form.knuckles == Knuckles::Spiked {
                add_rivets(&mut model, &shape, &axes, 2.5, Stuff::Rivet);
            }
        }
        Construction::Scale => {
            add_glove(&mut model, &shape, &axes, mitten, 1.5, Stuff::Backing, None);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Backing);
            add_scales(&mut model, &shape, &axes);
        }
        Construction::MailAndPlate if form.fingers == Fingers::Open => {
            add_tekko(&mut model, &shape, &axes);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Cloth);
        }
        Construction::MailAndPlate => {
            add_glove(
                &mut model,
                &shape,
                &axes,
                false,
                2.5,
                Stuff::Main,
                Some(6.0),
            );
            add_back(&mut model, form, &shape, &axes, Stuff::Main, 1.2);
            add_lames(&mut model, form, &shape, &axes, Stuff::Main, -4.0);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
        }
        Construction::PlatesInCloth => {
            add_glove(&mut model, &shape, &axes, mitten, 1.5, Stuff::Backing, None);
            add_back(&mut model, form, &shape, &axes, Stuff::Cloth, 1.7);
            add_rivets(&mut model, &shape, &axes, 7.0, Stuff::Rivet);
            add_lames(&mut model, form, &shape, &axes, Stuff::Main, 0.0);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Cloth);
        }
        _ => {
            if form.fingers != Fingers::Open {
                add_glove(&mut model, &shape, &axes, mitten, 1.5, Stuff::Backing, None);
            }
            add_back(&mut model, form, &shape, &axes, Stuff::Main, 1.75);
            add_lames(&mut model, form, &shape, &axes, Stuff::Main, 0.0);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
        }
    }
    add_extras(&mut model, form, &shape, &axes);
    model
}

/// Both hands of a pair, in place on this frame: for sets and a dressed character.
// Waiting for those: only the tests use it so far.
#[allow(dead_code)]
pub(in crate::item_art) fn build_pair(form: &HandsForm, frame: &Frame, digits: u8) -> Model {
    let mut model = Model::default();
    for side in Side::BOTH {
        model.join(build_hand(form, frame, side, digits));
    }
    model
}

/// Both hands built on an ordinary body, laid down for their picture: fingers to the
/// right, backs up, the right hand above the left, then fitted to the cells.
fn fitted(look: &HandsLook) -> Model {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    let mut picture = Model::default();
    for side in Side::BOTH {
        let mut hand = build_hand(&look.form, &frame, side, look.digits);
        let axes = Axes::of(&Hand::of(&frame, side, look.digits));
        // Down goes right, out comes towards the camera.
        let turn = Mat3::from_cols(axes.down, axes.out.cross(axes.down), axes.out).transpose();
        hand.map(|point| turn * (point - axes.wrist));
        let (low, high) = hand.bounds();
        // Thumbs point away from each other: the left one turns down.
        let shift = match side {
            Side::Right => PICTURE_GAP / 2.0 - low.y,
            Side::Left => -PICTURE_GAP / 2.0 - high.y,
        };
        hand.map(|point| point + Vec3::Y * shift);
        picture.join(hand);
    }
    let tilt = Quat::from_rotation_x(-TILT);
    picture.map(|point| tilt * point);
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

/// How each part looks, by what it is made of.
pub(in crate::item_art) fn material_of(
    stuff: Stuff,
    form: &HandsForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let main = || match form.material.category() {
        Category::Metal => metal_material(form.material, quality),
        Category::Leather => matte(colour_of(form.material), 0.75),
        Category::Textile => matte(colour_of(form.material), 0.95),
        _ => matte(colour_of(form.material), 0.4),
    };
    match stuff {
        Stuff::Main => main(),
        Stuff::Shade => darker(main()),
        Stuff::Backing => matte(colour_of(Material::Leather), 0.8),
        Stuff::Cloth => matte(colour_of(form.cloth.unwrap_or(Material::Linen)), 0.95),
        Stuff::Trim => metal_material(form.trim.unwrap_or(Material::Brass), quality),
        Stuff::Rivet => metal_material(
            match form.construction {
                Construction::Leather => Material::Brass,
                _ => Material::Iron,
            },
            quality,
        ),
        Stuff::Field | Stuff::Charge | Stuff::Bone => main(),
        Stuff::Etch => StandardMaterial {
            base_color: Color::srgb(0.14, 0.16, 0.24),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        },
    }
}

/// A pair of hand pieces, laid down to fill their picture.
pub(in crate::item_art) fn hands_parts(
    look: &HandsLook,
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
            material: materials.add(material_of(stuff, &look.form, quality)),
            transform: Transform::from_rotation(camera.rotation),
        })
        .collect()
}

#[cfg(test)]
mod tests;
