//! What becomes of an ability used: for `ServerMessage::AbilityLanded`.

use serde::{Deserialize, Serialize};

use crate::vitals::Points;

/// How an ability's wind-up ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Landed {
    /// It did this much harm, and the one hit has this much of their Health
    /// left, per mille (for health bars over tokens).
    Hit { harm: Points, left: u16 },
    /// Its target was no longer there to be reached (`shared::combat::in_reach`).
    Fizzled,
    /// It was called off before it landed.
    Cancelled,
    /// It gave this much Health (some of it lost if beyond the most), and the
    /// one healed has this much left, per mille.
    Healed { amount: Points, left: u16 },
    /// A kick landed on a door (`target` is `None`): this much of it is left,
    /// per mille; 0 means it broke.
    Door { left: u16 },
}
