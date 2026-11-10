//! Render Scale and Max Resolution (`GraphicsPrefs::drawn_share`): below 100%
//! the view's camera draws into a smaller picture,
//! which a second camera stretches over the window, with the interface drawn
//! on top at full resolution. At 100% the view's camera draws straight into
//! the window and the second camera isn't there, as before the setting.
//!
//! The picture's scale factor is chosen so its logical size is the window's,
//! so clicks and everything else working in logical pixels need no change.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::render::view::{DebandDither, Tonemapping};
use bevy::window::PrimaryWindow;

use super::ViewCamera;
use crate::common::Preferences;

/// The layer only the screen camera sees, with the stretched picture on it.
const SCREEN_LAYER: usize = 2;

/// The smaller picture and what shows it, while Render Scale is below 100%.
pub(super) struct Scaled {
    picture: Handle<Image>,
    camera: Entity,
    sprite: Entity,
}

/// How big the picture is for this window and share: physical pixels, and the
/// scale factor making its logical size the window's.
fn picture_size(window: &Window, share: f32) -> (UVec2, f32) {
    let size = (window.physical_size().as_vec2() * share)
        .round()
        .max(Vec2::ONE);
    let scale = size.x / window.width().max(1.0);
    (size.as_uvec2(), scale)
}

/// Keeps the view's camera drawing at the chosen share of the window.
pub(super) fn apply_render_scale(
    mut commands: Commands,
    preferences: Res<Preferences>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<&mut RenderTarget, With<ViewCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut sprites: Query<&mut Sprite>,
    mut scaled: Local<Option<Scaled>>,
) {
    let mut target = camera.into_inner();
    let share = preferences.graphics.drawn_share(window.physical_height());
    if share >= 1.0 {
        if let Some(gone) = scaled.take() {
            *target = RenderTarget::Window(default());
            commands.entity(gone.camera).despawn();
            commands.entity(gone.sprite).despawn();
            images.remove(&gone.picture);
        }
        return;
    }
    let (size, scale_factor) = picture_size(&window, share);
    let shown = scaled.get_or_insert_with(|| {
        let picture = images.add(Image::new_target_texture(
            size.x,
            size.y,
            // sRGB, as WebGL 2 can't give a plain one the second view format
            // Bevy asks for.
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        let camera = commands
            .spawn((
                Camera2d,
                Camera {
                    // After the view's camera, which draws the picture first.
                    order: 1,
                    ..default()
                },
                Tonemapping::None,
                DebandDither::Disabled,
                Msaa::Off,
                RenderLayers::layer(SCREEN_LAYER),
            ))
            .id();
        let sprite = commands
            .spawn((
                Sprite::from_image(picture.clone()),
                RenderLayers::layer(SCREEN_LAYER),
            ))
            .id();
        Scaled {
            picture,
            camera,
            sprite,
        }
    });
    if let Some(image) = images.get(&shown.picture)
        && image.size() != size
        && let Some(mut image) = images.get_mut(&shown.picture)
    {
        image.resize(Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        });
    }
    let wanted = ImageRenderTarget {
        handle: shown.picture.clone(),
        scale_factor,
    };
    if !matches!(&*target, RenderTarget::Image(now) if *now == wanted) {
        *target = RenderTarget::Image(wanted);
    }
    if let Ok(mut sprite) = sprites.get_mut(shown.sprite) {
        let fill = Some(window.size());
        if sprite.custom_size != fill {
            sprite.custom_size = fill;
        }
    }
}

#[cfg(test)]
mod tests;
