//! Small copies of the game's pictures, made once per size, for stamping on
//! maps.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use super::canvas::Sprite;

/// Part of a picture: left, top, and the width and height of a square. `None`
/// is the whole picture.
pub type Part = Option<(u32, u32, u32)>;

#[derive(Resource, Default)]
pub struct Sprites {
    pictures: HashMap<String, Handle<Image>>,
    scaled: HashMap<(String, Part, u32), Sprite>,
}

impl Sprites {
    /// Gets ready to stamp a picture at `size` pixels across. `false` while
    /// it is still loading.
    pub fn prepare(
        &mut self,
        asset_server: &AssetServer,
        images: &Assets<Image>,
        path: &str,
        part: Part,
        size: u32,
    ) -> bool {
        let key = (path.to_string(), part, size);
        if self.scaled.contains_key(&key) {
            return true;
        }
        let handle = self
            .pictures
            .entry(path.to_string())
            .or_insert_with(|| asset_server.load(path.to_string()));
        let Some(image) = images.get(&*handle) else {
            return false;
        };
        let Some(sprite) = shrink(image, part, size) else {
            return false;
        };
        self.scaled.insert(key, sprite);
        true
    }

    /// A sprite `prepare` has made.
    pub fn get(&self, path: &str, part: Part, size: u32) -> Option<&Sprite> {
        self.scaled.get(&(path.to_string(), part, size))
    }
}

/// A picture scaled to `size` by averaging what falls in each new pixel.
fn shrink(image: &Image, part: Part, size: u32) -> Option<Sprite> {
    if !matches!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm
    ) {
        return None;
    }
    let data = image.data.as_ref()?;
    let (width, height) = (image.width(), image.height());
    let (left, top, side) = part.unwrap_or((0, 0, width.min(height)));
    Some(Sprite {
        size,
        pixels: shrink_pixels(data, width, (left, top, side), size),
    })
}

fn shrink_pixels(
    data: &[u8],
    width: u32,
    (left, top, side): (u32, u32, u32),
    size: u32,
) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    let range = |at: u32| {
        let from = at * side / size;
        from..((at + 1) * side / size).max(from + 1)
    };
    for y in 0..size {
        for x in 0..size {
            // Weighted by alpha, so a see-through pixel's colour doesn't bleed.
            let (mut red, mut green, mut blue, mut alpha, mut count) = (0, 0, 0, 0, 0);
            for source_y in range(y) {
                for source_x in range(x) {
                    let at = (((top + source_y) * width + left + source_x) * 4) as usize;
                    let weight = u32::from(data[at + 3]);
                    red += u32::from(data[at]) * weight;
                    green += u32::from(data[at + 1]) * weight;
                    blue += u32::from(data[at + 2]) * weight;
                    alpha += weight;
                    count += 1;
                }
            }
            let colour = |total: u32| total.checked_div(alpha).unwrap_or(0) as u8;
            pixels.extend([
                colour(red),
                colour(green),
                colour(blue),
                (alpha / count) as u8,
            ]);
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinking_averages_and_ignores_the_colour_of_see_through_pixels() {
        let data = [
            200, 0, 0, 255, /**/ 0, 255, 0, 0, //
            100, 0, 0, 255, /**/ 0, 255, 0, 0,
        ];
        let pixels = shrink_pixels(&data, 2, (0, 0, 2), 1);
        assert_eq!(pixels, [150, 0, 0, 127]);
    }

    #[test]
    fn shrinking_can_take_part_of_a_picture() {
        let data = [
            1, 1, 1, 255, /**/ 2, 2, 2, 255, //
            3, 3, 3, 255, /**/ 4, 4, 4, 255,
        ];
        assert_eq!(shrink_pixels(&data, 2, (1, 1, 1), 1), [4, 4, 4, 255]);
    }
}
