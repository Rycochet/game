//! A picture being painted on the CPU: the pixels of a Bevy `Image`, with the
//! few shapes the maps need, their edges smoothed.

/// Pixels as sRGB red, green, blue, alpha bytes, row by row.
pub struct Canvas<'a> {
    pub pixels: &'a mut [u8],
    pub width: u32,
    pub height: u32,
}

/// A small picture to stamp on a canvas (see `sprites`).
pub struct Sprite {
    pub size: u32,
    /// Red, green, blue, alpha bytes, row by row.
    pub pixels: Vec<u8>,
}

impl Canvas<'_> {
    /// Every pixel in one colour.
    pub fn fill(&mut self, colour: [u8; 4]) {
        for pixel in self.pixels.as_chunks_mut::<4>().0 {
            pixel.copy_from_slice(&colour);
        }
    }

    /// Sets a pixel, which must be on the canvas.
    pub fn set(&mut self, x: u32, y: u32, colour: [u8; 3]) {
        let at = ((y * self.width + x) * 4) as usize;
        self.pixels[at..at + 3].copy_from_slice(&colour);
        self.pixels[at + 3] = 255;
    }

    /// Mixes `colour` over a pixel, `amount` from 0 to 1, keeping its alpha
    /// at least as solid as `amount` makes it.
    pub fn blend(&mut self, x: u32, y: u32, colour: [u8; 3], amount: f32) {
        if x >= self.width || y >= self.height || amount <= 0.0 {
            return;
        }
        let at = ((y * self.width + x) * 4) as usize;
        for (pixel, new) in self.pixels[at..at + 3].iter_mut().zip(colour) {
            let old = f32::from(*pixel);
            *pixel = (old + (f32::from(new) - old) * amount.min(1.0)).round() as u8;
        }
        let alpha = f32::from(self.pixels[at + 3]) / 255.0;
        let alpha = alpha + (1.0 - alpha) * amount.min(1.0);
        self.pixels[at + 3] = (alpha * 255.0).round() as u8;
    }

    /// Fills a shape given by its distance function (negative inside, in
    /// pixels). `(centre, reach)` is a square the shape stays within.
    pub fn shape(
        &mut self,
        centre: (f32, f32),
        reach: f32,
        colour: [u8; 3],
        distance: impl Fn(f32, f32) -> f32,
    ) {
        let low = |at: f32| (at - reach - 1.0).floor().max(0.0) as u32;
        let high = |at: f32, end: u32| ((at + reach + 1.0).ceil().max(0.0) as u32).min(end);
        for y in low(centre.1)..high(centre.1, self.height) {
            for x in low(centre.0)..high(centre.0, self.width) {
                let inside = 0.5 - distance(x as f32 + 0.5, y as f32 + 0.5);
                self.blend(x, y, colour, inside.clamp(0.0, 1.0));
            }
        }
    }

    /// Fills a circle.
    pub fn disc(&mut self, centre: (f32, f32), radius: f32, colour: [u8; 3]) {
        self.shape(centre, radius, colour, |x, y| {
            (x - centre.0).hypot(y - centre.1) - radius
        });
    }

    /// Stamps a sprite centred on a point, at `opacity` from 0 to 1.
    pub fn sprite(&mut self, sprite: &Sprite, centre: (f32, f32), opacity: f32) {
        let left = (centre.0 - sprite.size as f32 / 2.0).round() as i64;
        let top = (centre.1 - sprite.size as f32 / 2.0).round() as i64;
        for (index, pixel) in sprite.pixels.as_chunks::<4>().0.iter().enumerate() {
            let (column, row) = (
                index as i64 % sprite.size as i64,
                index as i64 / sprite.size as i64,
            );
            let (x, y) = (left + column, top + row);
            if x >= 0 && y >= 0 {
                let amount = f32::from(pixel[3]) / 255.0 * opacity;
                self.blend(x as u32, y as u32, [pixel[0], pixel[1], pixel[2]], amount);
            }
        }
    }
}

/// Distance from a point to a polygon, negative inside.
pub fn polygon_distance(corners: &[(f32, f32)], x: f32, y: f32) -> f32 {
    let mut nearest = f32::MAX;
    let mut inside = false;
    let mut previous = corners[corners.len() - 1];
    for &corner in corners {
        let (edge_x, edge_y) = (previous.0 - corner.0, previous.1 - corner.1);
        let (to_x, to_y) = (x - corner.0, y - corner.1);
        let along =
            ((to_x * edge_x + to_y * edge_y) / (edge_x * edge_x + edge_y * edge_y)).clamp(0.0, 1.0);
        nearest = nearest.min((to_x - edge_x * along).hypot(to_y - edge_y * along));
        if (corner.1 > y) != (previous.1 > y)
            && x < corner.0 + (y - corner.1) / (previous.1 - corner.1) * (previous.0 - corner.0)
        {
            inside = !inside;
        }
        previous = corner;
    }
    if inside { -nearest } else { nearest }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygon_distance_is_negative_inside() {
        let square = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        assert_eq!(polygon_distance(&square, 5.0, 5.0), -5.0);
        assert_eq!(polygon_distance(&square, 13.0, 5.0), 3.0);
    }

    #[test]
    fn half_see_through_sprites_mix_with_what_is_under_them() {
        let mut pixels = vec![0; 4];
        let mut canvas = Canvas {
            pixels: &mut pixels,
            width: 1,
            height: 1,
        };
        canvas.fill([0, 0, 0, 255]);
        let sprite = Sprite {
            size: 1,
            pixels: vec![200, 100, 0, 255],
        };
        canvas.sprite(&sprite, (0.5, 0.5), 0.5);
        assert_eq!(pixels, [100, 50, 0, 255]);
    }

    #[test]
    fn sprites_off_the_edge_are_cut_off() {
        let mut pixels = vec![0; 16];
        let mut canvas = Canvas {
            pixels: &mut pixels,
            width: 2,
            height: 2,
        };
        let sprite = Sprite {
            size: 2,
            pixels: vec![255; 16],
        };
        canvas.sprite(&sprite, (0.0, 0.0), 1.0);
        assert_eq!(pixels[0..4], [255; 4]);
        assert_eq!(pixels[4..8], [0; 4]);
    }
}
