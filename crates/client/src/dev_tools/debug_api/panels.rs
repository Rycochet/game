//! Opening the full map, floors, debug actions and menus, and the inventory.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params, refused};

use super::super::open_map;
use super::{done, yes};
use crate::common::{MapPosition, OtherLevels};
use crate::debug_menu::{DebugAction, DebugMenu};
use crate::inventory::InventoryDebug;
use crate::minimap::FullMap;
use crate::player::Player;
use crate::system_menu::SystemMenu;

#[derive(Deserialize)]
pub(super) struct MapParams {
    #[serde(default = "yes")]
    open: bool,
    #[serde(default)]
    dx: i64,
    #[serde(default)]
    dy: i64,
    zoom: Option<f32>,
}

/// `client.map { "dx": 0, "dy": 0, "zoom": 2 }`: opens the full map this far
/// from the player, without the limits on dragging (`HOW_DEBUG_MAP`);
/// `{ "open": false }` closes it.
pub(super) fn map(
    In(request): In<Option<Value>>,
    player: Single<&MapPosition, With<Player>>,
    mut map: ResMut<FullMap>,
) -> BrpResult {
    let MapParams { open, dx, dy, zoom } = params(request)?;
    if open {
        open_map(&mut map, &player, (dx, dy), zoom);
    } else {
        map.open = false;
    }
    done()
}

#[derive(Deserialize)]
pub(super) struct FloorParams {
    /// An index among the known floors, from 0; none for the player's own.
    floor: Option<u32>,
}

/// `client.show_floor { "floor": 1 }`: shows that known floor on the full
/// map, which must be open (`HOW_DEBUG_SHOW_FLOOR`).
pub(super) fn show_floor(
    In(request): In<Option<Value>>,
    mut levels: ResMut<OtherLevels>,
) -> BrpResult {
    let FloorParams { floor } = params(request)?;
    match floor {
        Some(floor) => levels.show(floor),
        None => levels.show_own(),
    }
    Ok(json!({ "showing": levels.showing() }))
}

#[derive(Deserialize)]
pub(super) struct ActionParams {
    /// As the menu shows it, in any case: `"God Mode"`, `"Truesight"`, ...
    name: String,
}

/// `client.debug_action { "name": "God Mode" }`: as choosing the entry in
/// the debug menu (`HOW_DEBUG_PANEL` for the panels). It isn't checked for
/// being available, but the server still refuses what it doesn't allow.
pub(super) fn debug_action(
    In(request): In<Option<Value>>,
    mut actions: MessageWriter<DebugAction>,
) -> BrpResult {
    let ActionParams { name } = params(request)?;
    let action = DebugAction::named(&name).ok_or_else(|| {
        let names: Vec<&str> = DebugAction::names().collect();
        refused(format!("no entry {name:?}; there are {}", names.join(", ")))
    })?;
    actions.write(action);
    done()
}

#[derive(Deserialize)]
pub(super) struct MenuParams {
    #[serde(default = "yes")]
    open: bool,
    #[serde(default)]
    filter: String,
}

/// `client.debug_menu { "filter": "tele" }`: opens it with that typed
/// (`HOW_DEBUG_MENU`); `{ "open": false }` closes it.
pub(super) fn debug_menu(In(request): In<Option<Value>>, mut menu: ResMut<DebugMenu>) -> BrpResult {
    let MenuParams { open, filter } = params(request)?;
    if open {
        *menu = DebugMenu::open_with(filter);
    } else {
        menu.open = false;
    }
    done()
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SystemMenuShown {
    Closed,
    Menu,
    Quit,
    Settings,
}

#[derive(Deserialize)]
pub(super) struct SystemMenuParams {
    show: SystemMenuShown,
}

/// `client.system_menu { "show": "settings" }`: `closed`, `menu`, `quit` (its
/// requester) or `settings` (`HOW_DEBUG_SYSTEM_MENU`).
pub(super) fn system_menu(
    In(request): In<Option<Value>>,
    mut menu: ResMut<SystemMenu>,
) -> BrpResult {
    let SystemMenuParams { show } = params(request)?;
    *menu = match show {
        SystemMenuShown::Closed => SystemMenu::Closed,
        SystemMenuShown::Menu => SystemMenu::Menu,
        SystemMenuShown::Quit => SystemMenu::Confirming,
        SystemMenuShown::Settings => SystemMenu::Settings,
    };
    done()
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum InventoryShown {
    /// The Character panel.
    Inventory,
    /// Every bag in the inventory.
    Bags,
    /// Holds the item at `index` in the inventory.
    Hold,
    /// All but the first of the inventory onto the ground.
    DropSome,
}

#[derive(Deserialize)]
pub(super) struct InventoryParams {
    show: InventoryShown,
    #[serde(default)]
    index: usize,
}

/// `client.inventory { "show": "inventory" }`: `inventory`, `bags`, `hold` (with
/// `index`) or `drop_some`, as `HOW_DEBUG_INVENTORY` does.
pub(super) fn inventory(
    In(request): In<Option<Value>>,
    mut debug: MessageWriter<InventoryDebug>,
) -> BrpResult {
    let InventoryParams { show, index } = params(request)?;
    debug.write(match show {
        InventoryShown::Inventory => InventoryDebug::OpenInventory,
        InventoryShown::Bags => InventoryDebug::OpenBags,
        InventoryShown::Hold => InventoryDebug::PickUp(index),
        InventoryShown::DropSome => InventoryDebug::DropSome,
    });
    done()
}
