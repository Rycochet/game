//! `SavedNode`: one node as written in saves, `("combat/melee", practice, best, used,
//! clock)`. A save from before rust has only the path and practice: its best is
//! that practice, and it counts as used when the character next joins
//! (`learned::NOT_SEEN`).

use std::fmt;

use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::learned::{Learned, NOT_SEEN};

pub(super) struct SavedNode {
    pub path: String,
    pub learned: Learned,
}

impl<'de> Deserialize<'de> for SavedNode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_tuple(5, Fields)
    }
}

struct Fields;

impl<'de> Visitor<'de> for Fields {
    type Value = SavedNode;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a skill's path and practice, then perhaps its best, used and clock")
    }

    fn visit_seq<L: SeqAccess<'de>>(self, mut fields: L) -> Result<SavedNode, L::Error> {
        let path: String = fields
            .next_element()?
            .ok_or_else(|| L::Error::invalid_length(0, &self))?;
        let practice: u32 = fields
            .next_element()?
            .ok_or_else(|| L::Error::invalid_length(1, &self))?;
        let best: Option<u32> = fields.next_element()?;
        let used: Option<u64> = fields.next_element()?;
        let clock: Option<u64> = fields.next_element()?;
        Ok(SavedNode {
            path,
            learned: Learned {
                practice,
                best: best.unwrap_or(practice).max(practice),
                used: used.unwrap_or(NOT_SEEN),
                clock: clock.unwrap_or(0),
            },
        })
    }
}
