//! Arm armour: bracers, vambraces, couters, rerebraces, arm harness, manicae, kote,
//! bazubands and sleeves of every construction. Each arm is built in millimetres in
//! place on a body frame (`build_arm`), so a set and a dressed character use the same
//! parts; the picture shows the piece alone, seen from the front with the elbows out
//! and the forearms turned in towards each other.

mod colour;
mod path;
mod rigid;
mod shapes;
mod soft;

#[cfg(test)]
mod tests;

pub(in crate::item_art) use self::colour::material_of;
use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::arms::ArmsForm;
use shared::body::{Frame, Joint, Side};
use shared::creature::CreatureKind;
use shared::item::Quality;

use self::path::Arm;
use super::Model;
use crate::item_art::body::mesh_of;
use crate::item_art::{ArmsLook, CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, VIEW_WIDTH};

/// How far rigid pieces stand off the skin: an arming sleeve under them. Gauntlet
/// cuffs go 8 mm further out over their lower ends (agreed with the hands' pieces).
const RIGID_LIFT: f32 = 18.0;
/// How far sleeves of mail, scales and cloth stand off it.
const SOFT_LIFT: f32 = 7.0;
/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Seen from a little above.
const TILT: f32 = 0.25;
/// How far apart the two forearms' inner ends are in a picture, in millimetres.
const PICTURE_GAP: f32 = 60.0;
/// How far a picture bends the arms forwards at the elbow, so they look like arms
/// (the owner's request): about 45 degrees.
const PICTURE_BEND: f32 = 0.8;

/// An arm piece on one arm of this frame, as worn: the arm straight, as the frame has
/// it, so a gauntlet at its wrist meets it.
pub(in crate::item_art) fn build_arm(form: &ArmsForm, frame: &Frame, side: Side) -> Model {
    bent_arm(form, frame, side, 0.0)
}

/// The same, with the forearm swung forwards by `bend` radians at the elbow.
fn bent_arm(form: &ArmsForm, frame: &Frame, side: Side, bend: f32) -> Model {
    let mut model = Model::default();
    let arm = Arm { frame, side, bend };
    match form.construction {
        _ if form.kote() => soft::add_kote(&mut model, form, &arm),
        Construction::Plate | Construction::Leather => rigid::add_rigid(&mut model, form, &arm),
        Construction::Splint => soft::add_splints(&mut model, form, &arm),
        Construction::Laminar => soft::add_hoops(&mut model, form, &arm),
        _ => soft::add_sleeve(&mut model, form, &arm),
    }
    model
}

/// An arm piece on the arms it is for, as worn: for sets and the dressed character.
#[allow(dead_code)]
pub(in crate::item_art) fn build(form: &ArmsForm, frame: &Frame) -> Model {
    let mut model = Model::default();
    for &side in form.sides.sides() {
        model.join(build_arm(form, frame, side));
    }
    model
}

/// The piece built on an ordinary body for its picture, the arms bent at the elbow,
/// each turned a quarter about its shoulder so the elbows point out and the forearms
/// in, their outsides to the front (owner's choice), moved in towards each other and
/// tipped back, then fitted to its cells.
fn fitted(look: &ArmsLook) -> Model {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    let mut picture = Model::default();
    for &side in look.form.sides.sides() {
        let mut arm = bent_arm(&look.form, &frame, side, PICTURE_BEND);
        let [x, y, z] = frame.joint(Joint::Shoulder(side));
        let shoulder = Vec3::new(x as f32, y as f32, z as f32);
        // Forwards becomes inwards: the outside of each arm is +X times its sign.
        let sign = side.sign() as f32;
        let turn = Quat::from_rotation_y(-sign * FRAC_PI_2);
        arm.map(|point| shoulder + turn * (point - shoulder));
        let (low, high) = arm.bounds();
        let inner = if sign > 0.0 { low.x } else { high.x };
        arm.map(|point| point + Vec3::X * (sign * PICTURE_GAP / 2.0 - inner));
        picture.join(arm);
    }
    let view = Quat::from_rotation_x(TILT);
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

/// An arm piece, as worn, filling its picture.
pub(in crate::item_art) fn arms_parts(
    look: &ArmsLook,
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
