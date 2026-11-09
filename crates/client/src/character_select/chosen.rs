//! Which character is chosen on the screen, and whether Delete is being asked about.

use bevy::prelude::*;
use shared::protocol::CharacterId;

use crate::common::CharacterSelect;

/// The character chosen, and whether the Delete requester is up for them.
#[derive(Resource, Default)]
pub(super) struct Chosen {
    pub id: Option<CharacterId>,
    pub deleting: bool,
}

/// Whenever the list changes: the same one stays chosen if still there, or else
/// the latest played. Nothing chosen once there are none.
pub(super) fn keep_chosen(select: Res<CharacterSelect>, mut chosen: ResMut<Chosen>) {
    if !select.is_changed() {
        return;
    }
    let sorted = select.sorted();
    let still = chosen
        .id
        .filter(|id| sorted.iter().any(|c| c.summary.id == *id));
    let id = still.or_else(|| sorted.first().map(|c| c.summary.id));
    if chosen.id != id || chosen.deleting {
        chosen.id = id;
        chosen.deleting = false;
    }
}
