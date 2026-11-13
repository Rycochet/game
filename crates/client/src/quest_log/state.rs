//! What the open Quest Log is showing, and what it has been told of other
//! planets.

use bevy::prelude::*;
use shared::poi::ChunkPos;
use shared::protocol::{DailyDone, LogEntry};
use shared::quest::QuestId;
use shared::world::WorldId;
use std::collections::BTreeMap;

/// The Quest Log, while open (`document` set).
#[derive(Resource, Default)]
pub(super) struct LogPanel {
    pub(super) document: Option<Entity>,
    /// The planet shown: `None` for the one the player is on.
    pub(super) planet: Option<WorldId>,
    /// The quest shown in full.
    pub(super) chosen: Option<QuestId>,
    /// Abandon was pressed once for the quest shown: a second press gives it
    /// up.
    pub(super) abandoning: bool,
    /// Other planets' logs, as the server told them.
    pub(super) others: BTreeMap<WorldId, Told>,
    /// The planets in the chooser, in its order (the one the player is on
    /// first), to know which a choice is.
    pub(super) choices: Vec<Option<WorldId>>,
    /// The quest each row of the list shows, by its index.
    pub(super) rows: Vec<QuestId>,
    /// The poi chunk each target button shows on the map, by its index.
    pub(super) targets: Vec<ChunkPos>,
    /// Pictures of each planet's runes, made once.
    pub(super) runes: BTreeMap<WorldId, Handle<Image>>,
    /// The chooser's options need putting in again.
    pub(super) planets_dirty: bool,
    /// Something shown has changed.
    pub(super) dirty: bool,
}

/// Another planet's quests and dailies.
#[derive(Clone, Debug, Default)]
pub(super) struct Told {
    pub(super) quests: Vec<LogEntry>,
    pub(super) dailies: Vec<DailyDone>,
}

impl LogPanel {
    pub(super) fn is_open(&self) -> bool {
        self.document.is_some()
    }
}
