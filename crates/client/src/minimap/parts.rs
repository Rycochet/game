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

/// What of the maps an element is, for reading `dom::Hit`: `dom`'s search
/// finds what the pointer is on, and the maps only turn a place on these
/// into their own.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MapPart {
    /// The minimap's round face (`minimap/disc`), without the suns and
    /// moons round it.
    Minimap,
    /// The full map's parchment, and all on it.
    Parchment,
    /// The full map's picture, on the parchment.
    Picture,
}

impl MapPart {
    /// Whether the pointer, at the end of `chain` (`dom::Hit::chain`, or a
    /// finger's from `dom::Finder`), is on this part or in it.
    pub(super) fn holds(self, chain: &[Entity], parts: &Query<&MapPart>) -> bool {
        chain.iter().any(|held| parts.get(*held) == Ok(&self))
    }
}
