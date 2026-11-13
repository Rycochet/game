//! What the client knows of the character's quests and of the doors and gates
//! with quests: all told by the server (`shared::protocol::QuestNews`).

use bevy::prelude::*;
use shared::poi::ChunkPos;
use shared::protocol::{DailyDone, LogEntry, Marker, PlanetQuests, QuestState};
use shared::quest::QuestId;
use shared::world::WorldId;
use std::collections::BTreeMap;

/// The character's quests on this planet, as their log shows them.
#[derive(Resource, Default)]
pub struct QuestBook {
    pub entries: BTreeMap<QuestId, LogEntry>,
    /// The daily givers they have done dailies for here.
    pub dailies: Vec<DailyDone>,
    /// The quest they chose to follow (none: the newest taken).
    pub tracked: Option<QuestId>,
}

/// Every planet the character has quest history on, as last told (asked for
/// by the Quest Log, and told on coming into the Hall).
#[derive(Resource, Default)]
pub struct QuestPlanets(pub Vec<PlanetQuests>);

impl QuestBook {
    /// The quest followed: the one chosen while it is under way, else the
    /// newest taken (Claude, in the quests plan).
    pub fn followed(&self) -> Option<QuestId> {
        let under_way =
            |entry: &&LogEntry| matches!(entry.state, QuestState::Taken | QuestState::Ready);
        let chosen = self.tracked.and_then(|id| self.entries.get(&id));
        if let Some(entry) = chosen.filter(under_way) {
            return Some(entry.id);
        }
        let newest = self
            .entries
            .values()
            .filter(under_way)
            .max_by_key(|entry| entry.at);
        newest.map(|entry| entry.id)
    }
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
