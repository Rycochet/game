//! The game's icon on its window (native): the stylised HW drawn by
//! `cargo xtask app-icon`. The program file gets the same icon from
//! `build.rs`, and the web page from `index.html`'s favicons.

#[cfg(not(target_arch = "wasm32"))]
mod set;

use bevy::prelude::*;

pub struct AppIconPlugin;

impl Plugin for AppIconPlugin {
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, set::set_window_icon);
    }
}
