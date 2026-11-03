//! The player's dot, the arrow to them from the edge, and the floor hint.

use super::super::canvas::{Canvas, polygon_distance};
use super::{BLACK, HINT_EDGE, HINT_GAP, HINT_HEAD_HEIGHT, HINT_HEAD_WIDTH, HINT_SHAFT, WHITE};

/// The player: a white circle with a thin black outline, `size` across in all.
pub fn player_dot(canvas: &mut Canvas, centre: (f32, f32), size: f32, outline: f32) {
    canvas.disc(centre, size / 2.0, BLACK);
    canvas.disc(centre, size / 2.0 - outline, WHITE);
}

/// A little arrow, head and shaft, `size` tall beside a mark `reach` across
/// each way, up or down: which way the player's floor is. On its right, or its
/// left if that is off the picture.
pub fn floor_hint(canvas: &mut Canvas, mark: (f32, f32), reach: f32, size: f32, up: bool) {
    let half_head = size * HINT_HEAD_WIDTH / 2.0;
    let side = reach + size * HINT_GAP + half_head;
    let goes_off = mark.0 + side + half_head > canvas.width as f32;
    let x = mark.0 + if goes_off { -side } else { side };
    let half_shaft = size * HINT_SHAFT / 2.0;
    // Worked out pointing up, then turned over for down.
    let at = |across: f32, from_tip: f32| {
        let down = from_tip - size / 2.0;
        (x + across, mark.1 + if up { down } else { -down })
    };
    let neck = size * HINT_HEAD_HEIGHT;
    let corners = [
        at(0.0, 0.0),
        at(half_head, neck),
        at(half_shaft, neck),
        at(half_shaft, size),
        at(-half_shaft, size),
        at(-half_shaft, neck),
        at(-half_head, neck),
    ];
    let centre = (x, mark.1);
    let edge = size * HINT_EDGE;
    canvas.shape(centre, size + edge, BLACK, |x, y| {
        polygon_distance(&corners, x, y) - edge
    });
    canvas.shape(centre, size, WHITE, |x, y| polygon_distance(&corners, x, y));
}

/// Where to put the arrow to a player off the map: on the edge of the box
/// `inset` inside the picture, towards `target` from the picture's middle.
/// `None` if the target is inside that box.
pub fn arrow_place(
    (width, height): (f32, f32),
    target: (f32, f32),
    inset: f32,
) -> Option<((f32, f32), (f32, f32))> {
    let (middle_x, middle_y) = (width / 2.0, height / 2.0);
    let (dx, dy) = (target.0 - middle_x, target.1 - middle_y);
    let (reach_x, reach_y) = (middle_x - inset, middle_y - inset);
    if dx.abs() <= reach_x && dy.abs() <= reach_y {
        return None;
    }
    let scale =
        (reach_x / dx.abs().max(f32::MIN_POSITIVE)).min(reach_y / dy.abs().max(f32::MIN_POSITIVE));
    let length = dx.hypot(dy);
    Some((
        (middle_x + dx * scale, middle_y + dy * scale),
        (dx / length, dy / length),
    ))
}

/// The corners of an arrowhead pointing along `direction` (a unit vector).
pub(super) fn arrow_corners(
    centre: (f32, f32),
    direction: (f32, f32),
    size: f32,
) -> [(f32, f32); 4] {
    let (along_x, along_y) = direction;
    let (across_x, across_y) = (-along_y, along_x);
    let at = |along: f32, across: f32| {
        (
            centre.0 + along_x * along + across_x * across,
            centre.1 + along_y * along + across_y * across,
        )
    };
    // A notched back makes it an arrowhead rather than a triangle.
    [
        at(size * 0.6, 0.0),
        at(-size * 0.5, size * 0.5),
        at(-size * 0.2, 0.0),
        at(-size * 0.5, -size * 0.5),
    ]
}

/// A white arrowhead with a black outline, in the dot's style, pointing along
/// `direction` (a unit vector).
pub fn arrow(
    canvas: &mut Canvas,
    centre: (f32, f32),
    direction: (f32, f32),
    size: f32,
    outline: f32,
) {
    let corners = arrow_corners(centre, direction, size);
    canvas.shape(centre, size + outline, BLACK, |x, y| {
        polygon_distance(&corners, x, y) - outline
    });
    canvas.shape(centre, size, WHITE, |x, y| polygon_distance(&corners, x, y));
}
