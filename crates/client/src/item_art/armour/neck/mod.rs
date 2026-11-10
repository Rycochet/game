//! Neck armour: padded, mail and kikko collars, gorgets, colletins and the nodowa,
//! bevors, aventails and the bishop's mantle. Each is built in millimetres in place on a
//! body frame (`build`), so a set and a dressed character use the same parts; an
//! aventail hangs from a helmet's edge when it has one (`build_under`). The picture
//! shows the piece alone, as worn.

mod around;
mod bevor;
mod colour;
mod curtain;
mod gorget;
mod hanging;

#[cfg(test)]
mod tests;

pub(in crate::item_art) use self::colour::material_of;
use std::f32::consts::TAU;

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::head::OFF_HEAD;
use shared::armour::neck::{NeckForm, NeckKind};
use shared::body::Frame;
use shared::creature::CreatureKind;
use shared::item::Quality;

pub(in crate::item_art) use self::around::Around;
use super::Model;
use crate::item_art::body::mesh_of;
use crate::item_art::{CAMERA_AT, CAMERA_UP, NeckLook, OUTLINE_PIXELS, Part, VIEW_WIDTH};

/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Turned to its right and seen a little from above, so its side and top show.
const TURN: f32 = 0.55;
const TILT: f32 = 0.35;

/// A neck piece built on this frame, as worn; an aventail hangs from a plain edge, as
/// if from a bascinet.
pub(in crate::item_art) fn build(form: &NeckForm, frame: &Frame) -> Model {
    let around = Around { frame };
    let rim = |angle: f32| plain_rim(&around, angle);
    build_with(form, &around, &rim)
}

/// The same, with an aventail hung from this helmet's edge: `HeadForm::rim`'s points,
/// from the front round to the left. For sets and the dressed character.
#[allow(dead_code)]
pub(in crate::item_art) fn build_under(form: &NeckForm, frame: &Frame, rim: &[[i32; 3]]) -> Model {
    let around = Around { frame };
    let rim = |angle: f32| on_rim(rim, angle);
    build_with(form, &around, &rim)
}

fn build_with(form: &NeckForm, around: &Around, rim: &dyn Fn(f32) -> Vec3) -> Model {
    let mut model = Model::default();
    match form.kind {
        NeckKind::Collar => match form.construction {
            Construction::Mail | Construction::RingMail => {
                hanging::add_standard(&mut model, form, around)
            }
            _ => hanging::add_soft_collar(&mut model, form, around),
        },
        NeckKind::Gorget => gorget::add_gorget(&mut model, form, around),
        NeckKind::Bevor => bevor::add_bevor(&mut model, form, around),
        NeckKind::Aventail => hanging::add_aventail(&mut model, form, around, rim),
        NeckKind::Mantle => hanging::add_standard(&mut model, form, around),
    }
    model
}

/// A bascinet's edge: under the ears at the sides, at the nape behind.
fn plain_rim(around: &Around, angle: f32) -> Vec3 {
    let head = around.head_height();
    let (side, back) = (around.chin() + head * 0.3, around.chin() + head * 0.18);
    let behind = (-angle.cos()).max(0.0);
    let y = side + (back - side) * behind;
    around.head(angle, y, OFF_HEAD as f32)
}

/// A point on a helmet's edge at an angle, between its sixteen points.
fn on_rim(rim: &[[i32; 3]], angle: f32) -> Vec3 {
    let point = |at: usize| {
        let [x, y, z] = rim[at % rim.len()];
        Vec3::new(x as f32, y as f32, z as f32)
    };
    let along = angle.rem_euclid(TAU) / TAU * rim.len() as f32;
    let at = along.floor() as usize;
    point(at).lerp(point(at + 1), along.fract())
}

/// The piece built on an ordinary body for its picture, turned and tipped, then fitted
/// to its cells.
fn fitted(look: &NeckLook) -> Model {
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

/// A neck piece, as worn, filling its picture.
pub(in crate::item_art) fn neck_parts(
    look: &NeckLook,
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
