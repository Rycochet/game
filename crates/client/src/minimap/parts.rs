//! The maps' parts on screen, and the pictures they are drawn in.

use bevy::prelude::*;

#[derive(Component)]
pub(super) struct Minimap;

#[derive(Component)]
pub(super) struct FullMapPanel;

/// Darkens everything outside the full map, and stops widgets under it being
/// used; presses there are left for `take_input`, which closes the map.
#[derive(Component)]
pub(super) struct FullMapDarkening;

#[derive(Component)]
pub(super) struct FullMapPicture;

#[derive(Component)]
pub(super) struct FullMapTitle;

/// Holds the box naming what is under the mouse, centred along the parchment.
#[derive(Component)]
pub(super) struct HoverBox;

#[derive(Component)]
pub(super) struct HoverText;

/// The box naming the poi the mouse is over on the world map itself.
#[derive(Component)]
pub(super) struct WorldHoverBox;

#[derive(Component)]
pub(super) struct WorldHoverText;

/// The pictures the maps are drawn in.
#[derive(Resource)]
pub(super) struct MapPictures {
    pub(super) minimap: Handle<Image>,
    pub(super) full: Handle<Image>,
}
