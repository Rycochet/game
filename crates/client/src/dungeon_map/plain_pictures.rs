//! Floor Textures off: sites' surfaces drawn in plain colours. Each picture is
//! swapped for a copy in which every cell's tile (`TILE` pixels square) is its
//! own average colour, so an atlas (towns) keeps a colour per tile. Whatever
//! else gives a surface a picture (the theme, the ground's colour, memory's grey
//! copies) carries on as before: the copy follows it the next frame.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use super::SurfaceMaterials;
use super::sight_lines::SurfaceMaterial;
use crate::common::Preferences;

/// Pixels across one cell's picture.
const TILE: u32 = 32;

/// The plain copies made so far, both ways round.
#[derive(Default)]
pub(super) struct PlainCopies {
    /// From a picture to its plain copy.
    plain: HashMap<AssetId<Image>, Handle<Image>>,
    /// From a plain copy back to its picture.
    picture: HashMap<AssetId<Image>, Handle<Image>>,
}

/// Every frame: gives each surface its plain copy or its picture back, as the
/// setting says.
pub(super) fn plain_pictures(
    preferences: Res<Preferences>,
    surfaces: Res<SurfaceMaterials>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
    mut copies: Local<PlainCopies>,
) {
    let textured = preferences.video.floor_textures;
    if textured && copies.picture.is_empty() {
        return;
    }
    let all = surfaces
        .solid
        .iter()
        .chain(&surfaces.memory)
        .chain(&surfaces.stairs_through)
        .chain([&surfaces.see_through]);
    for handle in all {
        let Some(shown) = materials
            .get(handle)
            .and_then(|material| material.base.base_color_texture.clone())
        else {
            continue;
        };
        let wanted = if textured {
            copies.picture.get(&shown.id()).cloned()
        } else if copies.picture.contains_key(&shown.id()) {
            None
        } else if let Some(plain) = copies.plain.get(&shown.id()) {
            Some(plain.clone())
        } else if let Some(copy) = images.get(&shown).map(plain_copy) {
            // Made first: `images` can't be added to while borrowed.
            let plain = images.add(copy);
            copies.plain.insert(shown.id(), plain.clone());
            copies.picture.insert(plain.id(), shown.clone());
            Some(plain)
        } else {
            // Not loaded yet: try again next frame.
            None
        };
        if let (Some(wanted), Some(mut material)) = (wanted, materials.get_mut(handle)) {
            material.base.base_color_texture = Some(wanted);
        }
    }
}

/// A copy of an 8-bit RGBA picture with each `TILE`-square tile filled with its
/// average colour (anything else is copied as it is).
pub(super) fn plain_copy(image: &Image) -> Image {
    let mut copy = image.clone();
    let rgba = matches!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm
    );
    let (width, height) = (image.width(), image.height());
    let Some(data) = copy.data.as_mut().filter(|_| rgba) else {
        return copy;
    };
    let pixel = |x: u32, y: u32| ((y * width + x) * 4) as usize;
    for top in (0..height).step_by(TILE as usize) {
        for left in (0..width).step_by(TILE as usize) {
            let xs = left..(left + TILE).min(width);
            let ys = top..(top + TILE).min(height);
            let mut total = [0u64; 4];
            for y in ys.clone() {
                for x in xs.clone() {
                    let at = pixel(x, y);
                    for (sum, value) in total.iter_mut().zip(&data[at..at + 4]) {
                        *sum += u64::from(*value);
                    }
                }
            }
            let count = (xs.len() * ys.len()).max(1) as u64;
            let average = total.map(|sum| (sum / count) as u8);
            for y in ys.clone() {
                for x in xs.clone() {
                    let at = pixel(x, y);
                    data[at..at + 4].copy_from_slice(&average);
                }
            }
        }
    }
    copy
}

#[cfg(test)]
mod tests;
