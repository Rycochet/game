//! Foot armour: sabatons in every toe shape (armoured boots when they rise over the
//! ankle), scale and splinted sabatons, mail shoes and kusari tabi, kogake, leather
//! foot guards and the Greek bronze feet. Each foot is built in millimetres in place on
//! a body frame (`build_foot`), over the shoe, so a set and a dressed character use the
//! same parts; the picture shows the pair alone. `Last` is the shape every piece is
//! built over.

mod colour;
mod guards;
mod last;
mod mail;
mod parts;
mod sabaton;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use shared::armour::Style;
use shared::armour::feet::{FeetForm, FeetPiece, Toe};
use shared::body::{Frame, Side};
use shared::creature::CreatureKind;
use shared::item::Quality;

pub(in crate::item_art) use self::last::{Foot, Last};
use super::Model;
use crate::item_art::body::mesh_of;
use crate::item_art::{CAMERA_AT, CAMERA_UP, FeetLook, OUTLINE_PIXELS, Part, VIEW_WIDTH};

/// How far rigid pieces stand off the bare foot: the shoe under them, so their ankle
/// plate is `ankle_radius` + 10 (agreed with the legs' greaves, which go over it).
const RIGID_LIFT: f32 = 10.0;
/// Mail over the shoe.
const MAIL_LIFT: f32 = 8.0;
/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Turned to its right and seen well from above, so the tops of the feet show.
const TURN: f32 = 0.6;
const TILT: f32 = 0.55;

/// The shape a piece is built over on this foot.
fn last_of<'a>(form: &FeetForm, foot: &'a Foot<'a>) -> Last<'a> {
    let soft = matches!(form.piece, FeetPiece::MailShoes);
    Last {
        foot,
        lift: if soft { MAIL_LIFT } else { RIGID_LIFT },
        toe: form.toe,
        point: form.point as f32,
        width: form.width as f32,
        up: if form.style == Style::Elven && form.toe == Toe::Pointed {
            25.0
        } else {
            0.0
        },
    }
}

/// A foot piece on one foot of this frame, as worn.
pub(in crate::item_art) fn build_foot(form: &FeetForm, frame: &Frame, side: Side) -> Model {
    let mut model = Model::default();
    let foot = Foot { frame, side };
    let last = last_of(form, &foot);
    match form.piece {
        FeetPiece::Sabatons => sabaton::add_sabaton(&mut model, form, &last, &foot),
        FeetPiece::MailShoes => mail::add_mail_shoe(&mut model, form, &last, &foot),
        FeetPiece::FootGuards => guards::add_guard(&mut model, form, &last, &foot),
    }
    model
}

/// A pair on both feet, as worn: for sets and the dressed character.
pub(in crate::item_art) fn build(form: &FeetForm, frame: &Frame) -> Model {
    let mut model = Model::default();
    for side in Side::BOTH {
        model.join(build_foot(form, frame, side));
    }
    model
}

/// The pair built on an ordinary body for its picture, turned and tipped back, then
/// fitted to its cells.
fn fitted(look: &FeetLook) -> Model {
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

/// A pair of foot pieces, as worn, filling its picture.
pub(in crate::item_art) fn feet_parts(
    look: &FeetLook,
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
