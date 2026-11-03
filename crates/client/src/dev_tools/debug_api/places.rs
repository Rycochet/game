//! Teleporting, entering sites and setting cells.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params, refused};
use shared::dungeon::Cell;
use shared::poi::PoiKind;
use shared::protocol::ClientMessage;

use super::done;
use crate::common::{GameView, MapPosition};
use crate::connection::ToServer;
use crate::player::Player;

#[derive(Deserialize)]
pub(super) struct Place {
    pub(super) x: i64,
    pub(super) y: i64,
}

/// `client.teleport { "x": 10, "y": -4 }`: on the world map, or onto a
/// walkable cell of the site level (as Teleport Mode, `HOW_DEBUG_POS`).
pub(super) fn teleport(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let Place { x, y } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
    done()
}

#[derive(Deserialize)]
pub(super) struct EnterSiteParams {
    /// By name, as `PoiKind` is saved: `"Castle"`, `"RockCave"`, ...
    kind: PoiKind,
    #[serde(default = "one")]
    seed: u64,
    #[serde(default)]
    level: u32,
    #[serde(default)]
    ruin: u8,
}

pub(super) fn one() -> u64 {
    1
}

/// `client.enter_site { "kind": "Castle", "seed": 1, "level": 0, "ruin": 0 }`:
/// a made-up site where the player stands (`HOW_DEBUG_ENTER`).
pub(super) fn enter_site(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let EnterSiteParams {
        kind,
        seed,
        level,
        ruin,
    } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevEnterSite {
        kind,
        seed,
        level,
        ruin,
    }));
    done()
}

#[derive(Deserialize)]
pub(super) struct CellChange {
    dx: i64,
    dy: i64,
    /// By name: `"Wall"`, `"SecretDoor"`, `"Water"`, ...
    cell: Cell,
}

#[derive(Deserialize)]
pub(super) struct SetCellsParams {
    cells: Vec<CellChange>,
}

/// `client.set_cells { "cells": [{ "dx": 1, "dy": 0, "cell": "Wall" }] }`:
/// changes cells of the site level, from where the player stands
/// (`HOW_DEBUG_CELLS`).
pub(super) fn set_cells(
    In(request): In<Option<Value>>,
    view: Res<State<GameView>>,
    player: Single<&MapPosition, With<Player>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let SetCellsParams { cells } = params(request)?;
    if *view.get() != GameView::Dungeon {
        return Err(refused("only in a site"));
    }
    for CellChange { dx, dy, cell } in cells {
        let (x, y) = (player.x + dx, player.y + dy);
        to_server.write(ToServer(ClientMessage::DevSetCell { x, y, cell }));
    }
    done()
}
