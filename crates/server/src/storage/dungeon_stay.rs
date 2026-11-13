//! A character's time in a quest's own dungeon (`shared::quest::QuestDungeon`),
//! kept in their `QuestBook`: where leaving it takes them back to, and when
//! they last left, as it resets about an hour after (owner).

use serde::{Deserialize, Serialize};

use super::VisitSave;

/// One quest dungeon a character has been in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DungeonStay {
    /// Where they were when they went in (beside the giver's door), where
    /// leaving by its way up takes them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back: Option<VisitSave>,
    /// When they last came out (Unix seconds); `None` while inside, even
    /// logged out, so no reset comes then (owner).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<i64>,
}
