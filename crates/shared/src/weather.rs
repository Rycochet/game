//! Clouds and rain at any place and time. Only the terrain seed and the
//! server's clock go in, so every client draws the same sky.
//!
//! Four-dimensional simplex noise: three for the surface's east-west-wrapping
//! cylinder, one for time. See `docs/context/shared/weather.md`.

use crate::rng::Seed;
use crate::world::WorldParams;
use core::f64::consts::TAU;

/// Grid units per noise unit: single clouds, and the weather fronts they
/// gather in.
const CLOUD_ZOOM: f64 = 110.0;
const FRONT_ZOOM: f64 = 800.0;

/// Noise units per game hour: how fast clouds and fronts change shape.
const CLOUD_CHANGE: f64 = 0.2;
const FRONT_CHANGE: f64 = 0.04;

/// Grid units per game hour the whole sky drifts east.
const WIND: f64 = 15.0;

/// Clouds at one place, each `0..=1`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WeatherSample {
    /// How much of the sky is cloud.
    pub cover: f64,
    /// How hard it is raining or snowing: 0 none, 1 the worst of a storm.
    pub rain: f64,
}

/// Works out the weather for one world.
pub struct WeatherField {
    cloud: Simplex4,
    front: Simplex4,
    width: f64,
}

impl WeatherField {
    pub fn new(world: &WorldParams) -> Self {
        Self {
            cloud: Simplex4::new(world.seed.derive("clouds")),
            front: Simplex4::new(world.seed.derive("fronts")),
            width: (world.circumference / 1000).max(1) as f64,
        }
    }

    /// The weather at a grid position and time, in (fractional) game hours.
    pub fn sample(&self, x: i64, y: i64, hours: f64) -> WeatherSample {
        let width_x = x.rem_euclid(self.width as i64) as f64;
        let angle = TAU * (width_x - WIND * hours) / self.width;
        let (cos, sin) = (libm::cos(angle), libm::sin(angle));
        // On a cylinder in noise space, as `surface` does.
        let at = |noise: &Simplex4, zoom: f64, time: f64| {
            let radius = self.width / (zoom * TAU);
            noise.get(cos * radius, sin * radius, y as f64 / zoom, time)
        };
        let cloud = at(&self.cloud, CLOUD_ZOOM, hours * CLOUD_CHANGE)
            + 0.5 * at(&self.cloud, CLOUD_ZOOM / 2.0, hours * CLOUD_CHANGE * 2.0);
        let front = at(&self.front, FRONT_ZOOM, hours * FRONT_CHANGE);
        let total = 0.55 * cloud + 1.3 * front;
        WeatherSample {
            cover: smoothstep(-0.05, 0.45, total),
            rain: smoothstep(0.45, 1.0, total),
        }
    }
}

/// 0 below `low`, 1 above `high`, a smooth curve between.
fn smoothstep(low: f64, high: f64, value: f64) -> f64 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Ken Perlin's simplex noise in four dimensions, after Stefan Gustavson's
/// public-domain description. Only `+ - * /` and `floor`, so it gives the same
/// result on every platform. Output is roughly `-1..=1`.
struct Simplex4 {
    /// A shuffle of `0..256`, twice over so lookups needn't wrap.
    permutation: [u8; 512],
}

impl Simplex4 {
    fn new(seed: Seed) -> Self {
        let mut shuffled: [u8; 256] = core::array::from_fn(|i| i as u8);
        let mut rng = seed.rng();
        for i in (1..256).rev() {
            shuffled.swap(i, rng.range_u64(0, i as u64 + 1) as usize);
        }
        Self {
            permutation: core::array::from_fn(|i| shuffled[i % 256]),
        }
    }

