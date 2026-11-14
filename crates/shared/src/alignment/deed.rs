//! What moves an alignment: deeds, each worth a whole number, with no
//! randomness. Killing, and quests' rescues, escorts, help and
//! dark work. Every number is *to balance*.

use serde::{Deserialize, Serialize};

use super::{Alignment, Band, GOOD_FROM, MOST_GOOD};

/// What killing someone Neutral costs.
pub const KILLED_NEUTRAL: i32 = -10;
/// What killing the best of the Good costs; one only just Good costs
/// `KILLED_NEUTRAL`, and between the two it goes by how good they were.
pub const KILLED_BEST: i32 = -60;
/// What rescuing someone who needed it is worth.
pub const RESCUED: i32 = 20;
/// What seeing someone safely where they were going is worth.
pub const ESCORTED: i32 = 15;
/// What any other help to someone who asked is worth.
pub const HELPED: i32 = 5;
/// What failing someone one promised to save or see safely somewhere costs.
pub const BETRAYED_QUEST: i32 = -30;
/// What a dark quest done costs: a murder, a theft, a framing, a silencing.
pub const DARK_QUEST: i32 = -20;

/// Something done that moves its doer's alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Deed {
    /// Killed someone with this alignment. `provoked`: in self-defence (they
    /// attacked first, or were unfriendly from the start), which costs
    /// nothing (owner, 2026-11-12).
    Killed { victim: Alignment, provoked: bool },
    /// Rescued someone (a quest's).
    Rescued,
    /// Saw someone safely where they were going (a quest's escort).
    Escorted,
    /// Helped someone who asked (a quest done for them).
    Helped,
    /// Failed someone they had promised to save or escort.
    BetrayedQuest,
    /// Did a dark quest's work (Quest system PR 10).
    DarkQuest,
}

impl Deed {
    /// How much it moves an alignment. Killing someone Evil, or in self-defence,
    /// costs nothing, and no killing raises it: good comes only from helping.
    pub fn shift(self) -> i32 {
        match self {
            Self::Killed { provoked: true, .. } => 0,
            Self::Killed { victim, .. } => match victim.band() {
                Band::Evil => 0,
                Band::Neutral => KILLED_NEUTRAL,
                Band::Good => {
                    let over = i32::from(victim.value() - GOOD_FROM);
                    let span = i32::from(MOST_GOOD - GOOD_FROM);
                    KILLED_NEUTRAL + (KILLED_BEST - KILLED_NEUTRAL) * over / span
                }
            },
            Self::Rescued => RESCUED,
            Self::Escorted => ESCORTED,
            Self::Helped => HELPED,
            Self::BetrayedQuest => BETRAYED_QUEST,
            Self::DarkQuest => DARK_QUEST,
        }
    }
}
