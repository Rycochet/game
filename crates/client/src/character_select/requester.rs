//! "Delete <name>?": asked before a character is deleted, as it can't be undone.
//! Its dialog is in the screen's markup (`screen`).

use bevy::prelude::*;

use super::chosen::Chosen;
use crate::common::CharacterSelect;
use crate::dom::Dom;

/// The requester's answers. A press outside it is No too.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum Answer {
    Delete,
    Keep,
}

/// Up while Delete is being asked about, naming the character.
pub(super) fn show_requester(chosen: Res<Chosen>, select: Res<CharacterSelect>, mut dom: Dom) {
    if !chosen.is_changed() {
        return;
    }
    let name = chosen
        .id
        .and_then(|id| select.list.iter().find(|c| c.summary.id == id))
        .map(|c| c.summary.name.as_str());
    let asking = chosen.deleting && name.is_some();
    if let Some(name) = name {
        let question = format!("Delete {name}? This can't be undone.");
        dom.set_text("character_select/question", question);
    }
    dom.set_flag("character_select/asking", "hidden", !asking);
}
