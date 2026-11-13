//! Quests as the client knows them: the character's log, the markers over the
//! doors of the town they are in, the planets with quests for the Hall's
//! gates, all from the server; and the "!" and "?" shown over doors, gates and
//! the world map's pois (the maps draw theirs from `QuestPois`). See
//! `docs/context/client/quests.md`.

use bevy::prelude::*;

// Split into files (`cargo xtask lint` keeps files small).
mod anchors;
mod book;
mod dots;
mod look;
mod over_view;
mod poi_marks;
mod read;
mod words;

pub use book::{BossMarks, DoorMarkers, GateQuests, QuestBook, QuestPlanets, done_steps};
pub use dots::{LevelDots, OpenQuestLog, TrackedDots};
pub use look::MarkLook;
pub use poi_marks::QuestPois;
pub use words::{or_partly, plain, reward_line, step_lines};

pub struct QuestsPlugin;

impl Plugin for QuestsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuestBook>()
            .init_resource::<DoorMarkers>()
            .init_resource::<GateQuests>()
            .init_resource::<QuestPlanets>()
            .init_resource::<QuestPois>()
            .init_resource::<BossMarks>()
            .init_resource::<LevelDots>()
            .init_resource::<TrackedDots>()
            .add_message::<OpenQuestLog>()
            .add_systems(
                Update,
                (
                    read::read_server,
                    poi_marks::find_quest_pois,
                    dots::find_tracked_dots,
                )
                    .chain(),
            )
            // After everything has moved, so the markers stay on what they mark.
            .add_systems(
                PostUpdate,
                over_view::show_marks.after(TransformSystems::Propagate),
            );
    }
}

#[cfg(test)]
pub(crate) mod tests;
