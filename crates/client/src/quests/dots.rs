//! The followed quest's dots for the maps (owner): one for every objective
//! that can be done now. On the world map, each open step's poi or town, or
//! the giver's town once every step is done; on a level, the cells the server
//! sends (`QuestNews::Dots`: stairs towards the objective, the giver's door, a
//! poi in the wilderness), as it decides what may be known there.

use bevy::prelude::*;
use shared::poi::ChunkPos;
use shared::protocol::{LogEntry, QuestState};
use shared::quest::{QuestId, Target};
use std::collections::BTreeMap;

use super::{MarkLook, QuestBook, done_steps};
use crate::common::{CurrentWorld, GameView};

/// The cells of the level the player is on that each quest's dots go on, as
/// the server last said.
#[derive(Resource, Default)]
pub struct LevelDots(pub BTreeMap<QuestId, Vec<(i64, i64)>>);

/// The followed quest's dots, where the maps draw them: world map points on
/// the world map, else the level's cells.
#[derive(Resource, Default, Clone, PartialEq)]
pub struct TrackedDots {
    /// The quest's colour (its "!"'s).
    pub rgb: [u8; 3],
    pub places: Vec<(i64, i64)>,
}

/// Works them out again when anything they come from changes.
pub(super) fn find_tracked_dots(
    book: Res<QuestBook>,
    level: Res<LevelDots>,
    view: Res<State<GameView>>,
    world: Option<Res<CurrentWorld>>,
    mut dots: ResMut<TrackedDots>,
) {
    let world_changed = world.as_ref().is_some_and(|world| world.is_changed());
    if !(book.is_changed() || level.is_changed() || view.is_changed() || world_changed) {
        return;
    }
    let entry = book.followed().and_then(|id| book.entries.get(&id));
    let now = match entry {
        None => TrackedDots::default(),
        Some(entry) => TrackedDots {
            rgb: MarkLook::new(entry.kind, false).rgb,
            places: match (view.get(), world) {
                (GameView::WorldMap, Some(world)) => (world_chunks(entry).into_iter())
                    .filter_map(|chunk| world.pois.poi_in(chunk))
                    .map(|poi| (poi.x, poi.y))
                    .collect(),
                (GameView::Dungeon, _) => level.0.get(&entry.id).cloned().unwrap_or_default(),
                _ => Vec::new(),
            },
        },
    };
    if *dots != now {
        *dots = now;
    }
}

/// The chunks of the world map a quest's objectives that can be done now are
/// in: each open step's poi or town (none for the wilds round a town), or the
/// giver's town when only handing in is left.
pub(super) fn world_chunks(entry: &LogEntry) -> Vec<ChunkPos> {
    match entry.state {
        QuestState::Ready => vec![entry.id.town],
        QuestState::Taken => {
            let done = done_steps(entry);
            let mut chunks: Vec<ChunkPos> = (entry.steps.iter().enumerate())
                .filter(|(i, step)| !done[*i] && step.is_open(&done))
                .filter_map(|(_, step)| match step.target {
                    // A quest's own dungeon is entered in its giver's town.
                    Target::Poi { chunk, .. }
                    | Target::Town { chunk }
                    | Target::Dungeon { chunk, .. } => Some(chunk),
                    Target::Wilds { .. } => None,
                })
                .collect();
            chunks.dedup();
            chunks
        }
        _ => Vec::new(),
    }
}

/// Asks the Quest Log to open on a quest (the tracked quest under the
/// minimap, clicked).
#[derive(Message, Clone, Copy, Debug)]
pub struct OpenQuestLog(pub QuestId);

#[cfg(test)]
mod tests;
