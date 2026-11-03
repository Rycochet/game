//! The outline of one floor, and which cells are in it.

use super::Outline;

/// One floor's outline, in level coordinates.
#[derive(Clone, Copy, Debug)]
pub(in crate::site) struct Shape {
    pub(super) outline: Outline,
    pub(super) x: i64,
    pub(super) y: i64,
    pub(super) width: i64,
    pub(super) height: i64,
    /// How far corners are cut or rounded.
    pub(super) corner: i64,
}

impl Shape {
    /// Whether `(x, y)` is inside the outline (outer wall included).
    pub(in crate::site) fn contains(&self, x: i64, y: i64) -> bool {
        let (cx, cy) = (x - self.x, y - self.y);
        if cx < 0 || cy < 0 || cx >= self.width || cy >= self.height {
            return false;
        }
        // Distance in from the nearest side, each way.
        let dx = cx.min(self.width - 1 - cx);
        let dy = cy.min(self.height - 1 - cy);
        let r = self.corner;
        match self.outline {
            Outline::Rectangle => true,
            Outline::CutCorners => dx + dy >= r,
            // Within a quarter circle of radius r in each corner, measured
            // between cell centres in half cells, so it stays in integers.
            Outline::RoundCorners => {
                dx >= r || dy >= r || {
                    let (ax, ay) = (2 * (r - dx) - 1, 2 * (r - dy) - 1);
                    ax * ax + ay * ay <= 4 * r * r
                }
            }
            Outline::Circle => {
                let (ax, ay) = (2 * cx + 1 - self.width, 2 * cy + 1 - self.height);
                ax * ax + ay * ay <= self.width * self.height
            }
        }
    }

    /// Inside the outer wall: in the outline, and so are all eight
    /// neighbours.
    pub(super) fn is_interior(&self, x: i64, y: i64) -> bool {
        (-1..=1).all(|dy| (-1..=1).all(|dx| self.contains(x + dx, y + dy)))
    }

    /// The same outline `by` cells smaller on every side.
    pub(super) fn shrunk(&self, by: i64) -> Self {
        Self {
            x: self.x + by,
            y: self.y + by,
            width: self.width - 2 * by,
            height: self.height - 2 * by,
            corner: (self.corner - by).max(1),
            ..*self
        }
    }
}
