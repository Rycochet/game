//! The movement pad's buttons and the steps they ask for (`step_pad` spawns
//! and reads them; `player` steps, `site` says whether Up and Down work).

use bevy::prelude::*;

/// One of the movement pad's buttons. The arrows are screen directions, as the
/// step keys are; Up and Down are the stairs (and leaving or exploring).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadButton {
    Up,
    StepUp,
    Down,
    StepLeft,
    StepDown,
    StepRight,
    /// Turn the character (never the camera), held to keep turning.
    TurnLeft,
    TurnRight,
}

/// Which way the pad's arrows held now ask the player to step, in screen
/// directions as the step keys: -1, 0 or 1 across (`dx`) and down (`dy`).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PadSteps {
    pub dx: i64,
    pub dy: i64,
    /// The turn buttons held: -1 left, 1 right, 0 neither or both.
    pub turn: i64,
}
