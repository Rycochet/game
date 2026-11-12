//! The ground's shape on the level the player is on, for drawing: the heights
//! of its grid corners (`shared::relief::Relief`), with the surface between
//! them four flat triangles a cell, meeting at its middle. Everything the
//! views stand on the ground (cells' meshes, tokens, bags, the camera, clicks,
//! rain) reads it here.

use bevy::prelude::*;
use shared::movement::WildHeights;
use shared::relief::{Relief, UNITS_PER_CELL};

/// The ground of the current level, worked out by `player` whenever the level,
/// its kind of ground, its stairs or (in the wilderness) its window's heights
/// change; `relief` is `None` where it is flat (buildings, the Hall).
#[derive(Resource, Default)]
pub struct DrawnGround {
    pub relief: Option<Relief>,
    /// The height drawn at 0, in units: the level's (or window's) middle, so
    /// what is drawn stays near the origin's height (far up, the depth buffer
    /// and floats lose precision and the ground shows through tokens).
    pub reference: i32,
    /// A site's first level's land (`LevelGround::Land`), kept to level its
    /// flats again as cells come without sampling the land again.
    pub land: Option<WildHeights>,
}

impl DrawnGround {
    /// Whether the ground has any shape (it isn't flat at 0).
    pub fn slopes(&self) -> bool {
        self.relief.is_some()
    }

    /// Whether a cell's ground is known: everywhere on flat ground.
    pub fn covers(&self, x: i64, y: i64) -> bool {
        self.relief
            .as_ref()
            .is_none_or(|relief| relief.covers(x, y))
    }

    /// How high a grid corner (cell `(x, y)`'s north-west one) is drawn, in
    /// cells: 0 on flat ground.
    pub fn corner(&self, x: i64, y: i64) -> f32 {
        self.relief
            .as_ref()
            .map_or(0.0, |relief| self.cells(relief.corner(x, y)))
    }

    /// How high a cell's middle is drawn, in cells.
    pub fn drawn(&self, x: i64, y: i64) -> f32 {
        self.relief
            .as_ref()
            .map_or(0.0, |relief| self.cells(relief.centre(x, y)))
    }

    /// How high the ground is drawn at a place given in cells (cell `(x, y)`'s
    /// middle at `(x, y)`), in cells: on the triangle of its cell it is on,
    /// as `Relief::at` works it out in whole units.
    pub fn drawn_between(&self, x: f32, y: f32) -> f32 {
        let Some(relief) = &self.relief else {
            return 0.0;
        };
        let (cx, cy) = (x.round(), y.round());
        let (dx, dy) = (x - cx, y - cy);
        let (cx, cy) = (cx as i64, cy as i64);
        let [nw, ne, sw, se] = relief.corners_of(cx, cy).map(|h| self.cells(h));
        let c = (nw + ne + sw + se) / 4.0;
        // Along towards the side the place is nearest, and across it; a
        // corner is half a cell from the middle.
        if dx.abs() >= dy.abs() {
            let (a, b) = if dx >= 0.0 { (ne, se) } else { (nw, sw) };
            c + dx.abs() * (a + b - 2.0 * c) + dy * (b - a)
        } else {
            let (a, b) = if dy >= 0.0 { (sw, se) } else { (nw, ne) };
            c + dy.abs() * (a + b - 2.0 * c) + dx * (b - a)
        }
    }

    /// Units of height from the reference, in cells.
    fn cells(&self, units: i32) -> f32 {
        (units - self.reference) as f32 / UNITS_PER_CELL as f32
    }
}

#[cfg(test)]
mod tests;
