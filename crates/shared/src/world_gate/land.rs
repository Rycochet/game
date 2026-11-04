//! Coarse land: the planet in cells `CELL` grid points across, each judged by
//! the broad height at its middle. A cell counts as land only if it and most
//! of its neighbours are dry, so a lone spit or a speck of noise doesn't.
//! Cells are worked out as they're asked for, and kept.

use std::collections::HashMap;

use crate::surface::Surface;

/// A cell's size in grid points.
pub(super) const CELL: i64 = 32;
/// Of a cell's 8 neighbours, how many must be dry too.
const DRY_NEIGHBOURS: usize = 6;

/// The coarse land of one planet, as far as it has been looked at.
pub(super) struct CoarseLand<'a> {
    surface: &'a Surface,
    /// Above this is dry (high tide, on the height scale).
    dry_above: f64,
    /// Cells round the planet, where `x` wraps.
    around: i64,
    /// Whether each cell's middle is dry, by cell (`x` wrapped).
    dry: HashMap<(i64, i64), bool>,
}

impl<'a> CoarseLand<'a> {
    pub(super) fn new(surface: &'a Surface, dry_above: f64) -> Self {
        Self {
            surface,
            dry_above,
            around: (surface.width() / CELL).max(1),
            dry: HashMap::new(),
        }
    }

    /// The cell a grid position is in.
    pub(super) fn cell_of(&self, x: i64, y: i64) -> (i64, i64) {
        (
            (x.rem_euclid(self.surface.width()) / CELL).min(self.around - 1),
            y.div_euclid(CELL),
        )
    }

    /// The middle of a cell, in grid points.
    pub(super) fn middle(&self, (x, y): (i64, i64)) -> (i64, i64) {
        (
            x.rem_euclid(self.around) * CELL + CELL / 2,
            y * CELL + CELL / 2,
        )
    }

    /// The cell `x` wrapped round the planet.
    pub(super) fn wrapped(&self, (x, y): (i64, i64)) -> (i64, i64) {
        (x.rem_euclid(self.around), y)
    }

    fn dry(&mut self, cell: (i64, i64)) -> bool {
        let cell = self.wrapped(cell);
        if let Some(&dry) = self.dry.get(&cell) {
            return dry;
        }
        let (x, y) = self.middle(cell);
        let dry = self.surface.broad_height(x, y) > self.dry_above;
        self.dry.insert(cell, dry);
        dry
    }

    /// Whether a cell counts as land: dry, with most of its neighbours dry.
    pub(super) fn is_land(&mut self, (x, y): (i64, i64)) -> bool {
        if !self.dry((x, y)) {
            return false;
        }
        let mut dry_neighbours = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if (dx, dy) != (0, 0) && self.dry((x + dx, y + dy)) {
                    dry_neighbours += 1;
                }
            }
        }
        dry_neighbours >= DRY_NEIGHBOURS
    }
}
