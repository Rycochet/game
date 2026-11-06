//! Placing a body in its picture: a person of the usual height fills it, anything
//! shorter is drawn that much smaller on the same ground; an animal fills its own.

use bevy::prelude::*;
use shared::body::{Layout, NONE, PERSON};

use super::{
    MARGIN_PIXELS, PIXELS_PER_CELL, TILT, TURN, Triangle, VIEW_WIDTH, build, cells, features,
};

/// How far an animal is turned to its right: nearly side on.
const BEAST_TURN: f32 = 1.15;

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
/// still overflow it (wide horns, say) is fitted to it. The skin, then any feathers.
pub(super) fn fitted(layout: &Layout, height: i32) -> (Vec<Triangle>, Vec<Triangle>) {
    let (skin, feathers) = (build(layout), features::wings(layout));
    // An animal turned further, so its side shows.
    let turn = if layout.frame().is_some() {
        TURN
    } else {
        BEAST_TURN
    };
    let view = Quat::from_rotation_x(TILT) * Quat::from_rotation_y(turn);
    let all: Vec<Triangle> = skin.iter().chain(&feathers).copied().collect();
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
    (placed(skin), placed(feathers))
}
