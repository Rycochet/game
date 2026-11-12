//! Teleporting, entering sites and setting cells.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params, refused};
use shared::dungeon::Cell;
use shared::poi::PoiKind;
use shared::protocol::ClientMessage;

use super::done;
use crate::common::{GameView, MapPosition, in_map_measure};
use crate::connection::ToServer;
use crate::dev_tools::dev_area::dev_area_of;
use crate::player::Player;

#[derive(Deserialize)]
pub(super) struct Place {
    pub(super) x: i64,
    pub(super) y: i64,
}

/// `client.teleport { "x": 10, "y": -4 }`: on the world map, or onto a
/// walkable cell of the site level (its middle; as Teleport Mode,
/// `HOW_DEBUG_POS`).
pub(super) fn teleport(
    In(request): In<Option<Value>>,
    view: Res<State<GameView>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let Place { x, y } = params(request)?;
    let (x, y) = in_map_measure(*view.get(), (x, y));
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

#[derive(Deserialize)]
pub(super) struct DevAreaParams {
    size: Option<u16>,
    generator: Option<String>,
    mobs: Option<String>,
    count: Option<u8>,
    /// A number, or any words (hashed, as the panel does).
    seed: Option<Value>,
}

/// `client.dev_area { "size": 64, "generator": "Moat", "mobs": "Wolf", "count": 3,
/// "seed": "test" }`: makes a Dev Area and goes in (`HOW_DEBUG_DEV_AREA`); all
/// optional, as the panel starts. Its stairs lead back.
pub(super) fn dev_area(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let DevAreaParams {
        size,
        generator,
        mobs,
        count,
        seed,
    } = params(request)?;
    let seed = seed.map(|seed| match seed {
        Value::String(text) => text,
        other => other.to_string(),
    });
    let area = dev_area_of(
        size,
        generator.as_deref(),
        mobs.as_deref(),
        count,
        seed.as_deref(),
    )
    .map_err(|why| refused(&why))?;
    to_server.write(ToServer(ClientMessage::DevGenerateArea(area)));
    Ok(json!({ "size": area.size, "seed": area.seed }))
}
