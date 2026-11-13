//! A castle's size, rolled before its keep: the keep is what fits inside.

use super::super::super::grid::roll;
use super::CastleOptions;
use crate::rng::Seed;

/// The wall's rectangle and the gaps inside it, from the castle's own seed.
#[derive(Clone, Copy, Debug)]
pub(in crate::site) struct CastleSize {
    width: i64,
    height: i64,
    /// Ground between the wall and the keep: west, north, east, south.
    pub(super) gaps: [i64; 4],
    /// The pond's size, and the side it is meant for (west, north or east).
    pub(super) pond: (i64, i64),
    pub(super) pond_side: usize,
}

impl CastleSize {
    pub(in crate::site) fn roll(options: &CastleOptions, seed: Seed) -> Self {
        let mut rng = seed.derive("castle").rng();
        let width = roll(&mut rng, options.width.0, options.width.1);
        let height = roll(&mut rng, options.height.0, options.height.1);
        let mut gaps = [(); 4].map(|_| roll(&mut rng, options.gap.0, options.gap.1));
        let pond = (
            roll(&mut rng, options.pond.0, options.pond.1),
            roll(&mut rng, options.pond.0, options.pond.1),
        );
        let pond_side = rng.range_u64(0, 3) as usize;
        // A lane each side of it.
        let across = if pond_side == 1 { pond.1 } else { pond.0 };
        gaps[pond_side] = gaps[pond_side].max(across + 2);
        Self {
            width,
            height,
            gaps,
            pond,
            pond_side,
        }
    }

    /// The keep's size: what is left inside the wall past the gaps.
    pub(in crate::site) fn keep(&self) -> (i64, i64) {
        let [west, north, east, south] = self.gaps;
        (
            self.width - 2 - west - east,
            self.height - 2 - north - south,
        )
    }
}
