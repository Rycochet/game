//! The border round a poi's cutout: the wilderness cells within `BORDER` of
//! it, hatched on the client, where a click or tap leads back to the world
//! map (walking over them does nothing). It follows the cutout's own shape,
//! so a cave mouth's border runs round its hill.

use super::Cutout;

/// How far round a cutout its border reaches, in cells (owner).
pub const BORDER: i64 = 2;

impl Cutout {
    /// The cells of its border, in wilderness cells: not its own, but within
    /// `BORDER` of one of them each way (diagonals too), row by row.
    pub fn border(&self) -> Vec<(i64, i64)> {
        let (min_x, min_y, width, height) = self.level().bounds();
        let near = |x: i64, y: i64| {
            (y - BORDER..=y + BORDER)
                .any(|ny| (x - BORDER..=x + BORDER).any(|nx| self.contains(nx, ny)))
        };
        (min_y - BORDER..min_y + height + BORDER)
            .flat_map(|y| (min_x - BORDER..min_x + width + BORDER).map(move |x| (x, y)))
            .filter(|&(x, y)| !self.contains(x, y) && near(x, y))
            .collect()
    }
}

#[cfg(test)]
mod tests;
