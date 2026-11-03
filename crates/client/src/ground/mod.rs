//! Bags on the ground of a site: where the server says there are some, or where the
//! character remembers them, with a sparkle behind them while in sight. The grid of
//! what is in one is `inventory`'s. See `docs/context/client/ground.md`.

mod bag;

use bevy::prelude::*;

use crate::common::HoveredBag;
use crate::player::PlayerSystems;
use crate::view3d::View3dSystems;

// Split into files (`cargo xtask lint` keeps files small).
mod assets;
mod clicks;
#[cfg(not(target_arch = "wasm32"))]
mod debug;
mod parts;
mod server;
mod show;
mod spawn;

use assets::{make_assets, sparkle_scale, sparkle_up};
use clicks::{change_of_mind, find_hovered, follow_click, open_on_arrival};
#[cfg(not(target_arch = "wasm32"))]
pub use debug::GroundDebug;
#[cfg(not(target_arch = "wasm32"))]
use debug::run_debug;
use parts::{Bag, BagAssets, BagMeshes, BagPart, Bags, Heading, OfBag, Sparkle, Star};
use server::read_server;
use show::show_bags;
use spawn::set_bag;

/// A sparkle's main star: how long its points are, and how thin.
const STAR_LONG: f32 = 7.0;
const STAR_SHORT: f32 = 1.8;
/// A small glint beside it, as part of the main star's size.
const GLINT_SCALE: f32 = 0.4;
/// How big a sparkle is on the biggest bag, as part of those sizes; it shrinks with the bag.
const SPARKLE_SCALE: f32 = 0.5;
/// How far a star's outline stands out as drawn, always and while the pointer is on it,
/// so that shrinking doesn't lose it.
const STAR_OUTLINE: f32 = 0.2;
const STAR_HOVER_OUTLINE: f32 = 0.45;
/// How far a sparkle grows and shrinks, and how fast, by real time.
const TWINKLE: f32 = 0.18;
const TWINKLE_PER_SECOND: f32 = 2.2;
/// How fast it slowly turns.
const TURN_PER_SECOND: f32 = 0.35;
/// How far above the floor it never goes below, on the biggest bag.
const LOWEST_SPARKLE: f32 = 4.0;
/// A light gold, showing whatever the light.
const SPARKLE: Color = Color::srgb(1.0, 0.92, 0.55);
/// A star's outline.
const STAR_EDGE: Color = Color::BLACK;
/// How many pixels a side the weave's picture has.
const WEAVE_PIXELS: usize = 16;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Bags>()
            .init_resource::<HoveredBag>()
            .init_resource::<Heading>()
            .add_systems(Startup, make_assets)
            // After the steps there have gone, as a shop's door does.
            .add_systems(PostUpdate, open_on_arrival)
            .add_systems(
                Update,
                (
                    read_server,
                    find_hovered.in_set(View3dSystems::Click),
                    (change_of_mind, follow_click)
                        .chain()
                        .in_set(PlayerSystems::Input)
                        .after(View3dSystems::Click),
                    show_bags
                        .after(read_server)
                        .after(PlayerSystems::Travel)
                        .after(View3dSystems::Place),
                ),
            );
        #[cfg(not(target_arch = "wasm32"))]
        app.add_message::<GroundDebug>()
            .add_systems(Update, run_debug.before(read_server));
    }
}

#[cfg(test)]
mod tests;
