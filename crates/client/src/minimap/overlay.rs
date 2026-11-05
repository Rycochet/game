//! The marks layer: a see-through picture over each map holding only what
//! moves (the player's mark, and dots for mobs and other players), so that
//! someone stepping redraws this layer and not the map under it.

use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;

use super::MapPictures;
use super::canvas::Canvas;
use super::marks::player_mark;
use super::others::{Seen, draw_others, seen};
use super::projection::Projection;
use crate::common::{CurrentDungeon, MapPosition, Other, OtherLevels, SmoothPosition, Token};
use crate::player::Player;

/// What the map under the marks was last drawn with, which they must match:
/// set by `draw_maps` whenever it draws.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Under {
    pub(super) projection: Projection,
    pub(super) size: (u32, u32),
    pub(super) full_map: bool,
    /// The circle dots are cut to (centre, radius): inside the minimap's
    /// border, or `None` on the full map.
    pub(super) inside: Option<((f32, f32), f32)>,
    /// The marks' size against their own: the zoom times the screen's pixels.
    pub(super) mark: f32,
    /// Another floor than the player's is shown, which way theirs is.
    pub(super) other_floor: Option<bool>,
    /// The world's width, on the world map, where places wrap round.
    pub(super) wrap: Option<i64>,
}

/// The map the marks go over, as last drawn.
#[derive(Resource, Default)]
pub(super) struct MarksLayer(pub(super) Option<Under>);

/// What the marks were last drawn from: if nothing here changes, they needn't be.
#[derive(PartialEq)]
pub(super) struct MarksFrom {
    under: Under,
    player: (f64, f64),
    others: Vec<Seen>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_marks(
    layer: Res<MarksLayer>,
    pictures: Res<MapPictures>,
    mut images: ResMut<Assets<Image>>,
    player: Single<&SmoothPosition, With<Player>>,
    dungeon: Res<CurrentDungeon>,
    levels: Res<OtherLevels>,
    others: Query<(&MapPosition, &Token), With<Other>>,
    mut last: Local<Option<MarksFrom>>,
) {
    let Some(under) = layer.0 else {
        return;
    };
    let spot = levels.spot(&dungeon, (player.x, player.y));
    let now = MarksFrom {
        under,
        player: spot,
        // Others are only on the player's floor.
        others: match under.other_floor {
            Some(_) => Vec::new(),
            None => seen(&others, spot, under.wrap),
        },
    };
    if last.as_ref() == Some(&now) {
        return;
    }
    let picture = if under.full_map {
        &pictures.full_marks
    } else {
        &pictures.minimap_marks
    };
    let Some(mut image) = images.get_mut(picture) else {
        return;
    };
    let (width, height) = under.size;
    if image.width() != width || image.height() != height {
        image.resize(Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        });
    }
    let Some(pixels) = image.data.as_mut() else {
        return;
    };
    let mut canvas = Canvas {
        pixels,
        width,
        height,
    };
    paint(&mut canvas, &now);
    *last = Some(now);
}

/// Paints the marks on a cleared layer: others first, so the player is on top.
pub(super) fn paint(canvas: &mut Canvas, from: &MarksFrom) {
    canvas.fill([0, 0, 0, 0]);
    let under = &from.under;
    draw_others(canvas, under, &from.others);
    let player_at = under.projection.to_picture(from.player.0, from.player.1);
    player_mark(
        canvas,
        player_at,
        under.mark,
        under.full_map,
        under.other_floor,
    );
}

/// `MarksFrom` for tests.
#[cfg(test)]
pub(super) fn marks_from(under: Under, player: (f64, f64), others: Vec<Seen>) -> MarksFrom {
    MarksFrom {
        under,
        player,
        others,
    }
}

#[cfg(test)]
mod tests;
