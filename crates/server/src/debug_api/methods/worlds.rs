//! `server.worlds`, and choosing which world another method looks at.

use bevy::prelude::*;
use serde::Deserialize;
use shared::world::WorldId;

use super::super::{BrpResult, Value, json, refused};
use crate::{Connections, ServerWorld, ServerWorlds};

/// Which world a method looks at: the home unless asked.
#[derive(Deserialize)]
pub(super) struct WorldParams {
    #[serde(default)]
    world: Option<uuid::Uuid>,
}

/// The world asked for, which must be open.
pub(super) fn chosen(worlds: &ServerWorlds, asked: WorldParams) -> BrpResult<&ServerWorld> {
    match asked.world {
        None => Ok(worlds.home()),
        Some(id) => worlds
            .get(WorldId(id))
            .ok_or_else(|| refused(format!("world {id} isn't open"))),
    }
}

/// `server.worlds`: every open world, with its time and who is in it.
pub(super) fn worlds(
    _: In<Option<Value>>,
    worlds: Res<ServerWorlds>,
    connections: Res<Connections>,
) -> BrpResult {
    let open: Vec<Value> = worlds
        .iter()
        .map(|(id, world)| {
            let players = connections
                .clients()
                .iter()
                .filter(|client| client.world == Some(id))
                .count();
            json!({
                "id": id.0,
                "home": id == worlds.home_id(),
                "minutes": world.minutes(),
                "rate": [world.time().rate.game, world.time().rate.real],
                "players": players,
                "sites": world.sites.len(),
                "levels_in_use": world.populations.0.len(),
            })
        })
        .collect();
    Ok(json!(open))
}
