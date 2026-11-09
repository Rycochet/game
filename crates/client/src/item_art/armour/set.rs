//! An armour set's model: each of its pieces built in place on one body frame, as its
//! own location builds it, each coloured as that location colours it.

use bevy::prelude::*;
use shared::armour::set::SetForm;
use shared::body::Frame;
use shared::creature::CreatureKind;
use shared::item::Quality;

use super::{Model, arms, feet, hands, head, legs, neck, shoulders, torso, waist};
use crate::item_art::body::{Triangle, mesh_of};
use crate::item_art::{CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, SetLook, VIEW_WIDTH};

/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Seen as the bare body is: turned a little to its right and from a little above.
const TURN: f32 = 0.35;
const TILT: f32 = 0.25;

/// Triangles with what they are made of.
pub(in crate::item_art) type Coloured = Vec<(Vec<Triangle>, StandardMaterial)>;

/// Every piece of a set built on this frame, as worn, each part with its material.
/// `digits` is how many digits each hand has, for its gauntlets.
pub(in crate::item_art) fn build(
    form: &SetForm,
    frame: &Frame,
    digits: u8,
    quality: Option<Quality>,
) -> Coloured {
    let mut all = Coloured::new();
    let mut add = |model: Model, colour: &dyn Fn(super::Stuff) -> StandardMaterial| {
        for (stuff, triangles) in model.parts {
            all.push((triangles, colour(stuff)));
        }
    };
    let torso = &form.torso;
    add(torso::build(torso, frame), &|s| {
        torso::material_of(s, torso, quality)
    });
    if let Some(piece) = &form.waist {
        add(waist::build(piece, frame), &|s| {
            waist::material_of(s, piece, quality)
        });
    }
    if let Some(piece) = &form.legs {
        add(legs::build(piece, frame), &|s| {
            legs::material_of(s, piece, quality)
        });
    }
    if let Some(piece) = &form.feet {
        add(feet::build(piece, frame), &|s| {
            feet::material_of(s, piece, quality)
        });
    }
    if let Some(piece) = &form.arms {
        add(arms::build(piece, frame), &|s| {
            arms::material_of(s, piece, quality)
        });
    }
    if let Some(piece) = &form.hands {
        let model = hands::build_pair(piece, frame, digits);
        add(model, &|s| hands::material_of(s, piece, quality));
    }
    if let Some(piece) = &form.shoulders {
        let model = shoulders::build_pair(piece, frame);
        add(model, &|s| shoulders::material_of(s, piece, quality));
    }
    if let Some(piece) = &form.head {
        add(head::build(piece, frame), &|s| {
            head::material_of(s, piece, quality)
        });
    }
    if let Some(piece) = &form.neck {
        // An aventail hangs from the set's own helmet.
        let model = match &form.head {
            Some(helmet) => neck::build_under(piece, frame, &helmet.rim(frame)),
            None => neck::build(piece, frame),
        };
        add(model, &|s| neck::material_of(s, piece, quality));
    }
    all
}

/// A set as worn on an ordinary body, turned and fitted to its cells, filling its
/// picture.
pub(in crate::item_art) fn set_parts(
    look: &SetLook,
    quality: Option<Quality>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    let pieces = build(&look.form, &frame, look.digits, quality);
    fit(pieces, look.cells, TURN, meshes, materials)
}

/// Parts built on a frame, turned as a body is seen and fitted to these cells.
pub(in crate::item_art) fn fit(
    pieces: Coloured,
    cells: (u8, u8),
    turned: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    fit_standing(pieces, cells, turned, None, meshes, materials)
}

/// As `fit`, or with `scale` (the share of the picture's height a millimetre
/// takes), drawn at that scale standing on the picture's floor instead, unless
/// it would overflow: every character to the same scale.
pub(in crate::item_art) fn fit_standing(
    mut pieces: Coloured,
    cells: (u8, u8),
    turned: f32,
    scale: Option<f32>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let view = Quat::from_rotation_x(TILT) * Quat::from_rotation_y(turned);
    let turn = |triangles: &mut Vec<Triangle>, change: &dyn Fn(Vec3) -> Vec3| {
        for triangle in triangles.iter_mut() {
            *triangle = triangle.map(change);
        }
    };
    for (triangles, _) in &mut pieces {
        turn(triangles, &|point| view * point);
    }
    let corners = pieces.iter().flat_map(|(t, _)| t.iter().flatten());
    let (low, high) = corners.fold(
        (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
        |(low, high), &corner| (low.min(corner), high.max(corner)),
    );
    let (wide, long) = (cells.0 as f32, cells.1 as f32);
    let world_per_pixel = VIEW_WIDTH / (wide.max(long) * PIXELS_PER_CELL);
    let room = |cells: f32| (cells * PIXELS_PER_CELL - 2.0 * MARGIN_PIXELS) * world_per_pixel;
    let size = high - low;
    let filling = (room(long) / size.y).min(room(wide) / size.x);
    let mut middle = (low + high) / 2.0;
    let scale = match scale {
        Some(share) => {
            let scale = (share * room(long)).min(filling);
            // Feet on the floor: the middle moved up so the lowest point is there.
            middle.y = low.y + room(long) / scale / 2.0;
            scale
        }
        None => filling,
    };
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    pieces
        .into_iter()
        .filter(|(triangles, _)| !triangles.is_empty())
        .map(|(mut triangles, material)| {
            turn(&mut triangles, &|point| (point - middle) * scale);
            Part {
                mesh: meshes.add(mesh_of(&triangles)),
                material: materials.add(material),
                transform: Transform::from_rotation(camera.rotation),
            }
        })
        .collect()
}
