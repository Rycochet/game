//! The Hall of Worlds game server: the authoritative simulation. See
//! `docs/context/server/server.md`.
//!
//! Native only. It runs either as the standalone `hall-of-worlds-server`
//! binary (dedicated / Docker) or embedded in a native client for single
//! player. The web client never includes it.
//!
//! For now it only starts up; it gains the authoritative game state and
//! command validation when networking is added (see
//! `docs/context/architecture.md`).

use bevy::prelude::*;

/// Simulation ticks per second for the standalone server.
pub const TICKS_PER_SECOND: f64 = 20.0;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, announce);
    }
}

fn announce() {
    info!(
        "Hall of Worlds server {} started (no networking yet)",
        env!("CARGO_PKG_VERSION")
    );
}
