//! What the globe's picture shows, as the bar's Layer dropdown chooses.

/// What the picture shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Layer {
    /// As the world map colours it: biomes, water, snow, darker under trees.
    #[default]
    Surface,
    /// Height in metres from the sea, in a map's tints.
    Elevation,
}

impl Layer {
    pub(super) const ALL: [Layer; 2] = [Layer::Surface, Layer::Elevation];

    pub(super) fn name(self) -> &'static str {
        match self {
            Layer::Surface => "Surface",
            Layer::Elevation => "Elevation",
        }
    }
}

/// Tints by height, as maps colour them: blues darkening with depth, greens
/// low down, then yellows and browns, then white on the highest peaks.
const TINTS: [(i32, [u8; 3]); 13] = [
    (-8000, [8, 22, 64]),
    (-4000, [24, 58, 134]),
    (-1000, [56, 112, 184]),
    (-200, [104, 164, 212]),
    (0, [150, 200, 230]),
    (1, [74, 138, 74]),
    (200, [120, 170, 86]),
    (500, [186, 194, 116]),
    (1000, [204, 172, 108]),
    (2000, [166, 122, 78]),
    (3500, [128, 96, 84]),
    (5000, [200, 196, 196]),
    (7000, [255, 255, 255]),
];

/// The tint for a height in metres from the sea.
pub(super) fn elevation_tint(metres: i32) -> [u8; 4] {
    let after = TINTS.iter().position(|&(from, _)| from > metres);
    let rgb = match after {
        None => TINTS[TINTS.len() - 1].1,
        Some(0) => TINTS[0].1,
        Some(index) => {
            let (low, low_rgb) = TINTS[index - 1];
            let (high, high_rgb) = TINTS[index];
            let along = (metres - low) as f32 / (high - low) as f32;
            let mix = |part: usize| {
                (low_rgb[part] as f32 + (high_rgb[part] as f32 - low_rgb[part] as f32) * along)
                    .round() as u8
            };
            [mix(0), mix(1), mix(2)]
        }
    };
    [rgb[0], rgb[1], rgb[2], 255]
}
