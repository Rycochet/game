//! A glow round a creature's model, in a colour: a quest boss's tier
//! (owner), set by `entities`, drawn by `view3d` behind the black outline.

use bevy::prelude::*;

/// Its owner's model glows in this colour (sRGB bytes).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TokenGlow(pub [u8; 3]);
