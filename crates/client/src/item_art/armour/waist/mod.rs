//! Waist armour: faulds and culets, tassets (cuirassier tassets with the legs' knee
//! cops), tonlets and steel bases, mail, scale, lamellar and other skirts, kusazuri and
//! hip flaps, pteruges and the Roman apron, war girdles and the mitra, mail breeches.
//! Each is built in millimetres in place on a body frame (`build`), starting under the
//! torso piece's hem, so a set and a dressed character use the same parts; the picture
//! shows the piece alone.

mod colour;
mod cover;
mod hips;
mod hoops;
mod panels;
mod skirt;
mod tassets;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use shared::armour::STANDOFF_MM;
use shared::armour::waist::{WaistForm, WaistPiece};
use shared::body::{Frame, Side};
use shared::creature::CreatureKind;
use shared::item::Quality;

use self::hips::Hips;
use super::Model;
use crate::item_art::body::mesh_of;
use crate::item_art::{CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, VIEW_WIDTH, WaistLook};

/// Skin to the inside of any Outer armour: clothes and padding.
const STANDOFF: f32 = STANDOFF_MM as f32;
/// How far rigid pieces at the waist stand off, under the torso piece's hem.
const RIGID_OUT: f32 = STANDOFF;
/// How far mail, scale and cloth skirts stand off.
const SOFT_OUT: f32 = STANDOFF - 5.0;
/// How far hanging tassets stand off the thigh, over the legs' cuisses (agreed with
/// the legs' pieces).
const HANGING_OUT: f32 = 35.0;
/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Turned a little to its right and seen from a little above, so its sides show.
const TURN: f32 = 0.5;
const TILT: f32 = 0.3;

/// A waist piece on this frame, as worn: for sets and the dressed character.
pub(in crate::item_art) fn build(form: &WaistForm, frame: &Frame) -> Model {
    let mut model = Model::default();
    let hips = Hips { frame };
    let bottom = form.length.bottom(frame) as f32;
    hoops::add_hoops(&mut model, form, &hips);
    match form.piece {
        WaistPiece::Tassets => {
            for side in Side::BOTH {
                tassets::add_tassets(&mut model, form, frame, side);
            }
        }
        WaistPiece::Panels => panels::add_panels(&mut model, form, &hips, bottom),
        WaistPiece::Pteruges => panels::add_pteruges(&mut model, form, &hips, bottom),
        WaistPiece::Skirt => skirt::add_skirt(&mut model, form, &hips, bottom),
        WaistPiece::Breeches => skirt::add_breeches(&mut model, form, frame, &hips),
        WaistPiece::Girdle | WaistPiece::Fauld => {}
    }
    model
}

/// The piece built on an ordinary body for its picture, turned a little and tipped
/// back, then fitted to its cells.
fn fitted(look: &WaistLook) -> Model {
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

/// A waist piece, as worn, filling its picture.
pub(in crate::item_art) fn waist_parts(
    look: &WaistLook,
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
