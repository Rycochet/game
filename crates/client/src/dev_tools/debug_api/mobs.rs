//! Making mobs: what Create Mob's Mob Mode does with a click.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params, refused};
use shared::creature::CreatureKind;
use shared::protocol::{ClientMessage, DevMob};
use shared::sheet::Feeling;

use super::done;
use crate::connection::ToServer;

#[derive(Deserialize)]
pub(super) struct CreateMobParams {
    /// By name, as `CreatureKind` is saved: `"Goblin"`, `"Wolf"`, ...
    kind: CreatureKind,
    x: i64,
    y: i64,
    /// `"Friendly"`, `"Neutral"` or `"Unfriendly"`; the kind's own if missing.
    attitude: Option<Feeling>,
    #[serde(default = "yes")]
    gear: bool,
    #[serde(default)]
    still: bool,
    #[serde(default = "super::places::one")]
    seed: u64,
}

fn yes() -> bool {
    true
}

/// `client.create_mob { "kind": "Goblin", "x": 5, "y": 16, "attitude": "Friendly",
/// "gear": true, "still": false, "seed": 1 }`: puts a mob made to order on a
/// cell of the site level, as a click in Mob Mode does. The server refuses
/// walls, cells someone is on and the wilderness.
pub(super) fn create_mob(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let p: CreateMobParams = params(request)?;
    if p.attitude == Some(Feeling::None) {
        return Err(refused("attitude: Friendly, Neutral or Unfriendly"));
    }
    to_server.write(ToServer(ClientMessage::DevCreateMob(DevMob {
        kind: p.kind,
        x: p.x,
        y: p.y,
        attitude: p.attitude,
        gear: p.gear,
        still: p.still,
        seed: p.seed,
    })));
    done()
}
