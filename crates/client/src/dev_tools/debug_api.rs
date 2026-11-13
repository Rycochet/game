//! The client's debug API methods (only with the `debug-api` feature): what
//! the game is showing, screenshots, and what the `HOW_DEBUG_` variables do,
//! at any time, without restarting. They go through the same code as the
//! variables (`super`), and ask the server as the game does, so the server's
//! checks still apply. See `docs/context/debug-api.md`.

use bevy::prelude::*;
use server::debug_api::AddDebugMethod;

// Split into files (`cargo xtask lint` keeps files small).
mod burst;
mod drawing;
mod items;
mod login;
mod mobs;
mod moving;
mod panels;
mod places;
mod state;
mod view;
mod world;

use drawing::{fields_stage, herbs_shift, stairs_style, token};
use items::{give_item, set_creature, wear_item};
use moving::{step, travel, walk, yes};
use panels::{debug_action, debug_menu, inventory, map, show_floor, system_menu};
use places::{Place, dev_area, enter_site, set_cells, teleport};
use state::{done, screenshot, state};
use view::{cursor, interface, view, window_size};
use world::{celestial, reveal_pois, set_time};

pub(super) fn add(app: &mut App) {
    app.init_resource::<burst::Burst>()
        .add_systems(Last, burst::shoot)
        .add_debug_method("tmp.burst", burst::burst)
        .add_debug_method("client.state", state)
        .add_debug_method("client.screenshot", screenshot)
        // Asking the server.
        .add_debug_method("client.teleport", teleport)
        .add_debug_method("client.enter_site", enter_site)
        .add_debug_method("client.dev_area", dev_area)
        .add_debug_method("client.create_mob", mobs::create_mob)
        .add_debug_method("client.inspect_mob", mobs::inspect_mob)
        .add_debug_method("client.inspected", mobs::inspected)
        .add_debug_method("client.set_cells", set_cells)
        .add_debug_method("client.set_time", set_time)
        .add_debug_method("client.go_to_world", world::go_to_world)
        .add_debug_method("client.reveal_pois", reveal_pois)
        .add_debug_method("client.give_item", give_item)
        .add_debug_method("client.wear_item", wear_item)
        .add_debug_method("client.set_creature", set_creature)
        // Moving.
        .add_debug_method("client.step", step)
        .add_debug_method("client.travel", travel)
        .add_debug_method("client.walk", walk)
        // How things look.
        .add_debug_method("client.view", view)
        .add_debug_method("client.window_size", window_size)
        .add_debug_method("client.interface", interface)
        .add_debug_method("client.cursor", cursor)
        .add_debug_method("client.stairs_style", stairs_style)
        .add_debug_method("client.fields_stage", fields_stage)
        .add_debug_method("client.herbs_shift", herbs_shift)
        .add_debug_method("client.celestial", celestial)
        .add_debug_method("client.token", token)
        // What is open.
        .add_debug_method("client.map", map)
        .add_debug_method("client.show_floor", show_floor)
        .add_debug_method("client.debug_action", debug_action)
        .add_debug_method("client.debug_menu", debug_menu)
        .add_debug_method("client.system_menu", system_menu)
        .add_debug_method("client.login_screen", login::login_screen)
        .add_debug_method("client.inventory", inventory);
}
