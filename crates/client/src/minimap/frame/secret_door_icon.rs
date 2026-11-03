//! The "S" stamped over a secret door.

use std::f32::consts::{FRAC_PI_2, PI};

use super::super::canvas::Canvas;
use super::{SECRET_DOOR_GAP, SECRET_DOOR_INK, SECRET_DOOR_STROKE};

/// How far a point is from the middle of the S's top curve: round its circle,
/// from straight down (where it meets the bottom curve) through the left and
/// the top, stopping short of the right.
fn top_curve(radius: f32, (x, y): (f32, f32)) -> f32 {
    // The circle's centre is `radius` above the middle of the S. Angles are
    // as on the screen, y down: 0 to the right, a half turn down.
    let (dx, dy) = (x, y + radius);
    let start = FRAC_PI_2;
    let end = 2.0 * PI - SECRET_DOOR_GAP;
    let mut angle = dy.atan2(dx);
    if angle < 0.0 {
        angle += 2.0 * PI;
    }
    if (start..=end).contains(&angle) {
        (dx.hypot(dy) - radius).abs()
    } else {
        // Past an end: the distance to it, so the ends are round.
        let towards = |at: f32| (dx - radius * at.cos()).hypot(dy - radius * at.sin());
        towards(start).min(towards(end))
    }
}

/// An "S" `size` tall, upright whichever way the map is turned, with an edge
/// `edge` wide round it in `background`, so it reads over the door and the
/// hatching beside it.
pub fn secret_door_icon(
    canvas: &mut Canvas,
    centre: (f32, f32),
    size: f32,
    edge: f32,
    background: [u8; 3],
) {
    let stroke = size * SECRET_DOOR_STROKE;
    let radius = (size - stroke) / 4.0;
    // The bottom curve is the top one turned a half turn about the middle.
    let distance = |x: f32, y: f32| {
        let at = (x - centre.0, y - centre.1);
        top_curve(radius, at).min(top_curve(radius, (-at.0, -at.1))) - stroke / 2.0
    };
    let reach = size / 2.0 + edge;
    canvas.shape(centre, reach, background, |x, y| distance(x, y) - edge);
    canvas.shape(centre, reach, SECRET_DOOR_INK, distance);
}
