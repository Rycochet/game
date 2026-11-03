//! Starting places: where the player starts, entering a site, cells changed on arriving.

use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::poi::PoiKind;
use shared::protocol::ClientMessage;
use std::env;

use super::{Start, parse_cell, parse_pos};
use crate::common::{GameView, MapPosition};
use crate::connection::ToServer;
use crate::player::Player;

/// `HOW_DEBUG_POS`, `HOW_DEBUG_LOADING_HOLD`, `HOW_DEBUG_REVEAL_POIS`, `HOW_DEBUG_ENTER` and `HOW_DEBUG_CELLS`.
pub(super) fn add(app: &mut App) {
    if let Some((x, y)) = env::var("HOW_DEBUG_POS")
        .ok()
        .and_then(|pos| parse_pos(&pos))
    {
        app.add_systems(
            OnExit(GameView::Connecting),
            (move |mut to_server: MessageWriter<ToServer>| {
                to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
            })
            .in_set(Start::Place),
        );
    }

    if server::option_on("HOW_DEBUG_LOADING_HOLD", false) {
        app.insert_resource(crate::loading_screen::HoldLoadingScreen(true));
    }

    if server::option_on("HOW_DEBUG_REVEAL_POIS", false) {
        app.add_systems(
            OnExit(GameView::Connecting),
            (|mut to_server: MessageWriter<ToServer>| {
                to_server.write(ToServer(ClientMessage::DevRevealPois));
            })
            .in_set(Start::Enter),
        );
    }

    if let Ok(enter) = env::var("HOW_DEBUG_ENTER") {
        let numbers: Vec<u64> = enter
            .split(',')
            .filter_map(|n| n.trim().parse().ok())
            .collect();
        let kind = numbers
            .first()
            .and_then(|&k| PoiKind::ALL.get(k as usize).copied())
            .unwrap_or(PoiKind::Castle);
        let seed = numbers.get(1).copied().unwrap_or(1);
        let level = numbers.get(2).copied().unwrap_or(0) as u32;
        let ruin = numbers.get(3).copied().unwrap_or(0) as u8;
        app.add_systems(
            OnExit(GameView::Connecting),
            (move |mut to_server: MessageWriter<ToServer>| {
                // Where the player is (after `HOW_DEBUG_POS`), as its way out.
                to_server.write(ToServer(ClientMessage::DevEnterSite {
                    kind,
                    seed,
                    level,
                    ruin,
                }));
            })
            .in_set(Start::Enter),
        );
    }

    if let Ok(cells) = env::var("HOW_DEBUG_CELLS") {
        let cells: Vec<(i64, i64, Cell)> = cells.split(';').filter_map(parse_cell).collect();
        app.add_systems(
            Update,
            move |view: Res<State<GameView>>,
                  mut done: Local<bool>,
                  player: Single<&MapPosition, With<Player>>,
                  mut to_server: MessageWriter<ToServer>| {
                if !*done && view.get() == &GameView::Dungeon {
                    *done = true;
                    for &(dx, dy, cell) in &cells {
                        let (x, y) = (player.x + dx, player.y + dy);
                        to_server.write(ToServer(ClientMessage::DevSetCell { x, y, cell }));
                    }
                }
            },
        );
    }
}
