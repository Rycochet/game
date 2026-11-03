//! The server's own debug API methods.

use std::path::PathBuf;
use std::process;

use bevy::prelude::*;

use super::{AddDebugMethod, BrpResult, Value, json};
use crate::{Connections, ServerOptions, ServerWorld};

/// Adds the server's methods. `data` is its data folder (none: saves nothing).
pub(crate) fn add_server_methods(app: &mut App, data: Option<PathBuf>) {
    app.add_debug_method(
        "server.state",
        move |_: In<Option<Value>>,
              world: Res<ServerWorld>,
              connections: Res<Connections>,
              options: Res<ServerOptions>|
              -> BrpResult { Ok(state(&data, &world, &connections, &options)) },
    );
}

/// `server.state`: which server answered (process and data folder, so a
/// caller can tell it is talking to its own game), its clock and who is
/// connected, with where they are.
fn state(
    data: &Option<PathBuf>,
    world: &ServerWorld,
    connections: &Connections,
    options: &ServerOptions,
) -> Value {
    let players: Vec<Value> = connections
        .clients()
        .iter()
        .map(|client| {
            json!({
                "character": client.character_id,
                "phase": format!("{:?}", client.phase),
                "world_position": client.character.world_position,
                "visit": client.visit.as_ref().map(|visit| json!({
                    "site": visit.site_id(),
                    "level": visit.index(),
                    "position": visit.position(),
                })),
            })
        })
        .collect();
    json!({
        "program": if options.exit_when_alone { "embedded server" } else { "server" },
        "pid": process::id(),
        "data": data,
        "dev_tools": options.dev_tools,
        "clock_minutes": world.clock.minutes(),
        "paused": world.clock.is_paused(),
        "players": players,
    })
}
