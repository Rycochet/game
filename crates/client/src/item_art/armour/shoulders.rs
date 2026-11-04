//! Shoulder pieces: pauldrons, spaulders, hanging guards (sode and their eastern
//! cousins), doublings and the linothorax yoke, shoulder discs, ailettes and the Chukchi
//! wing. Each pair is built in millimetres in place on a body frame, lying outside the
//! breastplate and gorget (agreed with the torso's and neck's pieces): rigid pieces
//! 35 mm out from the skin, hanging ones 28 mm out from the arm. Lames end about 130 mm
//! down the arm, where a rerebrace tucks under them. The picture shows the pair as worn,
//! seen as a character is.

use std::f32::consts::PI;

use bevy::prelude::*;
use shared::armour::shoulders::{CapeCut, Cut, ShouldersForm, ShouldersPiece};
use shared::body::{Frame, Side};
use shared::creature::CreatureKind;
use shared::item::Quality;

use super::surface::{Surface, grid};
use super::{Model, Stuff, plate};
use crate::item_art::body::mesh_of;
use crate::item_art::{CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, ShouldersLook, VIEW_WIDTH};

// Split into files (`cargo xtask lint` keeps files small).
mod adornment;
mod arm;
mod cape;
mod colour;
mod disc;
mod guards;
mod look;
mod pauldron;
mod pauldron_extras;
mod surfaces;

use adornment::add_adornment;
use arm::Arm;
use cape::{add_cape, add_yoke};
pub(in crate::item_art) use colour::material_of;
use disc::{add_ailette, add_shoulder_disc, add_wing, disc_place};
use guards::add_guards;
use look::{Look, edge_drop, flute, look_of, shaped};
use pauldron::{add_cop_and_lames, add_disc};
use pauldron_extras::{add_hanging_pauldron, add_pauldron_extras, add_pteruges};
use surfaces::{arm_surface, cop_surface, guard_surface, shoulder_top, wing_surface};

/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Seen as the bare body is: turned a little to its right and from a little above.
const TURN: f32 = 0.35;
const TILT: f32 = 0.25;
/// How far each piece of a pair is turned to show its outside, and the gap between them.
const SHOW_TURN: f32 = 0.3;
const PICTURE_GAP: f32 = 16.0;
/// How far rigid pieces stand out from the skin, and the wings from the trunk's: past
/// a breastplate and a gorget.
const RIGID_CLEAR: f32 = 35.0;
const WING_CLEAR: f32 = 45.0;
/// How far hanging pieces are out from the arm's skin: past a rerebrace.
const HANGING_CLEAR: f32 = 28.0;
/// How far a cape or yoke lies off the skin, over a gambeson.
const DRAPE_CLEAR: f32 = 22.0;
/// Half the gap at the front of a cape, as an angle round it.
const OPENING: f32 = 0.14;
/// Points round and along each plate.
const ACROSS: usize = 16;
const ALONG: usize = 8;

/// A plate over a whole surface.
fn sheet(model: &mut Model, stuff: Stuff, surface: Surface, thick: f32) {
    plate(model.of(stuff), &grid(surface, ALONG, ACROSS), thick, false);
}

