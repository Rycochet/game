//! Leg armour: cuisses, poleyns, greaves, schynbalds, leg harness, chausses, suneate,
//! haidate, Classical greaves and ankle guards, and leg guards of every construction.
//! Each leg is built in millimetres in place on a body frame (`build_leg`), so a set
//! and a dressed character use the same parts; the picture shows the piece alone.
//! `rigid::add_knee` is the poleyn other locations may build on (agreed with the
//! waist's lobster tassets).

mod colour;
mod japan;
mod path;
mod rigid;
mod shapes;
mod soft;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::legs::LegsForm;
use shared::body::{Frame, Side};
use shared::creature::CreatureKind;
use shared::item::Quality;

use self::path::Leg;
use super::Model;
use crate::item_art::body::mesh_of;
use crate::item_art::{CAMERA_AT, CAMERA_UP, LegsLook, OUTLINE_PIXELS, Part, VIEW_WIDTH};

/// How far rigid pieces stand off the skin: arming hose under them. Waist's hanging
/// tassets stand 35 mm off the thigh, over them (agreed with the waist's pieces).
const RIGID_LIFT: f32 = 15.0;
/// How far chausses of mail, scales and cloth stand off it.
const SOFT_LIFT: f32 = 7.0;
/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Turned a little further to its right than the bare body, so knee wings and the
/// open backs of cuisses show, and seen from a little above.
const TURN: f32 = 0.45;
const TILT: f32 = 0.25;

/// A leg piece on one leg of this frame, as worn.
pub(in crate::item_art) fn build_leg(form: &LegsForm, frame: &Frame, side: Side) -> Model {
    let mut model = Model::default();
    let leg = Leg { frame, side };
    match form.construction {
        _ if form.japanese() => japan::add_japanese(&mut model, form, &leg),
        Construction::Plate | Construction::Leather => rigid::add_rigid(&mut model, form, &leg),
        _ => soft::add_soft(&mut model, form, &leg),
    }
    model
}

/// A leg piece on the legs it is for, as worn: for sets and the dressed character.
pub(in crate::item_art) fn build(form: &LegsForm, frame: &Frame) -> Model {
    let mut model = Model::default();
    for &side in form.sides.sides() {
        model.join(build_leg(form, frame, side));
    }
    model
}

/// The piece built on an ordinary body for its picture, turned a little and tipped
/// back, then fitted to its cells.
fn fitted(look: &LegsLook) -> Model {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    let mut picture = build(&look.form, &frame);
    let view = Quat::from_rotation_x(TILT) * Quat::from_rotation_y(TURN);
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

/// A leg piece, as worn, filling its picture.
pub(in crate::item_art) fn legs_parts(
    look: &LegsLook,
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
