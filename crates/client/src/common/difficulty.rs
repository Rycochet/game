//! The character's difficulty (`shared::difficulty`) as the server last told
//! it, and the words shown for each difficulty.

use bevy::prelude::*;
use shared::difficulty::{Difficulties, Difficulty};

/// What the server said of difficulty: the character's (`None` before there is
/// one, while making one, or with no server) and the ones it allows. Set by
/// `connection`; read by Settings' Gameplay page and the creation screen.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayDifficulty {
    pub current: Option<Difficulty>,
    pub allowed: Difficulties,
}

/// What a difficulty means, for tips.
pub fn difficulty_words(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Easy => {
            "Easy: you take a fifth less harm, Health and Mana refill twice as fast, and dying \
             costs nothing."
        }
        Difficulty::Normal => "Normal: the game as it is meant to be played.",
        Difficulty::Hard => {
            "Hard: you take a fifth more harm, and dying drops everything but what is worn \
             against the skin."
        }
        Difficulty::Ironman => {
            "Ironman: you take a fifth more harm, and death is permanent. Only chosen when \
             making a character."
        }
    }
}

/// The question asked before Ironman is chosen.
pub const IRONMAN_QUESTION: &str = "On Ironman, death is permanent: when the character \
     dies, everything they carry and wear is dropped and they are deleted. A character can \
     never change to or from Ironman later. Are you sure?";
