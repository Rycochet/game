//! Entry point for both the native app and the browser (WebAssembly) build.
//!
//! This file only creates the window and registers each feature module's
//! plugin. Game logic lives in the modules themselves.

mod character;
mod common;
mod connection;
mod debug_menu;
#[cfg(not(target_arch = "wasm32"))]
mod dev_tools;
mod dungeon_map;
mod entities;
mod fonts;
mod ground;
mod hall_view;
mod inventory;
mod item_art;
mod loading_screen;
mod minimap;
mod planet_view;
mod player;
mod site;
mod surface_style;
mod system_menu;
mod touch;
mod trade;
mod ui;
mod view3d;
mod widgets;
mod world_map;

use bevy::prelude::*;

/// Where Bevy loads assets from. Native debug builds use an `assets` folder
/// beside the executable if there is one (a copy made by `cargo xtask latest`),
/// and otherwise the source folder (an absolute path fixed at compile time), so
/// the debug executable finds them however and wherever it is started.
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
fn asset_folder() -> String {
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("assets")))
        .filter(|folder| folder.is_dir());
    match beside {
        Some(folder) => folder.to_string_lossy().into_owned(),
        None => concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string(),
    }
}

/// Where Bevy loads assets from: `assets` next to the executable (native) or
/// the page (web, copied there by Trunk).
#[cfg(not(all(debug_assertions, not(target_arch = "wasm32"))))]
fn asset_folder() -> String {
    "assets".to_string()
}

fn main() {
    let plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hall of Worlds".into(),
                // In the browser, draw into the <canvas id="game"> in index.html.
                canvas: Some("#game".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        })
        .set(AssetPlugin {
            file_path: asset_folder(),
            // There are no `.meta` files, so don't look for one beside every asset
            // (on the web, each look is a request that fails with a 404).
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        });
    // The web has no environment to read `LOG_LEVEL` from.
    #[cfg(not(target_arch = "wasm32"))]
    let plugins = plugins.set(server::log_plugin());

    let mut app = App::new();
    app.add_plugins(plugins).add_plugins((
        fonts::FontsPlugin,
        common::CommonPlugin,
        connection::ConnectionPlugin,
        surface_style::SurfaceStylePlugin,
        touch::TouchPlugin,
        widgets::WidgetsPlugin,
        player::PlayerPlugin,
        view3d::View3dPlugin,
        world_map::WorldMapPlugin,
        dungeon_map::DungeonMapPlugin,
        site::SitePlugin,
        debug_menu::DebugMenuPlugin,
        minimap::MinimapPlugin,
        ui::UiPlugin,
        loading_screen::LoadingScreenPlugin,
    ));
    // (Apart: a tuple of plugins can only be so long.)
    app.add_plugins((
        system_menu::SystemMenuPlugin,
        inventory::InventoryPlugin,
        character::CharacterPlugin,
        entities::EntitiesPlugin,
        ground::GroundPlugin,
        trade::TradePlugin,
        item_art::ItemArtPlugin,
        planet_view::PlanetViewPlugin,
        hall_view::HallViewPlugin,
    ));

    // Native only: developer tools. (The embedded single-player server is
    // started by `connection`.)
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins((
        dev_tools::DevToolsPlugin,
        // Started by `HOW_DEBUG_API`, if this build has it.
        server::debug_api::DebugApiPlugin { port_offset: 0 },
    ));

    app.run();
}
