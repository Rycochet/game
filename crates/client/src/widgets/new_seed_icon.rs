//! The button that makes a new random seed, and its icon.

use bevy::asset::{RenderAssetUsages, uuid_handle};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::{ButtonState, ButtonTooltip, Pressable, spawn_icon_button};

/// The icon, made once at start (`make_new_seed_icon`), so buttons can show it
/// without reaching for the images themselves.
const NEW_SEED_ICON: Handle<Image> = uuid_handle!("8d3c2f61-7a4e-4b19-a0c5-2e9f6b1d4a73");

/// A square button showing a round arrow, named "Random seed", `side` across. It is
/// `Pressable`: put a marker on it to tell its presses apart.
pub fn spawn_new_seed_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    side: f32,
) -> EntityCommands<'a> {
    let node = Node {
        width: px(side),
        height: px(side),
        flex_shrink: 0.0,
        ..default()
    };
    let mut button = spawn_icon_button(parent, ButtonState::Normal, NEW_SEED_ICON, node);
    button.insert((Pressable, ButtonTooltip("Random seed".into())));
    button
}

pub(super) fn make_new_seed_icon(mut images: ResMut<Assets<Image>>) {
    let _ = images.insert(&NEW_SEED_ICON, new_seed_icon());
}

/// A round arrow, drawn here as there is no such letter in the font.
fn new_seed_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [64, 43, 26, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = (SIZE as f32 - 1.0) / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (dx, dy) = (x as f32 - middle, y as f32 - middle);
            let reach = dx.hypot(dy);
            // Clockwise from the top, leaving a gap for the arrow's head.
            let round = dx.atan2(-dy).rem_euclid(std::f32::consts::TAU);
            let ring = (8.0..12.0).contains(&reach) && round > 0.9;
            // The head, pointing round the ring at the top right.
            let head_along = dx + 2.0;
            let head =
                (0.0..8.0).contains(&head_along) && (dy + 10.0).abs() < (8.0 - head_along) * 0.9;
            if ring || head {
                bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&INK_BYTES);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
