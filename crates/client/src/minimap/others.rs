//! Mobs and other players on the maps: a dot like the player's for each one
//! the server says can be seen, in the colour of how it feels about the player.

use bevy::prelude::*;

use super::canvas::Canvas;
use super::overlay::Under;
use super::{DOT_OUTLINE, DOT_PIXELS};
use crate::common::{Attitude, MapPosition, Other, Token};

/// Where someone is, the near way round the world, and their colour.
pub(super) type Seen = ((i64, i64), Attitude);

/// Everyone the player is told of, in a set order so a list can be compared
/// with the last. On the world map each is placed the near way round from
/// `player` (`wrap` is the world's width).
pub(super) fn seen(
    others: &Query<(&MapPosition, &Token), With<Other>>,
    player: (f64, f64),
    wrap: Option<i64>,
) -> Vec<Seen> {
    let mut seen: Vec<Seen> = others
        .iter()
        .map(|(at, token)| ((near_way_round(at.x, player.0, wrap), at.y), token.attitude))
        .collect();
    seen.sort_by_key(|&((x, y), attitude)| (x, y, attitude as u8));
    seen
}

/// `x`, moved by whole widths of the world to be nearest `from`.
pub(super) fn near_way_round(x: i64, from: f64, wrap: Option<i64>) -> i64 {
    let Some(width) = wrap.filter(|width| *width > 0) else {
        return x;
    };
    let from = from.round() as i64;
    from + (x - from + width / 2).rem_euclid(width) - width / 2
}

/// Their dots, as big as the player's, cut to the minimap's circle.
pub(super) fn draw_others(canvas: &mut Canvas, under: &Under, others: &[Seen]) {
    let (size, outline) = (DOT_PIXELS * under.mark, DOT_OUTLINE * under.mark);
    for &((x, y), attitude) in others {
        let at = under.projection.to_picture(x as f64, y as f64);
        let [red, green, blue, _] = attitude.colour().to_srgba().to_u8_array();
        super::frame::dot(
            canvas,
            at,
            (size, outline),
            [red, green, blue],
            under.inside,
        );
    }
}
