//! The Generate POI panel's parts, and new seeds with their icon.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::poi::PoiKind;
use shared::rng::Seed;

/// Open the Generate POI panel for this kind (the submenu chose it).
#[derive(Message, Clone, Copy, Debug)]
pub(in crate::debug_menu) struct GenerateKind(pub PoiKind);

/// What the Generate POI panel is set to, besides its seed box.
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct GenerateForm {
    pub(super) kind: Option<PoiKind>,
    pub(super) ruin: u8,
    pub(super) seeds_made: i64,
}

#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct RuinButton(pub(super) u8);

#[derive(Component)]
pub(in crate::debug_menu) struct SeedBox;

#[derive(Component)]
pub(in crate::debug_menu) struct NewSeedButton;

#[derive(Component)]
pub(in crate::debug_menu) struct GenerateButton;

/// The most digits of a seed the new seed button makes: enough for many, and
/// short enough to write down and type in again.
pub(super) const NEW_SEEDS: u64 = 1_000_000_000;
pub(super) const SEED_WIDTH: f32 = 240.0;

/// A seed to try. `now` is any number that keeps changing, `made` how many so far.
pub(super) fn new_seed(now: u64, made: i64) -> u64 {
    Seed::new(now).at(&[made]) % NEW_SEEDS
}

/// A round arrow, drawn here as there is no such letter in the font.
pub(super) fn new_seed_icon() -> Image {
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
