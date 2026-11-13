//! The Quest Log (L): a moving panel with the character's quests on any
//! planet they have quested on, chosen by its runes; grouped under way,
//! handed in, failed or given up, each with its giver's marker; one in full
//! beside them, with Track and Abandon, and its places shown on the map; and
//! the dailies done for each giver. See `docs/context/client/quest_log.md`.

use bevy::prelude::*;

// Split into files (`cargo xtask lint` keeps files small).
mod detail;
mod entries;
mod input;
mod markup;
mod open;
mod page;
mod planets;
mod read;
mod runes;
mod show;
mod state;

pub struct QuestLogPlugin;

impl Plugin for QuestLogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<state::LogPanel>().add_systems(
            Update,
            (
                open::notice_closed,
                open::toggle_log,
                read::read_server,
                input::read_events,
                show::show_log,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests;
