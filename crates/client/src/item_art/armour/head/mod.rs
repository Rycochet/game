//! Head armour: helmets of every kind, coifs and arming caps. Each is built in
//! millimetres in place on a body frame (`build`), so a set and a dressed character use
//! the same parts; the picture shows it alone, as worn, turned to show its side.

mod back;
mod bowl;
mod colour;
mod face;
mod shell;
mod soft;
mod top;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use shared::armour::head::{HeadForm, HeadKind, OFF_HEAD};
use shared::body::Frame;
use shared::creature::CreatureKind;
use shared::item::Quality;

use self::shell::Shell;
use super::surface::grid;
use super::{Model, Stuff, plate};
use crate::item_art::body::mesh_of;
use crate::item_art::{CAMERA_AT, CAMERA_UP, HeadLook, OUTLINE_PIXELS, Part, VIEW_WIDTH};

/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Turned further than a body is, so a helmet's side and back show.
const TURN: f32 = 0.6;
const TILT: f32 = 0.25;
/// How thick plate is drawn: thin, as only its edges show.
const THICK: f32 = 2.0;

/// A head piece built on this frame, as worn.
pub(in crate::item_art) fn build(form: &HeadForm, frame: &Frame) -> Model {
    let mut model = Model::default();
    match form.kind {
        HeadKind::Cap => soft::add_cap(&mut model, form, frame),
        HeadKind::Coif => soft::add_coif(&mut model, form, frame),
        HeadKind::Helmet | HeadKind::BevoredHelm => {
            let shell = Shell::new(form, frame, OFF_HEAD as f32);
            bowl::add_bowl(&mut model, form, &shell);
            face::add_face(&mut model, form, &shell);
            back::add_back(&mut model, form, &shell);
            back::add_brim(&mut model, form, &shell);
            top::add_top(&mut model, form, &shell);
            top::add_ornament(&mut model, form, &shell);
        }
    }
    model
}

/// A plate over part of the shell, between two angles and two heights, standing out by
/// `lift` (which may change across it: `u` round, `v` up).
#[allow(clippy::too_many_arguments)]
fn sheet(
    model: &mut Model,
    stuff: Stuff,
    shell: &Shell,
    (from, to): (f32, f32),
    (low, high): (f32, f32),
    lift: &dyn Fn(f32, f32) -> f32,
    thick: f32,
    (rows, columns): (usize, usize),
) {
    let surface = |u: f32, v: f32| {
        let angle = from + (to - from) * u;
        let (point, out) = shell.point(angle, low + (high - low) * v);
        (point + out * lift(u, v), out)
    };
    plate(
        model.of(stuff),
        &grid(&surface, rows, columns),
        thick,
        false,
    );
}

/// The piece built on an ordinary body for its picture, turned and tipped back, then
/// fitted to its cells.
fn fitted(look: &HeadLook) -> Model {
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

/// A head piece, as worn, filling its picture.
pub(in crate::item_art) fn head_parts(
    look: &HeadLook,
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
