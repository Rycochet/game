//! Deterministic randomness: seeds, position hashing and number sequences.
//!
//! Never use any other source of randomness in shared code. Everything here is
//! plain integer arithmetic, so results are identical on every platform.

/// The SplitMix64 increment ("golden ratio" constant).
const GOLDEN_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

/// A seed for one purpose, e.g. the whole world, or just its land noise.
///
/// Derive a separate seed for each purpose (`world.derive("land")`) so that
/// changing how one thing is generated never shifts the results of another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Seed(u64);

impl Seed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    /// A new independent seed for a named purpose.
    pub fn derive(self, purpose: &str) -> Self {
        Self(mix(self.0 ^ mix(fnv1a(purpose))))
    }

    /// A random value for a position (or any list of integers).
    ///
    /// Use this for the infinite world: the result depends only on the seed
    /// and the coordinates, never on what was generated before.
    pub fn at(self, coords: &[i64]) -> u64 {
        coords.iter().fold(self.0, |hash, &coord| {
            mix(hash.wrapping_add(GOLDEN_GAMMA) ^ coord as u64)
        })
    }

    /// A sequence of random numbers starting from this seed.
    pub fn rng(self) -> Rng {
        Rng { state: self.0 }
    }

    /// A seed in the form the noise library wants.
    pub fn noise_seed(self) -> i32 {
        (self.0 >> 32) as i32
    }
}

/// A deterministic sequence of random numbers (SplitMix64).
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN_GAMMA);
        mix(self.state)
    }

    /// A number in `0.0..1.0`.
    pub fn next_f64(&mut self) -> f64 {
        // The top 53 bits fill an f64's mantissa exactly.
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// A number in `min..max`.
    pub fn range_f64(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }

    /// An integer in `min..max` (`max` excluded).
    pub fn range_u64(&mut self, min: u64, max: u64) -> u64 {
        assert!(min < max, "empty range {min}..{max}");
        // Multiply-and-shift avoids the bias of `%`.
        let span = max - min;
        min + ((self.next_u64() as u128 * span as u128) >> 64) as u64
    }
}

/// SplitMix64's finaliser: scrambles all 64 bits.
fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// FNV-1a: turns a purpose name into a number. Stable across Rust versions,
/// unlike `std`'s hashers.
fn fnv1a(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Golden values: if these change, every generated world changes.
    #[test]
    fn golden_values() {
        let seed = Seed::new(12345);
        let mut rng = seed.rng();
        let values = [
            seed.derive("land").value(),
            seed.at(&[1, -2, 3]),
            rng.next_u64(),
            rng.next_u64(),
        ];
        assert_eq!(values, GOLDEN_VALUES);
    }

    const GOLDEN_VALUES: [u64; 4] = [
        15857797406316004683,
        8104792498113331671,
        2454886589211414944,
        3778200017661327597,
    ];

    #[test]
    fn purposes_are_independent() {
        let seed = Seed::new(1);
        assert_ne!(seed.derive("land"), seed.derive("weather"));
        assert_ne!(seed.at(&[1, 2]), seed.at(&[2, 1]));
    }

    #[test]
    fn ranges_stay_in_bounds() {
        let mut rng = Seed::new(7).rng();
        for _ in 0..1000 {
            let n = rng.range_u64(10, 20);
            assert!((10..20).contains(&n));
            let f = rng.range_f64(-1.0, 1.0);
            assert!((-1.0..1.0).contains(&f));
        }
    }
}
