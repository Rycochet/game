//! The wall, floor and ground pictures for the place, and grey copies for memory.

use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use shared::hall;
use shared::protocol::LevelKind;
use shared::surface::SurfaceSample;

use super::mesh::Surface;
use super::sight_lines::{MaskSide, SurfaceMaterial};
use super::theme::Theme;
use super::{MEMORY_ALPHA, SurfaceMaterials, load_tile};
use crate::common::{CurrentDungeon, CurrentWorld, GameClock, MapPosition, Weather, WorldReturn};
use crate::surface_style::SurfaceStyle;

/// The wall and floor pictures for the kind of place the player is in.
pub(super) fn apply_theme(
    dungeon: Res<CurrentDungeon>,
    asset_server: Res<AssetServer>,
    mut surfaces: ResMut<SurfaceMaterials>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
) {
    if !dungeon.is_changed() {
        return;
    }
    let kind = dungeon.info.as_ref().map(|info| info.kind);
    let theme = Theme::of_level(kind);
    for surface in [Surface::Floor, Surface::Wall, Surface::WallBlend] {
        let picture = load_tile(&asset_server, theme.picture(surface));
        if let Some(mut material) = materials.get_mut(&surfaces.solid[surface as usize]) {
            material.base.base_color_texture = Some(picture.clone());
            if surface == Surface::Floor {
                material.base.base_color = floor_tint(kind);
            }
        }
        if let Some(mut material) = materials.get_mut(&surfaces.stairs_through[surface as usize]) {
            material.base.base_color_texture = Some(picture.clone());
        }
        surfaces.to_grey.push((surface as usize, picture));
    }
    // Nothing in the Hall of Worlds is drawn see-through (owner): memory there
    // is solid, like a town's.
    let in_hall = matches!(kind, Some(LevelKind::Hall { .. }));
    for (surface, handle) in Surface::ALL.iter().zip(&surfaces.memory) {
        if let Some(mut material) = materials.get_mut(handle) {
            let (alpha, mode) = if in_hall || *surface == Surface::Town {
                (1.0, AlphaMode::Opaque)
            } else {
                (MEMORY_ALPHA, AlphaMode::Blend)
            };
            if material.base.alpha_mode != mode || material.base.base_color.alpha() != alpha {
                material.base.alpha_mode = mode;
                material.base.base_color.set_alpha(alpha);
            }
        }
    }
    // Nothing blocks sight in the Hall, so the mask, which only reaches the
    // floor the camera shows, isn't used there: it would cut off the far end.
    let sides = [
        (&surfaces.solid, MaskSide::InSight),
        (&surfaces.stairs_through, MaskSide::InSight),
        (&surfaces.memory, MaskSide::OutOfSight),
    ];
    for (handles, side) in sides {
        for handle in handles {
            if let Some(mut material) = materials.get_mut(handle) {
                material.extension.set_side((!in_hall).then_some(side));
            }
        }
    }
    if let Some(mut material) = materials.get_mut(&surfaces.see_through) {
        material.base.base_color_texture = Some(load_tile(&asset_server, theme.wall));
    }
}

/// The Hall of Worlds' marble takes a little of the Hall's hue where the player
/// is (`shared::hall::hue`); any other floor is as drawn.
fn floor_tint(kind: Option<LevelKind>) -> Color {
    match kind {
        Some(LevelKind::Hall { origin }) => Color::hsl(hall::hue(origin) as f32, 0.3, 0.95),
        _ => Color::WHITE,
    }
}

/// Paints the outside ground the colour of the land the site stands on (its
/// biome, or snow), once the picture has loaded and whenever that changes.
#[allow(clippy::too_many_arguments)]
pub(super) fn colour_ground(
    world: Option<Res<CurrentWorld>>,
    place: Res<WorldReturn>,
    style: Res<SurfaceStyle>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    surfaces: Res<SurfaceMaterials>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
    mut painted: Local<Option<[u8; 4]>>,
) {
    let Some(world) = world else {
        return;
    };
    let MapPosition { x, y } = place.0;
    // No jitter: it only roughens edges on the map, and here one point's would
    // paint the whole site.
    let sample = SurfaceSample {
        jitter: 0.0,
        ..world.surface.sample(x, y)
    };
    let climate = world.climate.sample(x, y, sample.height);
    let conditions = world.conditions(&clock, &weather);
    let colour = style.ground_colour(sample, climate, conditions.water, conditions.snow_line);
    if *painted == Some(colour) {
        return;
    }
    let Some(picture) = images.get(&surfaces.ground_picture) else {
        return;
    };
    // Made first: `images` can't be added to while borrowed.
    let copy = recoloured(picture, colour);
    let copy = images.add(copy);
    let ground = Surface::Ground as usize;
    for handle in [&surfaces.solid[ground], &surfaces.stairs_through[ground]] {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base.base_color_texture = Some(copy.clone());
        }
    }
    *painted = Some(colour);
}

/// A copy of an 8-bit RGBA picture in one colour, keeping its detail: each
/// pixel is the colour, lighter or darker as the pixel is against the picture's
/// average (anything else is copied as it is).
pub(super) fn recoloured(image: &Image, colour: [u8; 4]) -> Image {
    let mut copy = image.clone();
    let rgba = matches!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm
    );
    if let (true, Some(data)) = (rgba, copy.data.as_mut()) {
        let luma = |pixel: &[u8; 4]| {
            let [r, g, b] = [pixel[0], pixel[1], pixel[2]].map(f32::from);
            0.299 * r + 0.587 * g + 0.114 * b
        };
        let pixels = data.as_chunks_mut::<4>().0;
        let total: f32 = pixels.iter().map(luma).sum();
        let average = (total / pixels.len().max(1) as f32).max(1.0);
        for pixel in pixels {
            let scale = luma(pixel) / average;
            for (channel, base) in pixel[..3].iter_mut().zip(colour) {
                *channel = (f32::from(base) * scale).clamp(0.0, 255.0) as u8;
            }
        }
    }
    copy
}

/// Gives remembered surfaces grey copies of their pictures, once the
/// pictures have loaded (each is copied once).
pub(super) fn grey_pictures(
    mut surfaces: ResMut<SurfaceMaterials>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
) {
    if surfaces.to_grey.is_empty() {
        return;
    }
    let surfaces = &mut *surfaces;
    surfaces.to_grey.retain(|(index, picture)| {
        let grey = match surfaces.greys.get(&picture.id()) {
            Some(grey) => grey.clone(),
            None => {
                let Some(image) = images.get(picture) else {
                    // Not loaded yet: try again next frame.
                    return true;
                };
                // Made first: `images` can't be added to while borrowed.
                let copy = greyed(image);
                let grey = images.add(copy);
                surfaces.greys.insert(picture.id(), grey.clone());
                grey
            }
        };
        if let Some(mut material) = materials.get_mut(&surfaces.memory[*index]) {
            material.base.base_color_texture = Some(grey);
        }
        false
    });
}

/// A grey copy of an 8-bit RGBA picture (anything else is copied as it is).
pub(super) fn greyed(image: &Image) -> Image {
    let mut grey = image.clone();
    let rgba = matches!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm
    );
    if let (true, Some(data)) = (rgba, grey.data.as_mut()) {
        for pixel in data.as_chunks_mut::<4>().0 {
            let [r, g, b] = [pixel[0], pixel[1], pixel[2]].map(f32::from);
            // Rec. 601 weights on the stored values: good enough for looks.
            let luma = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            pixel[..3].fill(luma);
        }
    }
    grey
}
