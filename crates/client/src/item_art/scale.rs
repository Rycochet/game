//! Drawing with more pixels on a larger interface.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{ItemPicture, ItemPictures, MOST_DETAIL, SHARP_UP_TO};

/// How many times over the pixels asked for pictures are drawn with, for this many of
/// the screen's pixels to an interface pixel.
pub(super) fn detail_for(screen_per_interface: f32) -> u32 {
    ((screen_per_interface / SHARP_UP_TO).ceil() as u32).clamp(1, MOST_DETAIL)
}

/// Draws pictures with more pixels when the interface is drawn larger, and
/// gives those on show the new ones.
pub(super) fn follow_interface_scale(
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
    mut shown: Query<(&ItemPicture, &mut ImageNode)>,
) {
    let detail = detail_for(window.scale_factor() * ui_scale.0);
    if pictures.detail == detail {
        return;
    }
    pictures.detail = detail;
    for (picture, mut node) in &mut shown {
        node.image = pictures.picture(picture.look.clone(), picture.pixels, &mut images);
    }
}
