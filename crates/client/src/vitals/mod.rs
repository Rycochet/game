//! The player's Health and Mana: a circle at the bottom middle of the window,
//! red Health on the left and blue Mana on the right, filling from the bottom.
//! See `docs/context/client/vitals.md`.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::protocol::ServerMessage;

use crate::common::{OwnVitals, ToldVitals};
use crate::connection::FromServer;
use crate::dom::{Template, bottom_strip, document};
use crate::html;
use crate::widgets::UiId;

mod killed;
mod paint;
mod show;

/// The circle across, as a share of the window's height: half the minimap's
/// (owner, 2026-11-06).
const ACROSS_OF_HEIGHT: f32 = 0.1;
/// The gap under it, in interface pixels (as the markup has it).
const BOTTOM_GAP: f32 = 8.0;
/// The circle's id.
const BALL: &str = "vitals/ball";

/// The circle: a place its picture is painted into, above the bottom of the
/// window, hidden until the server says what it holds. `show_circle` keeps
/// its size, its `left` (the middle of the window) and its tip (`title`).
/// It has no form round it: a form here has no height, and the pointer is
/// never on what is outside its form.
static VITALS: Template = html!(
    "vitals",
    r#"
<canvas id="ball" form="vitals" bottom="8" left="0" width="1" height="1" hidden/>
"#
);

pub struct VitalsPlugin;

impl Plugin for VitalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_circle, killed::spawn_killed))
            .add_systems(
                Update,
                (bind_circle, take_vitals, show::show_circle).chain(),
            )
            .add_systems(Update, killed::show_killed);
    }
}

/// Its picture, and what it was last painted with: its side in screen pixels
/// and how full each half is, in steps finer than a pixel.
#[derive(Resource)]
struct CirclePicture {
    image: Handle<Image>,
    painted: Option<(u32, u32, u32)>,
}

fn spawn_circle(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    commands.insert_resource(CirclePicture {
        image,
        painted: None,
    });
    // Along the bottom of the window, for the circle to be placed from.
    commands.spawn(document(&VITALS)).insert(bottom_strip());
}

/// Gives the circle, once built, its picture. It is an element, so presses
/// on it go no further: not walking, nor (a right press) clearing the target.
fn bind_circle(
    new: Query<(Entity, &UiId), Added<UiId>>,
    picture: Res<CirclePicture>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        if id.parts() == ("vitals", "ball") {
            let image = ImageNode::new(picture.image.clone());
            commands.entity(entity).insert(image);
        }
    }
}

/// Keeps what the server says of the player's own Health and Mana.
fn take_vitals(
    mut from_server: MessageReader<FromServer>,
    time: Res<Time>,
    mut own: ResMut<OwnVitals>,
) {
    for FromServer(message) in from_server.read() {
        if let ServerMessage::Vitals {
            who: None,
            health,
            mana,
        } = message
        {
            own.0 = Some(ToldVitals {
                health: *health,
                mana: *mana,
                at: time.elapsed_secs_f64(),
            });
        }
    }
}

#[cfg(test)]
mod tests;
