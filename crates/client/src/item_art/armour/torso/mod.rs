//! Torso armour: cuirasses and breastplates, leather, mail, ring mail, padding and the
//! arming vest. Each is built in millimetres in place on a body frame (`build`), so a
//! set and a dressed character use the same parts; the picture shows it alone, as worn,
//! seen as a character is.

mod colour;
mod plate;
mod shell;
mod soft;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use shared::armour::torso::{TorsoBody, TorsoForm};
use shared::body::Frame;
use shared::creature::CreatureKind;
use shared::item::Quality;

use self::shell::{Shell, Span};
use super::surface::studs;
use super::{Model, Stuff};
use crate::item_art::body::mesh_of;
use crate::item_art::{CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, TorsoLook, VIEW_WIDTH};

/// How far a rigid shell stands out from the skin: clothes and padding (the shared
/// 15 mm) and 10 mm more, so faulds tuck under its hem (agreed with the waist's pieces).
const RIGID_OUT: f32 = 25.0;
/// Rows up a shell.
const ROWS: usize = 12;
/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Seen as the bare body is: turned a little to its right and from a little above.
const TURN: f32 = 0.35;
const TILT: f32 = 0.25;

/// A torso piece built on this frame, as worn.
pub(in crate::item_art) fn build(form: &TorsoForm, frame: &Frame) -> Model {
    let mut model = Model::default();
    if form.over_mail {
        soft::add_mail(&mut model, form, frame, soft::MAIL_OUT);
    }
    match form.body {
        TorsoBody::Plate { .. } => plate::add_shell(&mut model, form, frame, 2.5),
        TorsoBody::Leather { studded, .. } if form.rigid() => {
            plate::add_shell(&mut model, form, frame, 5.0);
            if studded {
                add_studs(&mut model, frame, form.back);
            }
        }
        TorsoBody::Leather { .. } => soft::add_buff(&mut model, frame),
        TorsoBody::Mail { .. } => soft::add_mail(&mut model, form, frame, soft::MAIL_OUT),
        TorsoBody::RingMail { .. } => soft::add_ring_mail(&mut model, form, frame),
        TorsoBody::Padded { .. } => soft::add_padded(&mut model, form, frame),
    }
    if form.lined {
        // The lining shows below the hem.
        let reach = |_: f32| 0.1;
        let lining = Shell {
            frame,
            span: Span::Round,
            out: RIGID_OUT - 8.0,
            breast: None,
            hem: 8.0,
            reach: Some(&reach),
        };
        plate::sheet(&mut model, Stuff::Cloth, &lining, 6.0);
    }
    model
}

/// Rows of rivet heads over a studded leather cuirass.
fn add_studs(model: &mut Model, frame: &Frame, back: bool) {
    let spans: &[Span] = if back {
        &[Span::Front, Span::Back]
    } else {
        &[Span::Front]
    };
    for &span in spans {
        let shell = Shell {
            frame,
            span,
            out: RIGID_OUT,
            breast: None,
            hem: 0.0,
            reach: None,
        };
        let surface = |u: f32, v: f32| shell.at(u, v);
        studs(model, Stuff::Rivet, &surface, (7, 11), 5.0, 4.0);
    }
}

/// The piece built on an ordinary body for its picture, turned a little and tipped back,
/// then fitted to its cells.
fn fitted(look: &TorsoLook) -> Model {
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

/// A torso piece, as worn, filling its picture.
pub(in crate::item_art) fn torso_parts(
    look: &TorsoLook,
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