    fn get(&self, x: f64, y: f64, z: f64, w: f64) -> f64 {
        // sqrt(5), written out so no platform's sqrt comes into it.
        const SQRT5: f64 = 2.23606797749979;
        const F4: f64 = (SQRT5 - 1.0) / 4.0;
        const G4: f64 = (5.0 - SQRT5) / 20.0;

        // Which simplex cell this is in.
        let skew = (x + y + z + w) * F4;
        let cell = [
            (x + skew).floor(),
            (y + skew).floor(),
            (z + skew).floor(),
            (w + skew).floor(),
        ];
        let unskew = (cell[0] + cell[1] + cell[2] + cell[3]) * G4;
        let origin = [
            x - (cell[0] - unskew),
            y - (cell[1] - unskew),
            z - (cell[2] - unskew),
            w - (cell[3] - unskew),
        ];

        // Each axis's rank decides the order the simplex's corners go in.
        let mut rank = [0u8; 4];
        for a in 0..4 {
            for b in a + 1..4 {
                if origin[a] > origin[b] {
                    rank[a] += 1;
                } else {
                    rank[b] += 1;
                }
            }
        }

        // The cell wrapped to the permutation table.
        let wrapped = cell.map(|c| c.rem_euclid(256.0) as usize);
        let mut total = 0.0;
        for corner in 0..5u8 {
            // How far along each axis this corner is from the first.
            let steps: [usize; 4] = core::array::from_fn(|i| {
                if corner == 4 {
                    1
                } else {
                    usize::from(rank[i] >= 4 - corner)
                }
            });
            let offset = corner as f64 * G4;
            let d: [f64; 4] = core::array::from_fn(|i| origin[i] - steps[i] as f64 + offset);
            let falloff = 0.6 - d[0] * d[0] - d[1] * d[1] - d[2] * d[2] - d[3] * d[3];
            if falloff > 0.0 {
                let p = &self.permutation;
                let hash = p[wrapped[0]
                    + steps[0]
                    + p[wrapped[1]
                        + steps[1]
                        + p[wrapped[2] + steps[2] + p[wrapped[3] + steps[3]] as usize] as usize]
                        as usize];
                let squared = falloff * falloff;
                total += squared * squared * gradient(hash, d);
            }
        }
        27.0 * total
    }
}

/// One of the 32 gradient directions in 4D (each has three of ±1 and one 0),
/// dotted with `d`.
fn gradient(hash: u8, [x, y, z, w]: [f64; 4]) -> f64 {
    let hash = hash & 31;
    // Which axis is left out, then the signs of the other three.
    let (a, b, c) = match hash >> 3 {
        0 => (y, z, w),
        1 => (x, z, w),
        2 => (x, y, w),
        _ => (x, y, z),
    };
    let sign = |value: f64, bit: u8| if hash & bit == 0 { value } else { -value };
    sign(a, 4) + sign(b, 2) + sign(c, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> WeatherField {
        WeatherField::new(&WorldParams::from_seed(Seed::new(12345)))
    }

    // Golden values: if these change, all clients on older builds disagree.
    #[test]
    fn golden_samples() {
        let field = field();
        let samples: Vec<_> = [(0, 0, 0.0), (120, -80, 3.5), (1_000_000, 42, 1000.25)]
            .iter()
            .map(|&(x, y, hours)| field.sample(x, y, hours))
            .collect();
        assert_eq!(format!("{samples:?}"), GOLDEN_SAMPLES);
    }

    const GOLDEN_SAMPLES: &str = "[WeatherSample { cover: 0.6655160302734207, rain: 0.0 }, WeatherSample { cover: 0.0, rain: 0.0 }, WeatherSample { cover: 0.1131246678801845, rain: 0.0 }]";

    #[test]
    fn noise_stays_in_range_and_varies() {
        let noise = Simplex4::new(Seed::new(1));
        let (mut lowest, mut highest) = (f64::MAX, f64::MIN);
        for i in 0..200_000 {
            let v = i as f64 * 0.0137;
            let value = noise.get(v, v * 0.71 + 3.0, v * 0.37 - 9.0, v * 1.13);
            lowest = lowest.min(value);
            highest = highest.max(value);
        }
        assert!(lowest > -1.2 && highest < 1.2, "{lowest}..{highest}");
        assert!(lowest < -0.5 && highest > 0.5, "{lowest}..{highest}");
    }

    #[test]
    fn wraps_east_west() {
        let field = field();
        let width = field.width as i64;
        assert_eq!(field.sample(3, 7, 2.5), field.sample(3 + width, 7, 2.5));
    }

    #[test]
    fn the_sky_changes_gradually() {
        let field = field();
        let (a, b) = (field.sample(50, 50, 10.0), field.sample(50, 50, 10.01));
        assert!((a.cover - b.cover).abs() < 0.05);
    }

    /// Over a stretch of world and time there are clear skies, clouds and rain.
    #[test]
    fn there_is_a_range_of_weather() {
        let field = field();
        let (mut clear, mut cloudy, mut raining) = (0, 0, 0);
        for i in 0..4000i64 {
            let sample = field.sample(i * 37, i * 53 % 9000, i as f64 * 1.7);
            clear += usize::from(sample.cover < 0.1);
            cloudy += usize::from(sample.cover > 0.5);
            raining += usize::from(sample.rain > 0.1);
        }
        assert!(
            clear > 400 && cloudy > 400 && raining > 100,
            "{clear} {cloudy} {raining}"
        );
    }
}
