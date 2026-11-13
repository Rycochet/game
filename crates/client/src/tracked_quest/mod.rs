//! The tracked quest under the minimap (owner): the quest followed on this
//! planet (`quests::QuestBook::followed`), with its giver's marker and its
//! name, the objectives that can be done now with their counts, and "Return
//! to ..." with "?" once only handing in is left. Clicking or tapping it opens
//! the Quest Log on it. See `docs/context/client/tracked_quest.md`.

use bevy::prelude::*;

// Split into files (`cargo xtask lint` keeps files small).
mod lines;
mod markup;
mod show;

pub struct TrackedQuestPlugin;

impl Plugin for TrackedQuestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, markup::spawn_box)
            .add_systems(Update, (markup::bind_box, show::show_tracked).chain());
    }
}

#[cfg(test)]
mod tests;
