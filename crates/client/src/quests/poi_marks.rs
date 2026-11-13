//! The pois with a step of a quest taken that can be done now ("!"), and the
//! towns of givers waiting for a quest to be handed in ("?") or whose boss
//! the character has already killed ("!"): for the world map and the maps
//! (owner).

use bevy::prelude::*;
use shared::poi::ChunkPos;
use shared::protocol::QuestState;
use shared::quest::Target;
use std::collections::BTreeMap;

use super::{MarkLook, QuestBook, done_steps};

/// What to mark each poi with, by its chunk.
#[derive(Resource, Default)]
pub struct QuestPois(pub BTreeMap<ChunkPos, MarkLook>);

/// Works them out again whenever the book changes.
pub(super) fn find_quest_pois(book: Res<QuestBook>, mut pois: ResMut<QuestPois>) {
    if book.is_changed() {
        pois.0 = marks_of(&book);
    }
}

/// What to mark each poi with, from the character's quests.
pub(super) fn marks_of(book: &QuestBook) -> BTreeMap<ChunkPos, MarkLook> {
    let mut marks = BTreeMap::new();
    for entry in book.entries.values() {
        match entry.state {
            QuestState::Ready => {
                // "?" wins over "!" anywhere.
                marks.insert(entry.id.town, MarkLook::new(entry.kind, true));
            }
            QuestState::Taken => {
                let done = done_steps(entry);
                let open = (entry.steps.iter().enumerate())
                    .filter(|(i, step)| !done[*i] && step.is_open(&done));
                for (_, step) in open {
                    let chunk = match step.target {
                        Target::Poi { chunk, .. } | Target::Town { chunk } => chunk,
                        // The wilds round a town have no poi to mark.
                        Target::Wilds { .. } => continue,
                    };
                    marks
                        .entry(chunk)
                        .or_insert(MarkLook::new(entry.kind, false));
                }
            }
            _ => {}
        }
    }
    // A boss already dealt with: its giver's town (owner).
    for slain in &book.slain {
        marks
            .entry(slain.id.town)
            .or_insert(MarkLook::new(slain.kind, false));
    }
    marks
}
