//! Drawing one step of a body changing into another: the blended body, turned between
//! the two bodies' turns and drawn between their sizes, so the first and last steps
//! fill the token as the bodies themselves do.

use bevy::prelude::*;
use shared::body::{BLEND_WHOLE, Hair, Layout};

use super::super::colours::material;
use super::super::morph::{MorphLook, blend_colours};
use super::super::{CAMERA_AT, CAMERA_UP, Part, VIEW_WIDTH};
use super::{
    Cover, MARGIN_PIXELS, PIXELS_PER_CELL, TILT, Triangle, build_coated, coat_hair, hair, mesh_of,
    turn_of, wings,
};

/// A token's picture is two cells square.
const CELLS: f32 = 2.0;

pub(in crate::item_art) fn morph_parts(
    look: &MorphLook,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    placed(look)
        .into_iter()
        .map(|(triangles, colour)| Part {
            mesh: meshes.add(mesh_of(&triangles)),
            material: materials.add(material(colour)),
            transform: Transform::from_rotation(camera.rotation),
        })
        .collect()
}

/// The blended body's pieces in their colours, turned as seen and placed in the
/// picture.
pub(in crate::item_art) fn placed(look: &MorphLook) -> Vec<(Vec<Triangle>, [u8; 3])> {
    let t = look.along();
    let share = t as f32 / BLEND_WHOLE as f32;
    let layout = Layout::blend(&look.from, &look.to, t);
    let colours = blend_colours(&look.colours[0], &look.colours[1], t);
    let hair = look.hair[usize::from(t >= BLEND_WHOLE / 2)];
    // As big in the picture as the bodies at either end, never overflowing it.
    let (from_turn, to_turn) = (turn_of(&look.from), turn_of(&look.to));
    let from = filling(&pieces(&look.from, look.hair[0]), from_turn).0;
    let to = filling(&pieces(&look.to, look.hair[1]), to_turn).0;
    let turn = from_turn + (to_turn - from_turn) * share;
    let pieces = pieces(&layout, hair);
    let (fits, middle) = filling(&pieces, turn);
    let scale = (from + (to - from) * share).min(fits);
    let view = view(turn);
    let [skin, coat, feathers, hair, mane] = pieces;
    [
        (skin, colours.skin),
        (coat, colours.coat),
        (feathers, colours.feathers),
        (hair, colours.hair),
        (mane, colours.mane),
    ]
    .into_iter()
    .filter(|(triangles, _)| !triangles.is_empty())
    .map(|(triangles, colour)| {
        let placed = triangles
            .into_iter()
            .map(|triangle| triangle.map(|corner| (view * corner - middle) * scale))
            .collect();
        (placed, colour)
    })
    .collect()
}

fn view(turn: f32) -> Quat {
    Quat::from_rotation_x(TILT) * Quat::from_rotation_y(turn)
}

/// The skin, a taur's coat, feathers, hair and an animal's long hair. Hair goes on the
/// larger head: a taur's upper body's, unless it is still growing out of a body whose
/// own head is shrinking away.
fn pieces(layout: &Layout, styled: Hair) -> [Vec<Triangle>; 5] {
    let ((skin, coat), feathers) = (build_coated(layout), wings(layout));
    let upper = layout.upper_body();
    let head = match upper {
        Some(upper) if upper.head.length >= layout.head.length => upper,
        _ => *layout,
    };
    let hair = hair(&head, styled, Cover::default());
    [skin, coat, feathers, hair, coat_hair(layout)]
}

/// How much to scale these, seen at this turn, to fill a token's picture with room for
/// its black edge, and their middle as seen.
fn filling(pieces: &[Vec<Triangle>; 5], turn: f32) -> (f32, Vec3) {
    let view = view(turn);
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for corner in pieces.iter().flatten().flatten() {
        let seen = view * *corner;
        low = low.min(seen);
        high = high.max(seen);
    }
    let world_per_pixel = VIEW_WIDTH / (CELLS * PIXELS_PER_CELL);
    let room = (CELLS * PIXELS_PER_CELL - 2.0 * MARGIN_PIXELS) * world_per_pixel;
    let size = (high - low).max(Vec3::splat(1.0));
    (room / size.x.max(size.y), (low + high) / 2.0)
}
