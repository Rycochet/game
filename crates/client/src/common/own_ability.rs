//! The ability the player is using, as the server timed it.

use bevy::prelude::*;

/// When the player's ability lands and when they can act again, by `Time`
/// (seconds since starting), from `ServerMessage::AbilityStarted`. `abilities`
/// keeps it; the action bar and tokens may show it.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct OwnAbility {
    /// When it began.
    pub started: f64,
    /// When its wind-up is over and it lands.
    pub lands: f64,
    /// When the player can act again.
    pub ready: f64,
    /// Until when an ability asked for hasn't been answered yet, so it isn't
    /// asked for again meanwhile.
    pub asked_until: f64,
}

/// How long before the player can act again the next ability may be asked
/// for, in seconds: the server keeps it until then, so the next begins as soon
/// as it can, however slowly frames come.
const ASK_AHEAD: f64 = 0.25;

impl OwnAbility {
    /// Whether nothing more can be asked for yet.
    pub fn is_busy(&self, now: f64) -> bool {
        now < (self.ready - ASK_AHEAD).max(self.asked_until)
    }

    /// Whether it is still winding up, so it can be called off.
    pub fn is_winding_up(&self, now: f64) -> bool {
        now < self.lands
    }
}
