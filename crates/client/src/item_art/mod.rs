//! Pictures of items, drawn once in 3D, a page of them at a time, and kept as images.
//! See `docs/context/client/item_art.md`.

mod armour;
mod blade;
mod body;
#[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
mod debug_api;
mod gem;
mod looks;
mod ring;

use bevy::prelude::*;
use bevy::render::{ExtractSchedule, Render, RenderApp, RenderSystems};

pub use looks::{
    ArmsLook, BladeLook, HandsLook, HeadLook, LegsLook, ShouldersLook, TorsoLook, WaistLook,
};

// Split into files (`cargo xtask lint` keeps files small).
mod batch;
mod copy;
mod draw;
mod drawing;
mod edge;
mod look;
mod parts;
mod pictures;
mod samples;
mod scale;
mod sheet;
mod spawn;

use batch::Batch;
use copy::{PageCopies, copy_pictures, extract_copies};
use draw::{draw_pictures, fill_pictures};
use drawing::{Part, PictureLight, StillCompiling, count_compiling};
use edge::EdgeMaterial;
pub use look::{Look, RingLook, look_of};
pub use pictures::{ItemPicture, ItemPictures};
use scale::follow_interface_scale;
use spawn::{spawn_batch, spawn_cell, spawn_lights};

/// Render layers kept for this start here, past any the views use: two for each
/// page, the models and their edges.
const FIRST_LAYER: usize = 16;
/// Pages that may be drawn at once.
const PAGES: usize = 4;
/// Width and height of a page of pictures, unless one needs more.
const PAGE_PIXELS: u32 = 1024;
/// A page's render layers, for its models and their edges.
fn layers(slot: usize) -> (usize, usize) {
    (FIRST_LAYER + slot * 2, FIRST_LAYER + slot * 2 + 1)
}

/// Frames a page's cameras stay once nothing is compiling, to be sure of it being
/// drawn whole: something new to draw only starts compiling after a frame. Its
/// pictures are copied off in each of them but the last.
const FRAMES_ALIVE: u8 = 4;
/// How much of the world a picture shows, across: a model reaches about 1 each way.
const VIEW_WIDTH: f32 = 2.5;
/// Looking down at a slant, the way a thing lies on a table.
const CAMERA_AT: Vec3 = Vec3::new(0.0, 3.0, 1.0);
const CAMERA_UP: Vec3 = Vec3::NEG_Z;
/// How thick the black edge round every picture is, in its own pixels: pictures are
/// drawn at twice the size they are shown.
const OUTLINE_PIXELS: f32 = 4.0;
/// The most times over the pixels asked for a picture is drawn with, for a large
/// interface (see `ItemPictures::detail`).
const MOST_DETAIL: u32 = 2;
/// The screen's pixels to an interface pixel that pictures as asked for are sharp up
/// to: they are asked for at twice the size they are shown.
const SHARP_UP_TO: f32 = 2.0;

pub struct ItemArtPlugin;

impl Plugin for ItemArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<EdgeMaterial>::default())
            .init_resource::<ItemPictures>()
            .init_resource::<StillCompiling>()
            .init_resource::<PageCopies>()
            .add_systems(Startup, samples::warm_up)
            .add_systems(
                PostUpdate,
                (follow_interface_scale, fill_pictures, draw_pictures).chain(),
            );
        #[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
        debug_api::add(app);
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(ExtractSchedule, (count_compiling, extract_copies))
                .add_systems(
                    Render,
                    copy_pictures
                        .after(RenderSystems::Render)
                        .before(RenderSystems::Cleanup),
                );
        }
    }
}

#[cfg(test)]
mod tests;
