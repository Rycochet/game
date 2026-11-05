//! Placing a body in its picture: a person of the usual height fills it, anything
//! shorter is drawn that much smaller on the same ground.

use bevy::prelude::*;
use shared::body::{Frame, NONE, PERSON, Parts};

use super::{
    CELLS, MARGIN_PIXELS, PIXELS_PER_CELL, TILT, TURN, Triangle, VIEW_WIDTH, build, features,
};

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
pub(super) fn fitted(frame: &Frame, parts: &Parts, height: i32) -> (Vec<Triangle>, Vec<Triangle>) {
    let (skin, feathers) = (build(frame, parts), features::wings(frame, parts));
    let view = Quat::from_rotation_x(TILT) * Quat::from_rotation_y(TURN);
    let all: Vec<Triangle> = skin.iter().chain(&feathers).copied().collect();
    let (low, high) = bounds(&all, view);
    let (wide, long) = (CELLS.0 as f32, CELLS.1 as f32);
    let world_per_pixel = VIEW_WIDTH / (wide.max(long) * PIXELS_PER_CELL);
    let room = |cells: f32| (cells * PIXELS_PER_CELL - 2.0 * MARGIN_PIXELS) * world_per_pixel;
    let fit = |(low, high): (Vec3, Vec3)| {
        let size = high - low;
        (room(long) / size.y).min(room(wide) / size.x)
    };
    let person = fit(bounds(&build(&PERSON.frame(), &NONE), view));
    let shorter = height.min(AVERAGE_HEIGHT) as f32 / AVERAGE_HEIGHT as f32;
    let scale = (person * shorter).min(fit((low, high)));
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
