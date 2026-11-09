//! The player's mark, the compass, and the minimap's frame and sky.

use bevy::prelude::*;

use super::canvas::Canvas;
use super::frame::{FrameCache, RoundFrame};
use super::projection::Projection;
use super::{
    ARROW_INSET, BORDER, COMPASS_INSET, COMPASS_LINE, COMPASS_MIN_LENGTH, COMPASS_OF_HEIGHT,
    DOT_OUTLINE, DOT_PIXELS, FLOOR_HINT_PIXELS, HOVER_LIFT, NOTCH, OTHER_FLOOR_SIZE, Room,
    minimap_trim,
};
use crate::common::SkyNow;

/// The player on a map's marks layer: a dot, or on the full map an arrow at
/// the edge pointing to them, smaller with a hint which way their floor is
/// (`Some(player_is_up)`) when another floor is shown. `mark` is the dot's
/// size against `DOT_PIXELS`: the map's zoom times the screen's pixels.
pub(super) fn player_mark(
    canvas: &mut Canvas,
    player_at: (f32, f32),
    mark: f32,
    full_map: bool,
    other_floor: Option<bool>,
) {
    let (dot, outline) = (DOT_PIXELS * mark, DOT_OUTLINE * mark);
    if !full_map {
        super::frame::player_dot(canvas, player_at, dot, outline);
        return;
    }
    let picture_size = (canvas.width as f32, canvas.height as f32);
    // On another floor they are smaller, with what says which way beside
    // them. The arrow is as wide as the dot, outline included, so what is
    // beside it stays put as it moves round the edge.
    let dot = match other_floor {
        Some(_) => dot * OTHER_FLOOR_SIZE,
        None => dot,
    };
    let arrow_place = super::frame::arrow_place(picture_size, player_at, ARROW_INSET * mark);
    let at = arrow_place.map_or(player_at, |(place, _)| place);
    match arrow_place {
        Some((place, direction)) => {
            let arrow = dot - 2.0 * outline;
            super::frame::arrow(canvas, place, direction, arrow, outline)
        }
        None => super::frame::player_dot(canvas, player_at, dot, outline),
    }
    if let Some(player_is_up) = other_floor {
        let hint = FLOOR_HINT_PIXELS * mark;
        super::frame::floor_hint(canvas, at, dot / 2.0, hint, player_is_up);
    }
}

/// The full map's compass (not in the Hall of Worlds, which has none: owner).
pub(super) fn full_map_compass(
    canvas: &mut Canvas,
    window: &Room,
    size: (u32, u32),
    (dots, scale): (f32, f32),
) {
    let length = (window.height() * COMPASS_OF_HEIGHT).max(COMPASS_MIN_LENGTH);
    let foot = (
        size.0 as f32 - COMPASS_INSET * dots,
        size.1 as f32 - HOVER_LIFT * scale,
    );
    super::frame::compass(canvas, foot, length * scale, COMPASS_LINE * dots);
}

/// How thick the minimap's border is drawn, in the picture's pixels.
pub(super) fn minimap_border(window: &Room) -> f32 {
    BORDER * window.scale_factor() * minimap_trim(window)
}

/// The minimap's round frame (with its north notch if `north`), and the suns
/// and moons round it when `sky` is given.
pub(super) fn minimap_marks(
    canvas: &mut Canvas,
    (window, frame): (&Room, &mut FrameCache),
    projection: &Projection,
    clip: f32,
    (grid_right, grid_down): ((f64, f64), (f64, f64)),
    sky: Option<&SkyNow>,
    north: bool,
) {
    let trim = window.scale_factor() * minimap_trim(window);
    // North on the picture: the grid's north is `-y`.
    let round = RoundFrame {
        centre: projection.middle,
        radius: clip,
        border: minimap_border(window),
        notch: if north { NOTCH * trim } else { 0.0 },
        north: (-grid_right.1 as f32, -grid_down.1 as f32),
    };
    frame.draw(&round, canvas);
    // Last, as they are outside what that cuts off.
    if let Some(sky_now) = sky {
        let middle = Vec2::from(projection.middle);
        let icons = super::sky::icons(&sky_now.bodies, middle, clip, trim);
        super::sky::draw(canvas, &icons, &sky_now.bodies, &round, trim);
    }
}
