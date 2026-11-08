//! An account's characters, as kept with the account and as offered to choose
//! from (`ServerMessage::Characters`).

use serde::{Deserialize, Serialize};

use super::CharacterId;
use crate::creature::CreatureKind;
use crate::difficulty::Difficulty;
use crate::sheet::{Appearance, Sheet};
use crate::wear::WornLook;

/// A character as their account keeps them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub id: CharacterId,
    pub name: String,
    /// When last played (seconds since 1970), 0 for never.
    pub last_played: u64,
    /// When made (seconds since 1970). Missing from older saves, which are
    /// given the time they are next loaded.
    #[serde(default)]
    pub created: u64,
}

/// A character offered to choose from: their summary, and enough of their save
/// to show them as they are now.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListedCharacter {
    pub summary: CharacterSummary,
    /// The body they are in now.
    pub kind: CreatureKind,
    pub appearance: Appearance,
    /// What can be seen of what they wear (`Inventory::shown`).
    pub worn: Vec<WornLook>,
    pub sheet: Sheet,
    pub difficulty: Option<Difficulty>,
}
