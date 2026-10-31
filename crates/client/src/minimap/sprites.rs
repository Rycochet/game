//! Small copies of the game's pictures, made once per size, for stamping on
//! maps.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use super::canvas::Sprite;

/// How many times bigger `Sprites::make` has pictures drawn, to smooth them.
const OVERSAMPLE: u32 = 3;

/// How opaque a pixel must be to have a border drawn round it.
const SOLID: u8 = 128;

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

    /// Gets ready to stamp a picture worked out rather than loaded: `draw` is
    /// asked for it `OVERSAMPLE` times the size, as RGBA, the first time.
    pub fn make(&mut self, name: &str, size: u32, draw: impl FnOnce(u32) -> Vec<u8>) {
        self.scaled
            .entry((name.to_string(), None, size))
            .or_insert_with(|| {
                let big = size * OVERSAMPLE;
                let pixels = shrink_pixels(&draw(big), big, (0, 0, big), size);
                Sprite {
                    size,
                    pixels: with_border(&pixels, size),
                }
            });
    }

    /// A sprite `prepare` or `make` has made.
    pub fn get(&self, path: &str, part: Part, size: u32) -> Option<&Sprite> {
        self.scaled.get(&(path.to_string(), part, size))
    }
}

/// A square RGBA picture with a solid black line round what shows in it: each
/// clear pixel touching a solid one, diagonally too.
fn with_border(pixels: &[u8], size: u32) -> Vec<u8> {
    let size = size as i64;
    let solid = |x: i64, y: i64| {
        (0..size).contains(&x)
            && (0..size).contains(&y)
            && pixels[((y * size + x) * 4 + 3) as usize] >= SOLID
    };
    let mut bordered = pixels.to_vec();
    for y in 0..size {
        for x in 0..size {
            let touching = (-1..=1).any(|dy| (-1..=1).any(|dx| solid(x + dx, y + dy)));
            if touching && !solid(x, y) {
                let at = ((y * size + x) * 4) as usize;
                bordered[at..at + 4].copy_from_slice(&[0, 0, 0, 255]);
            }
        }
    }
    bordered
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
    fn a_border_goes_round_what_is_solid() {
        let mut pixels = vec![0; 3 * 3 * 4];
        pixels[16..20].copy_from_slice(&[9, 9, 9, 255]);
        let bordered = with_border(&pixels, 3);
        for (i, pixel) in bordered.chunks(4).enumerate() {
            let expected = if i == 4 {
                [9, 9, 9, 255]
            } else {
                [0, 0, 0, 255]
            };
            assert_eq!(pixel, expected);
        }
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