/// One shoulder's piece, in place on this frame.
fn build_side(form: &ShouldersForm, frame: &Frame, side: Side) -> Model {
    let mut model = Model::default();
    let arm = Arm::of(frame, side);
    let look = shaped(form);
    let scale = frame.elbow_length as f32 / 300.0;
    let mut height = 0.0;
    match form.piece {
        ShouldersPiece::Pauldrons {
            cut,
            wings,
            lames,
            upper_lames,
            haute_piece,
            gardbrace,
            besagews,
        } => {
            if lames == 0 {
                add_hanging_pauldron(&mut model, form, &arm, look);
            } else {
                // The right of a lance cut is cut away in front, under the arm.
                let range = if cut == Cut::LanceCut && side == Side::Right {
                    (-2.2, 0.7)
                } else {
                    (-2.2, 2.2)
                };
                add_cop_and_lames(&mut model, form, frame, &arm, look, range, lames, besagews);
                add_pauldron_extras(
                    &mut model,
                    frame,
                    &arm,
                    look,
                    wings,
                    cut,
                    upper_lames,
                    haute_piece,
                    gardbrace,
                );
                if gardbrace {
                    height = 9.0;
                }
            }
        }
        ShouldersPiece::Spaulders {
            lames,
            besagews,
            pteruges,
        } => {
            if form.dendra() {
                // One big plate over the shoulder, and one over the upper arm.
                let across = arm.radius + RIGID_CLEAR + 8.0;
                let cop = cop_surface(&arm, look, (-2.1, 2.1), (across, across - 6.0), PI / 2.0);
                sheet(&mut model, Stuff::Main, &cop, 2.4);
                let band = arm_surface(
                    &arm,
                    look,
                    (-2.0, 2.0),
                    (-10.0, arm.reach * 0.9),
                    across - 3.0,
                );
                sheet(&mut model, Stuff::Main, &band, 2.4);
            } else {
                add_cop_and_lames(
                    &mut model,
                    form,
                    frame,
                    &arm,
                    look,
                    (-1.85, 1.85),
                    lames,
                    besagews,
                );
            }
            if pteruges > 0 {
                add_pteruges(&mut model, &arm, pteruges, arm.radius + RIGID_CLEAR);
            }
        }
        ShouldersPiece::Guards { .. } => {
            add_guards(&mut model, form, frame, &arm);
            height = HANGING_CLEAR - RIGID_CLEAR;
        }
        ShouldersPiece::Discs { diameter, boss } => {
            add_shoulder_disc(&mut model, form, &arm, diameter as f32 * scale, boss);
            height = -6.0;
        }
        ShouldersPiece::Ailettes {
            shape,
            size,
            heraldry,
        } => {
            let (wide, high) = size.ailette();
            let size = (wide as f32 * scale, high as f32 * scale);
            add_ailette(&mut model, form, frame, &arm, shape, size, heraldry.charge);
        }
        // Both shoulders at once.
        ShouldersPiece::Cape { .. } | ShouldersPiece::Wing => {}
    }
    add_adornment(&mut model, form, &arm, height);
    model
}

/// Both shoulders' pieces, in place on this frame: for sets and a dressed character.
pub(in crate::item_art) fn build_pair(form: &ShouldersForm, frame: &Frame) -> Model {
    let mut model = Model::default();
    match form.piece {
        ShouldersPiece::Cape { cut, length } => match cut {
            CapeCut::Round => add_cape(&mut model, form, frame, length),
            CapeCut::Yoke => add_yoke(&mut model, form, frame, length),
        },
        ShouldersPiece::Wing => add_wing(&mut model, frame),
        _ => {
            for side in Side::BOTH {
                model.join(build_side(form, frame, side));
            }
        }
    }
    model
}

/// The pair built on an ordinary body for its picture: each piece turned to show its
/// outside and set beside the other, the right on the left as worn, tipped back a
/// little, then fitted to its cells. Capes and the wing are seen as worn.
fn fitted(look: &ShouldersLook) -> Model {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    let mut picture = match look.form.piece {
        ShouldersPiece::Cape { .. } | ShouldersPiece::Wing => {
            let mut model = build_pair(&look.form, &frame);
            // The wing faces out to the left: turned to face the camera.
            let turn = Quat::from_rotation_y(if look.form.piece == ShouldersPiece::Wing {
                -1.25
            } else {
                TURN
            });
            model.map(|point| turn * point);
            model
        }
        _ => {
            let mut picture = Model::default();
            for side in Side::BOTH {
                let mut piece = build_side(&look.form, &frame, side);
                let sign = side.sign() as f32;
                let joint = Arm::of(&frame, side).joint;
                let turn = Quat::from_rotation_y(-sign * SHOW_TURN);
                piece.map(|point| turn * (point - joint));
                let (low, high) = piece.bounds();
                // Edge to edge, a little apart.
                let shift = if sign > 0.0 {
                    PICTURE_GAP / 2.0 - low.x
                } else {
                    -PICTURE_GAP / 2.0 - high.x
                };
                piece.map(|point| point + Vec3::X * shift);
                picture.join(piece);
            }
            picture
        }
    };
    let tilt = Quat::from_rotation_x(TILT);
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

/// A pair of shoulder pieces, as worn, filling their picture.
pub(in crate::item_art) fn shoulders_parts(
    look: &ShouldersLook,
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
