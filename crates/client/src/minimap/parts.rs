//! The maps' parts on screen, and the pictures they are drawn in.

use bevy::prelude::*;

/// The picture of a World Gate's code in runes (`gate_runes`), under the full
/// map's hover box's name.
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
