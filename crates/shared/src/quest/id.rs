//! Which quest a quest is: its town and its place among the town's quests.
//! Worked out again from the seed whenever it is needed, so this is all a
//! save or a message keeps.

use serde::{Deserialize, Serialize};

use crate::poi::ChunkPos;

/// A quest, by its town and its slot there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct QuestId {
    /// The giver's town.
    pub town: ChunkPos,
    pub slot: Slot,
}

/// Where in a town a quest comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Slot {
    /// A giver's slot: a building (its `RoomId`), the slot's number there,
    /// and the quest's place in the slot's chain (0 first).
    Giver { building: u16, slot: u8, chain: u8 },
    /// The town's one feeder quest, taking something to a larger town joined
    /// by road. Its giver is chosen when the town's own quests are made, so
    /// a larger town can name it without reading the smaller town's
    /// buildings.
    Feeder,
    /// A giver's daily quest on a UTC day (days since 1970).
    Daily { building: u16, day: i64 },
    /// A city's artifact line: its giver's `step`th quest (0 first), each
    /// needing the one before.
    Artifact { step: u8 },
    // New slots go at the end: messages carry one by its place.
}

impl QuestId {
    /// The quest before this one in its chain, if it has one.
    pub fn earlier(self) -> Option<Self> {
        match self.slot {
            Slot::Giver {
                building,
                slot,
                chain,
            } if chain > 0 => Some(Self {
                slot: Slot::Giver {
                    building,
                    slot,
                    chain: chain - 1,
                },
                ..self
            }),
            Slot::Artifact { step } if step > 0 => Some(Self {
                slot: Slot::Artifact { step: step - 1 },
                ..self
            }),
            _ => None,
        }
    }
}
