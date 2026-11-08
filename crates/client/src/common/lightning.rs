//! Lightning flashing where the player is: how bright, for the views to draw.

use bevy::prelude::*;

/// How bright lightning makes everything under the sky this frame: 0 none,
/// 1 a full flash. Kept by the `lightning` module; it only changes during a
/// flash, so views can redraw on `is_changed`.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct Lightning(pub f32);

/// Starts a flash now, as the server's `Lightning` message does (the debug
/// menu's Lightning Now).
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct LightningStrike;
