//! Lightning: a flash for a moment when the server says it strikes where the
//! player is (in the heaviest rain), or the debug menu's Lightning Now. The
//! views brighten what is under the sky by `common::Lightning`.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::common::{Lightning, LightningStrike, Preferences};
use crate::connection::FromServer;

mod envelope;

pub struct LightningPlugin;

impl Plugin for LightningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, flash);
    }
}

/// Every frame: starts a flash on a strike, and keeps `Lightning` to how
/// bright it is now, gentler and longer with Reduce Motion.
fn flash(
    time: Res<Time>,
    preferences: Res<Preferences>,
    mut from_server: MessageReader<FromServer>,
    mut strikes: MessageReader<LightningStrike>,
    mut lightning: ResMut<Lightning>,
    mut since: Local<Option<f32>>,
) {
    let told = from_server
        .read()
        .any(|FromServer(message)| matches!(message, ServerMessage::Lightning));
    if strikes.read().count() > 0 || told {
        *since = Some(0.0);
    }
    let gentle = preferences.video.reduce_motion;
    let strength = match *since {
        Some(seconds) => match envelope::strength(seconds, gentle) {
            Some(strength) => {
                *since = Some(seconds + time.delta_secs());
                strength
            }
            None => {
                *since = None;
                0.0
            }
        },
        None => 0.0,
    };
    lightning.set_if_neq(Lightning(strength));
}
