//! Entry point for both the native app and the browser (WebAssembly) build.
//!
//! This file only creates the window and registers each feature module's
//! plugin. Game logic lives in the modules themselves.

mod common;
mod demo;
#[cfg(not(target_arch = "wasm32"))]
mod dev_tools;
mod player;
mod surface_style;
mod ui;
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
        surface_style::SurfaceStylePlugin,
        player::PlayerPlugin,
        world_map::WorldMapPlugin,
        demo::DemoPlugin,
        ui::UiPlugin,
    ));

    // Native only: the embedded server (single player) and developer tools.
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins((server::ServerPlugin, dev_tools::DevToolsPlugin));

    app.run();
}
