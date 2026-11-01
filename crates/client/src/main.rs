//! Entry point for both the native app and the browser (WebAssembly) build.
//!
//! This file only creates the window and registers each feature module's
//! plugin. Game logic lives in the modules themselves.

mod common;
mod connection;
mod demo;
#[cfg(not(target_arch = "wasm32"))]
mod dev_tools;
mod dungeon_map;
mod minimap;
mod player;
mod site;
mod surface_style;
mod touch;
mod ui;
mod view3d;
mod widgets;
mod world_map;

use bevy::prelude::*;

/// Where Bevy loads assets from. Native debug builds use the source folder
/// (an absolute path fixed at compile time), so `target/debug/hall-of-worlds.exe`
/// finds them however it is started. Otherwise `assets` is next to the
/// executable (native) or the page (web, copied there by Trunk).
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
const ASSET_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
#[cfg(not(all(debug_assertions, not(target_arch = "wasm32"))))]
const ASSET_FOLDER: &str = "assets";

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
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
                file_path: ASSET_FOLDER.into(),
                ..default()
            }),
    )
    .add_plugins((
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
        demo::DemoPlugin,
        minimap::MinimapPlugin,
        ui::UiPlugin,
    ));

    // Native only: developer tools. (The embedded single-player server is
    // started by `connection`.)
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(dev_tools::DevToolsPlugin);

    app.run();
}
