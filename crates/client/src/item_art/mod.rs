//! Pictures of items, drawn once in 3D by a camera of their own and kept as images.
//! See `docs/context/client/item_art.md`.

mod armour;
mod blade;
mod body;
mod gem;
mod looks;
mod ring;

use bevy::prelude::*;
use bevy::render::{ExtractSchedule, RenderApp};

pub use looks::{ArmsLook, BladeLook, HandsLook, HeadLook, ShouldersLook, TorsoLook};

// Split into files (`cargo xtask lint` keeps files small).
mod draw;
mod drawing;
mod look;
mod pictures;
mod scale;
mod spawn;

use draw::{draw_pictures, fill_pictures};
use drawing::{Drawing, Part, PictureLight, StillCompiling, count_compiling};
pub use look::{Look, RingLook, look_of};
pub use pictures::{ItemPicture, ItemPictures};
use scale::follow_interface_scale;
use spawn::{outline_copies, spawn_drawing, spawn_lights};

/// Render layers kept for this, past any the views use, one per picture being drawn.
const FIRST_LAYER: usize = 16;
const SLOTS: usize = 8;
/// Frames a picture's camera stays once nothing is compiling, to be sure of being
/// drawn once: something new to draw only starts compiling after a frame.
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
/// How many black copies make the edge: enough that it looks round.
const OUTLINE_COPIES: usize = 16;
/// How far behind a model its black copies go, to be behind every part of it.
const OUTLINE_BEHIND: f32 = 3.0;

pub struct ItemArtPlugin;

impl Plugin for ItemArtPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemPictures>()
            .init_resource::<StillCompiling>()
            .add_systems(
                PostUpdate,
                (follow_interface_scale, fill_pictures, draw_pictures).chain(),
            );
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(ExtractSchedule, count_compiling);
        }
    }
}

#[cfg(test)]
mod tests;
