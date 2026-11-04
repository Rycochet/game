//! What the globe's picture shows, as the bar's Layer dropdown chooses.

use shared::surface::PlateSample;

use crate::surface_style::hsl_to_rgba;

/// What the picture shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Layer {
    /// As the world map colours it: biomes, water, snow, darker under trees.
    #[default]
    Surface,
    /// Height in metres from the sea, in a map's tints.
    Elevation,
    /// The plates, for generators that have them: a colour each, brighter
    /// carrying continents, with their edges red where they push together and
    /// blue where they part.
    Plates,
    /// The yearly average temperature, blue cold to red hot.
    Temperature,
    /// How wet, brown dry to blue wet.
    Moisture,
}

impl Layer {
    pub(super) const ALL: [Layer; 5] = [
        Layer::Surface,
        Layer::Elevation,
        Layer::Plates,
        Layer::Temperature,
        Layer::Moisture,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Layer::Surface => "Surface",
            Layer::Elevation => "Elevation",
            Layer::Plates => "Plates",
            Layer::Temperature => "Temperature",
            Layer::Moisture => "Moisture",
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

/// How near its edge a plate's edge colour shows, in plate cells, and how
/// fast plates must close or part for it to.
const EDGE: f64 = 0.015;
const MOVING: f64 = 0.2;
/// A planet without plates.
const NO_PLATES: [u8; 4] = [60, 60, 66, 255];

/// A plate's colour, from which plate it is; or its edge's.
pub(super) fn plate_tint(sample: Option<PlateSample>) -> [u8; 4] {
    let Some(sample) = sample else {
        return NO_PLATES;
    };
    if sample.edge < EDGE {
        if sample.closing > MOVING {
            return [220, 40, 30, 255];
        }
        if sample.closing < -MOVING {
            return [40, 120, 230, 255];
        }
        return [230, 230, 230, 255];
    }
    let (column, row) = sample.plate;
    // Neighbours a few hues apart, so they're told apart.
    let hue = ((column * 5 + row * 3).rem_euclid(12) * 30) as f64;
    let lightness = if sample.continental { 0.62 } else { 0.32 };
    hsl_to_rgba(hue, 0.45, lightness)
}

/// A temperature in °C as a colour: deep blue at -30 or below, white at
/// freezing, through yellow to deep red at 35 or above.
pub(super) fn temperature_tint(celsius: f64) -> [u8; 4] {
    let t = ((celsius + 30.0) / 65.0).clamp(0.0, 1.0);
    // Hue from 240 (blue) to 0 (red); pale near freezing.
    let freezing = 30.0 / 65.0;
    let lightness = 0.85 - 0.4 * ((t - freezing).abs() / (1.0 - freezing)).min(1.0);
    hsl_to_rgba(240.0 * (1.0 - t), 0.75, lightness)
}

/// Moisture (`0..=1`) as a colour: brown dry, green, blue wet.
pub(super) fn moisture_tint(moisture: f64) -> [u8; 4] {
    let m = moisture.clamp(0.0, 1.0);
    hsl_to_rgba(30.0 + 180.0 * m, 0.55, 0.45)
}
