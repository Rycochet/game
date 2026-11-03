//! The player's mark, the compass, and the minimap's frame and sky.

use bevy::prelude::*;

use super::canvas::Canvas;
use super::frame::RoundFrame;
use super::projection::Projection;
use super::{
    ARROW_INSET, BORDER, COMPASS_INSET, COMPASS_LINE, COMPASS_MIN_LENGTH, COMPASS_OF_HEIGHT,
    DOT_OUTLINE, DOT_PIXELS, FLOOR_HINT_PIXELS, HOVER_LIFT, NOTCH, OTHER_FLOOR_SIZE, Room,
    minimap_trim,
};
use crate::common::SkyNow;

/// The player on the full map: a dot, or an arrow at the edge pointing to
/// them, smaller with a hint which way their floor is (`Some(player_is_up)`)
/// when another floor is shown; and the compass.
pub(super) fn full_map_marks(
    canvas: &mut Canvas,
    window: &Room,
    size: (u32, u32),
    player_at: (f32, f32),
    (zoom, dots, scale): (f32, f32, f32),
    other_floor: Option<bool>,
) {
    let (on_other_floor, player_is_up) = (other_floor.is_some(), other_floor == Some(true));
    // The dot grows with the map.
    let (dot, outline) = (DOT_PIXELS * zoom * dots, DOT_OUTLINE * zoom * dots);
    let picture_size = (size.0 as f32, size.1 as f32);
    // On another floor they are smaller, with what says which way beside
    // them. The arrow is as wide as the dot, outline included, so what is
    // beside it stays put as it moves round the edge.
    let (dot, hint) = if on_other_floor {
        (
            dot * OTHER_FLOOR_SIZE,
            Some(FLOOR_HINT_PIXELS * zoom * dots),
        )
    } else {
        (dot, None)
    };
    let arrow_place = super::frame::arrow_place(picture_size, player_at, ARROW_INSET * zoom * dots);
    let mark = arrow_place.map_or(player_at, |(place, _)| place);
    match arrow_place {
        Some((place, direction)) => {
            let arrow = dot - 2.0 * outline;
            super::frame::arrow(canvas, place, direction, arrow, outline)
        }
        None => super::frame::player_dot(canvas, player_at, dot, outline),
    }
    if let Some(hint) = hint {
        super::frame::floor_hint(canvas, mark, dot / 2.0, hint, player_is_up);
    }
    let length = (window.height() * COMPASS_OF_HEIGHT).max(COMPASS_MIN_LENGTH);
    let foot = (
        size.0 as f32 - COMPASS_INSET * dots,
        size.1 as f32 - HOVER_LIFT * scale,
    );
    super::frame::compass(canvas, foot, length * scale, COMPASS_LINE * dots);
}

/// The player's dot on the minimap, its round frame, and the suns and moons
/// round it when `sky` is given.
#[allow(clippy::too_many_arguments)]
pub(super) fn minimap_marks(
    canvas: &mut Canvas,
    window: &Room,
    projection: &Projection,
    player_at: (f32, f32),
    clip: f32,
    (zoom, dots, scale): (f32, f32, f32),
    (grid_right, grid_down): ((f64, f64), (f64, f64)),
    sky: Option<&SkyNow>,
) {
    let (dot, outline) = (DOT_PIXELS * zoom * dots, DOT_OUTLINE * zoom * dots);
    super::frame::player_dot(canvas, player_at, dot, outline);
    let trim = scale * minimap_trim(window);
    // North on the picture: the grid's north is `-y`.
    let round = RoundFrame {
        centre: projection.middle,
        radius: clip,
        border: BORDER * trim,
        notch: NOTCH * trim,
        north: (-grid_right.1 as f32, -grid_down.1 as f32),
    };
    round.draw(canvas);
    // Last, as they are outside what that cuts off.
    if let Some(sky_now) = sky {
        let middle = Vec2::from(projection.middle);
        let icons = super::sky::icons(&sky_now.bodies, middle, clip, trim);
        super::sky::draw(canvas, &icons, &sky_now.bodies, &round, trim);
    }
}
