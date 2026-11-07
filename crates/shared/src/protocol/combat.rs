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
}
