//! Opening panels, menus and the full map on starting.

use bevy::prelude::*;
use std::env;

use super::parse_pos;
use crate::common::{FirstFloor, MapPosition, OtherLevels};
use crate::debug_menu::{DebugAction, DebugMenu};
use crate::minimap::FullMap;
use crate::player::Player;
use crate::system_menu::SystemMenu;

/// `HOW_DEBUG_PANEL`, `HOW_DEBUG_MENU`, `HOW_DEBUG_SYSTEM_MENU`, `HOW_DEBUG_FIRST_FLOOR`, `HOW_DEBUG_SHOW_FLOOR` and `HOW_DEBUG_MAP`.
pub(super) fn add(app: &mut App) {
    if let Ok(panel) = env::var("HOW_DEBUG_PANEL") {
        let action = if panel == "suns" {
            DebugAction::SunsAndMoons
        } else if panel == "pictures" {
            DebugAction::ItemPictures
        } else if panel == "generate" {
            DebugAction::GeneratePoi
        } else {
            DebugAction::ShowWidgets
        };
        app.add_systems(
            Update,
            move |mut actions: MessageWriter<DebugAction>, mut sent: Local<bool>| {
                if !*sent {
                    *sent = true;
                    actions.write(action);
                }
            },
        );
    }

    if let Ok(filter) = env::var("HOW_DEBUG_MENU") {
        app.insert_resource(DebugMenu::open_with(filter));
    }

    match env::var("HOW_DEBUG_SYSTEM_MENU").as_deref() {
        Ok("menu") => app.insert_resource(SystemMenu::Menu),
        Ok("quit") => app.insert_resource(SystemMenu::Confirming),
        Ok("settings") => app.insert_resource(SystemMenu::Settings),
        _ => app,
    };

    if let Some(first) = env::var("HOW_DEBUG_FIRST_FLOOR")
        .ok()
        .and_then(|n| n.parse().ok())
    {
        app.insert_resource(FirstFloor(first));
    }

    if let Some(floor) = env::var("HOW_DEBUG_SHOW_FLOOR")
        .ok()
        .and_then(|n| n.parse().ok())
    {
        app.add_systems(
            Update,
            move |time: Res<Time>, mut done: Local<bool>, mut levels: ResMut<OtherLevels>| {
                if !*done && time.elapsed_secs() > 3.0 {
                    *done = true;
                    levels.show(floor);
                }
            },
        );
    }

    if let Some((dx, dy)) = env::var("HOW_DEBUG_MAP")
        .ok()
        .and_then(|pos| parse_pos(&pos))
    {
        let zoom = env::var("HOW_DEBUG_MAP_ZOOM")
            .ok()
            .and_then(|n| n.parse().ok());
        app.add_systems(
            Update,
            move |time: Res<Time>,
                  mut done: Local<bool>,
                  player: Single<&MapPosition, With<Player>>,
                  mut map: ResMut<FullMap>| {
                if !*done && time.elapsed_secs() > 2.0 {
                    *done = true;
                    open_map(&mut map, &player, (dx, dy), zoom);
                }
            },
        );
    }
}

/// Opens the full map, centred `offset` grid points from the player, at
/// `zoom` if given.
pub(super) fn open_map(
    map: &mut FullMap,
    player: &MapPosition,
    offset: (i64, i64),
    zoom: Option<f32>,
) {
    map.open = true;
    map.centre = Some(((player.x + offset.0) as f64, (player.y + offset.1) as f64));
    if let Some(zoom) = zoom {
        map.zoom = zoom;
    }
}
