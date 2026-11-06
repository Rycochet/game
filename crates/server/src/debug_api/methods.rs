//! The server's own debug API methods.

use std::path::PathBuf;
use std::process;

use bevy::prelude::*;
use serde::Deserialize;
use shared::dungeon::{Cell, Level};

use super::{AddDebugMethod, BrpResult, Value, json, params, refused};
use crate::connections::{Client, save_all};
use crate::{Connections, ServerOptions, ServerWorlds};

mod worlds;

use worlds::{WorldParams, chosen};

/// Adds the server's methods. `data` is its data folder (none: saves nothing).
pub(crate) fn add_server_methods(app: &mut App, data: Option<PathBuf>) {
    app.add_debug_method(
        "server.state",
        move |_: In<Option<Value>>,
              worlds: Res<ServerWorlds>,
              connections: Res<Connections>,
              options: Res<ServerOptions>|
              -> BrpResult { Ok(state(&data, &worlds, &connections, &options)) },
    )
    .add_debug_method("server.mobs", mobs)
    .add_debug_method("server.cells", cells)
    .add_debug_method("server.sites", sites)
    .add_debug_method("server.worlds", worlds::worlds)
    .add_debug_method("server.save", save);
}

/// `server.state`: which server answered (process and data folder, so a
/// caller can tell it is talking to its own game), its clocks and who is
/// connected, with which world they are in and where.
fn state(
    data: &Option<PathBuf>,
    worlds: &ServerWorlds,
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
                "world": client.world.map(|id| id.0),
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
        "home_world": worlds.home_id().0,
        "server_clock": worlds.clock().seconds(),
        "clock_minutes": worlds.home().minutes(),
        "paused": worlds.clock().is_paused(),
        "action_ticks": worlds.action_clock().ticks(),
        "players": players,
    })
}

/// `server.mobs { "world": id }`: every mob on the levels in use of a world
/// (the home unless asked), by site and level: who it is, where, where its
/// home is and what it is doing.
fn mobs(In(request): In<Option<Value>>, worlds: Res<ServerWorlds>) -> BrpResult {
    let world = chosen(&worlds, params::<WorldParams>(request)?)?;
    let levels: Vec<Value> = world
        .populations
        .0
        .iter()
        .map(|((site, level), population)| {
            let mobs: Vec<Value> = population
                .mobs
                .iter()
                .map(|mob| {
                    json!({
                        "id": mob.id,
                        "kind": mob.kind,
                        "position": mob.position,
                        "home": mob.home,
                        "doing": mob.doing,
                        "target": mob.target,
                    })
                })
                .collect();
            json!({ "site": site, "level": level, "mobs": mobs })
        })
        .collect();
    Ok(json!(levels))
}

#[derive(Deserialize)]
struct CellsParams {
    /// Which connected player, in `server.state`'s order.
    #[serde(default)]
    player: usize,
    /// How far each way from them.
    #[serde(default = "default_radius")]
    radius: i64,
}

fn default_radius() -> i64 {
    12
}

/// `server.cells { "player": 0, "radius": 12 }`: the cells of a player's
/// level around them as the server has them (secret doors and all), one
/// string per row from the north, `@` for the player, with a legend.
fn cells(In(request): In<Option<Value>>, connections: Res<Connections>) -> BrpResult {
    let CellsParams { player, radius } = params(request)?;
    let visit = connections
        .clients()
        .get(player)
        .and_then(|client: &Client| client.visit.as_ref())
        .ok_or_else(|| refused(format!("player {player} isn't in a site")))?;
    let (x, y) = visit.position();
    let radius = radius.clamp(1, 100);
    let (rows, legend) = picture(visit.cells(), (x, y), radius);
    Ok(json!({
        "site": visit.site_id(),
        "level": visit.index(),
        "position": [x, y],
        "from": [x - radius, y - radius],
        "rows": rows,
        "legend": legend,
    }))
}

/// The cells within `radius` of `centre`, one string per row, and what each
/// letter means. Outside the level is blank.
fn picture(level: &Level, centre: (i64, i64), radius: i64) -> (Vec<String>, Value) {
    let mut legend = serde_json::Map::new();
    legend.insert("@".into(), json!("player"));
    let rows = (centre.1 - radius..=centre.1 + radius)
        .map(|y| {
            (centre.0 - radius..=centre.0 + radius)
                .map(|x| {
                    if (x, y) == centre {
                        '@'
                    } else if !level.contains(x, y) {
                        ' '
                    } else {
                        let cell = level.cell(x, y);
                        let mark = mark(cell);
                        legend
                            .entry(mark.to_string())
                            .or_insert_with(|| json!(format!("{cell:?}")));
                        mark
                    }
                })
                .collect()
        })
        .collect();
    (rows, Value::Object(legend))
}

/// One letter per kind of cell, the common ones as in the tests' pictures.
fn mark(cell: Cell) -> char {
    match cell {
        Cell::Floor => '.',
        Cell::Wall => '#',
        Cell::Door => '+',
        Cell::SecretDoor => 'S',
        Cell::Ground => ',',
        Cell::Water => '~',
        Cell::Lava => '^',
        Cell::Window => '=',
        Cell::Hole => 'O',
        Cell::CoveredGround => ';',
        Cell::DirtTrack => 'd',
        Cell::GravelTrack => 'g',
        Cell::WoodenBoards => 'w',
        Cell::Cobblestone => 'c',
        Cell::Flagstones => 'f',
        Cell::DressedStone => 's',
        Cell::Garden => '"',
        Cell::BuildingDoor => 'D',
        Cell::BlindWindow => 'b',
        Cell::Field => 'F',
        Cell::Fence => '|',
        Cell::FieldGate => 'G',
        Cell::HerbBed => 'h',
        Cell::WorldGate => 'W',
        Cell::Grass => '`',
        Cell::Tree => 'T',
        Cell::Shallows => '-',
        Cell::Bridge => 'B',
    }
}

/// `server.sites { "world": id }`: the sites of a world (the home unless
/// asked) loaded so far, with their generator version and the levels that
/// have saves.
fn sites(In(request): In<Option<Value>>, worlds: Res<ServerWorlds>) -> BrpResult {
    let world = chosen(&worlds, params::<WorldParams>(request)?)?;
    let sites: Vec<Value> = world
        .sites
        .iter()
        .map(|(id, save)| {
            json!({
                "site": id,
                "version": save.version,
                "levels": save.levels.keys().collect::<Vec<_>>(),
            })
        })
        .collect();
    Ok(json!(sites))
}

/// `server.save`: saves everything changed now, and answers once it is
/// written (within a few seconds), so a caller can read the files.
fn save(
    _: In<Option<Value>>,
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
) -> BrpResult {
    save_all(&mut connections, &mut worlds);
    Ok(json!({ "written": worlds.home().storage.flush() }))
}

#[cfg(test)]
mod tests;
