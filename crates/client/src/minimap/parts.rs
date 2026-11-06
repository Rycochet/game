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

/// Under the full map's hover box's name, a World Gate's code in runes
/// (`gate_runes`), and its picture.
#[derive(Component)]
pub(super) struct HoverRunes;

#[derive(Component)]
pub(super) struct HoverRunesPicture;

/// The pictures the maps are drawn in, and the see-through marks layers over
/// them (`overlay`).
#[derive(Resource)]
pub(super) struct MapPictures {
    pub(super) minimap: Handle<Image>,
    pub(super) full: Handle<Image>,
    pub(super) minimap_marks: Handle<Image>,
    pub(super) full_marks: Handle<Image>,
}
