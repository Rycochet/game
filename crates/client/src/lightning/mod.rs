//! Lightning: a flash for a moment when the server says it strikes where the
//! player is (in the heaviest rain), or the debug menu's Lightning Now. The
//! views brighten what is under the sky by `common::Lightning`, and draw the
//! bolts it tells of (`common::BoltStruck`) with `spawn_bolt`.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::common::{BoltStruck, Lightning, LightningStrike, Preferences};
use crate::connection::FromServer;

mod bolt;
mod envelope;
mod shape;

pub use bolt::spawn_bolt;

pub struct LightningPlugin;

impl Plugin for LightningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (flash, bolt::fade_bolts));
    }
}

/// Every frame: starts a flash on a strike, and keeps `Lightning` to how
/// bright it is now, gentler and longer with Reduce Motion.
fn flash(
    time: Res<Time>,
    preferences: Res<Preferences>,
    mut from_server: MessageReader<FromServer>,
    mut strikes: MessageReader<LightningStrike>,
    mut bolts: MessageWriter<BoltStruck>,
    mut lightning: ResMut<Lightning>,
    mut since: Local<Option<f32>>,
) {
    let mut told = false;
    for FromServer(message) in from_server.read() {
        if let ServerMessage::Lightning { flash, strike } = message {
            told |= *flash;
            if let Some(strike) = strike {
                bolts.write(BoltStruck(*strike));
            }
        }
    }
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
