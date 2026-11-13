//! Placing a body in its picture: a person of the usual height fills it, anything
//! shorter is drawn that much smaller on the same ground; an animal fills its own.

use bevy::prelude::*;
use shared::body::{Hair, Layout, NONE, PERSON};
use shared::sheet::Marks;

use super::{
    Cover, MARGIN_PIXELS, PIXELS_PER_CELL, TILT, TURN, Triangle, VIEW_WIDTH, build, build_coated,
    cells, coat_hair, features, hair, marks,
};

/// How far an animal or a taur is turned to its right: nearly side on.
const BEAST_TURN: f32 = 1.15;

/// How far a body is turned to its right in a picture: a little, so an upright body's
/// sides show, or nearly side on for anything long.
pub(in crate::item_art) fn turn_of(layout: &Layout) -> f32 {
    if layout.frame().is_some() {
        TURN
    } else {
        BEAST_TURN
    }
}

/// How tall a person usually stands, in millimetres: the height that fills a picture.
const AVERAGE_HEIGHT: i32 = 1750;

/// The lowest and highest corners of these triangles as they are seen.
fn bounds(triangles: &[Triangle], view: Quat) -> (Vec3, Vec3) {
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for corner in triangles.iter().flatten() {
        let corner = view * *corner;
        low = low.min(corner);
        high = high.max(corner);
    }
    (low, high)
}

/// The body turned and tipped as it is seen, then placed in its picture with room for
/// its black edge: a person of the usual height fills it, anything shorter (`height`, in
/// millimetres) is drawn that much smaller on the same ground, and anything that would
/// still overflow it (wide horns, say) is fitted to it. The skin, its markings, a
/// taur's coat, any feathers, any hair (on the upper body's head, for a taur), then any
/// mane and tail hair.
pub(super) fn fitted(
    layout: &Layout,
    height: i32,
    styled: Hair,
    marked: Option<Marks>,
) -> [Vec<Triangle>; 6] {
    let ((skin, coat), feathers) = (build_coated(layout), features::wings(layout));
    let (skin, marked) = marks::split(layout, skin, marked);
    let head = layout.upper_body().unwrap_or(*layout);
    let hair = hair(&head, styled, Cover::default());
    let mane = coat_hair(layout);
    let view = Quat::from_rotation_x(TILT) * Quat::from_rotation_y(turn_of(layout));
    let all: Vec<Triangle> = [&skin, &marked, &coat, &feathers, &hair, &mane]
        .into_iter()
        .flatten()
        .copied()
        .collect();
    let (low, high) = bounds(&all, view);
    let (wide, long) = cells(layout);
    let (wide, long) = (wide as f32, long as f32);
    let world_per_pixel = VIEW_WIDTH / (wide.max(long) * PIXELS_PER_CELL);
    let room = |cells: f32| (cells * PIXELS_PER_CELL - 2.0 * MARGIN_PIXELS) * world_per_pixel;
    let fit = |(low, high): (Vec3, Vec3)| {
        let size = high - low;
        (room(long) / size.y).min(room(wide) / size.x)
    };
    let scale = if layout.frame().is_some() {
        let person = fit(bounds(&build(&Layout::biped(&PERSON.frame(), NONE)), view));
        let shorter = height.min(AVERAGE_HEIGHT) as f32 / AVERAGE_HEIGHT as f32;
        (person * shorter).min(fit((low, high)))
    } else {
        fit((low, high))
    };
    let middle = (low + high) / 2.0;
    let ground = -room(long) / 2.0;
    let placed = |triangles: Vec<Triangle>| -> Vec<Triangle> {
        triangles
            .into_iter()
            .map(|triangle| {
                triangle.map(|corner| {
                    let seen = view * corner;
                    Vec3::new(
                        (seen.x - middle.x) * scale,
                        (seen.y - low.y) * scale + ground,
                        (seen.z - middle.z) * scale,
                    )
                })
            })
            .collect()
    };
    [skin, marked, coat, feathers, hair, mane].map(placed)
}
