//! View Planet: the whole world as a turning globe with its moons, at a day a
//! minute, over everything until a click, tap or Esc. A debug view for now, a
//! loading screen for travelling between planets later. See
//! `docs/context/client/planet_view.md`.

mod bar;
mod close;
mod clouds;
mod colour;
#[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
mod debug_api;
mod gates;
mod globe;
mod input;
mod layer;
mod moon_look;
mod open;
mod planet;
mod read_bar;
mod relief;
mod showing;
mod sky;
mod spin;
mod texture;

use bevy::prelude::*;

use crate::widgets::WidgetSystems;

/// Render layer of the globe, its moons, lights and camera: drawn only into
/// the view's own picture.
const PLANET_LAYER: usize = 8;

/// Game hours that pass each real second: a day a minute.
const HOURS_PER_SECOND: f64 = 24.0 / 60.0;

/// Shows the planet, as the debug menu's View Planet does.
#[derive(Message, Clone, Copy, Debug)]
pub struct ViewPlanet;

pub struct PlanetViewPlugin;

impl Plugin for PlanetViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<showing::PlanetView>()
            .add_plugins(MaterialPlugin::<clouds::PlanetCloudMaterial>::default())
            .add_message::<ViewPlanet>()
            .add_systems(
                PreUpdate,
                input::take_input
                    .after(WidgetSystems::Presses)
                    .before(WidgetSystems::Front),
            )
            .add_systems(
                Update,
                (
                    open::open,
                    read_bar::read_bar,
                    gates::show_gate_words,
                    texture::fill_texture,
                    spin::spin,
                    clouds::move_clouds,
                    close::close_without_world,
                )
                    .chain(),
            );
        #[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
        debug_api::add(app);
    }
}

#[cfg(test)]
mod tests;
