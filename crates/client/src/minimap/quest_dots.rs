//! The followed quest's dots on the maps' marks layer (owner): one for each
//! objective that can be done now (`quests::TrackedDots`), as big as the
//! player's with the same black edge, in the quest's colour; and where one is
//! off the map, an arrowhead of that colour at its edge pointing to it (round
//! the minimap's border, or the full map's box). Only on the player's floor.

use super::canvas::{Canvas, polygon_distance};
use super::frame::arrow_corners;
use super::others::near_way_round;
use super::overlay::Under;
use super::{ARROW_INSET, DOT_OUTLINE, DOT_PIXELS};

/// The followed quest's colour and its dots' places on the map, the near way
/// round the planet from the player.
#[derive(Clone, PartialEq, Default)]
pub(super) struct QuestMarks {
    pub(super) rgb: [u8; 3],
    pub(super) places: Vec<(i64, i64)>,
}

impl QuestMarks {
    pub(super) fn of(dots: &crate::quests::TrackedDots, player: (f64, f64), under: &Under) -> Self {
        if under.other_floor.is_some() {
            return Self::default();
        }
        let places = (dots.places.iter())
            .map(|&(x, y)| (near_way_round(x, player.0, under.wrap), y))
            .collect();
        Self {
            rgb: dots.rgb,
            places,
        }
    }
}

/// Each dot, or its arrow at the edge when it is off the map.
pub(super) fn draw_quest_dots(canvas: &mut Canvas, under: &Under, marks: &QuestMarks) {
    let (size, outline) = (DOT_PIXELS * under.mark, DOT_OUTLINE * under.mark);
    let inset = ARROW_INSET * under.mark;
    let picture = (canvas.width as f32, canvas.height as f32);
    for &(x, y) in &marks.places {
        // (As others' dots: a cell is drawn round its own place.)
        let at = under.projection.to_picture(x as f64, y as f64);
        let edge = match under.inside {
            Some((centre, radius)) => round_edge(at, centre, radius - inset),
            None => super::frame::arrow_place(picture, at, inset),
        };
        match edge {
            Some((place, direction)) => arrow(
                canvas,
                place,
                direction,
                size - 2.0 * outline,
                outline,
                marks.rgb,
            ),
            None => super::frame::dot(canvas, at, (size, outline), marks.rgb, under.inside),
        }
    }
}

/// Where an arrow to `target` goes on the circle `reach` round `centre`, and
/// which way it points; `None` if the target is inside it.
fn round_edge(
    target: (f32, f32),
    centre: (f32, f32),
    reach: f32,
) -> Option<((f32, f32), (f32, f32))> {
    let (dx, dy) = (target.0 - centre.0, target.1 - centre.1);
    let length = dx.hypot(dy);
    if length <= reach || length <= f32::EPSILON {
        return None;
    }
    let direction = (dx / length, dy / length);
    let place = (
        centre.0 + direction.0 * reach,
        centre.1 + direction.1 * reach,
    );
    Some((place, direction))
}

/// An arrowhead of `rgb` with a black outline, pointing along `direction`.
fn arrow(
    canvas: &mut Canvas,
    centre: (f32, f32),
    direction: (f32, f32),
    size: f32,
    outline: f32,
    fill: [u8; 3],
) {
    let corners = arrow_corners(centre, direction, size);
    canvas.shape(centre, size + outline, [0, 0, 0], |x, y| {
        polygon_distance(&corners, x, y) - outline
    });
    canvas.shape(centre, size, fill, |x, y| polygon_distance(&corners, x, y));
}

#[cfg(test)]
mod tests;
