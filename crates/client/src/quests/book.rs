//! What the client knows of the character's quests and of the doors and gates
//! with quests: all told by the server (`shared::protocol::QuestNews`).

use bevy::prelude::*;
use shared::poi::ChunkPos;
use shared::protocol::{LogEntry, Marker};
use shared::quest::QuestId;
use shared::world::WorldId;
use std::collections::BTreeMap;

/// The character's quests on this planet, as their log shows them.
#[derive(Resource, Default)]
pub struct QuestBook {
    pub entries: BTreeMap<QuestId, LogEntry>,
}

/// Which steps of a quest are done, from how far along each is.
pub fn done_steps(entry: &LogEntry) -> Vec<bool> {
    (entry.steps.iter().enumerate())
        .map(|(i, step)| entry.counts.get(i).copied().unwrap_or(0) >= step.count.unwrap_or(1))
        .collect()
}

/// The markers over the doors of the town the player is in (or was last in):
/// by building.
#[derive(Resource, Default)]
pub struct DoorMarkers {
    pub town: Option<ChunkPos>,
    pub markers: BTreeMap<u16, Marker>,
}

/// The planets with quests taken, and whether one waits to be handed in
/// there: for the Hall's gates.
#[derive(Resource, Default)]
pub struct GateQuests(pub BTreeMap<WorldId, bool>);
