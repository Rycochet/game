//! Rectangles of floor in a quarter of a cell, and where they cut a wall's foot.

use bevy::prelude::*;

use super::HALF_CELL;

/// A rectangle of floor in a quarter of a cell, as distances from the cell's
/// centre.
#[derive(Clone, Copy)]
pub(super) struct Patch {
    pub(super) east: (f32, f32),
    pub(super) south: (f32, f32),
}

impl Patch {
    pub(super) fn contains(&self, east: f32, south: f32) -> bool {
        (self.east.0..=self.east.1).contains(&east)
            && (self.south.0..=self.south.1).contains(&south)
    }

    pub(super) fn distance_to(&self, east: f32, south: f32) -> f32 {
        let dx = (self.east.0 - east).max(east - self.east.1).max(0.0);
        let dz = (self.south.0 - south).max(south - self.south.1).max(0.0);
        Vec2::new(dx, dz).length()
    }
}

/// Where the patches' edges cut a quarter along one direction.
pub(super) fn cuts(foot: &[Patch], edges: impl Fn(&Patch) -> (f32, f32)) -> Vec<f32> {
    let mut cuts = vec![0.0, HALF_CELL];
    for patch in foot {
        let (low, high) = edges(patch);
        cuts.extend([low.min(HALF_CELL), high.min(HALF_CELL)]);
    }
    cuts.sort_by(f32::total_cmp);
    cuts.dedup();
    cuts
}
