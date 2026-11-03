//! Laying pictures out on a page: rows of cells, each row as tall as the first
//! picture put in it, with a gap all round each so that nothing from one cell
//! reaches another.

use bevy::math::{URect, UVec2};

/// The rows on a page so far.
pub(super) struct Shelves {
    size: UVec2,
    gap: u32,
    rows: Vec<Row>,
    /// Where the next row would start.
    bottom: u32,
}

struct Row {
    top: u32,
    height: u32,
    /// How far along it is filled, gaps included.
    used: u32,
}

impl Shelves {
    pub(super) fn new(size: UVec2, gap: u32) -> Self {
        Self {
            size,
            gap,
            rows: Vec::new(),
            bottom: 0,
        }
    }

    /// Where a picture this size goes, or `None` if the page has no room left for it.
    pub(super) fn place(&mut self, cell: UVec2) -> Option<URect> {
        let (width, gap) = (self.size.x, self.gap);
        // A row with room, which it doesn't leave more than a quarter empty above.
        let fits = |row: &Row| {
            cell.y <= row.height && cell.y * 4 >= row.height * 3 && row.used + cell.x <= width
        };
        let index = match self.rows.iter().position(fits) {
            Some(index) => index,
            None => {
                if cell.x > width || self.bottom + cell.y > self.size.y {
                    return None;
                }
                self.rows.push(Row {
                    top: self.bottom,
                    height: cell.y,
                    used: 0,
                });
                self.bottom += cell.y + gap;
                self.rows.len() - 1
            }
        };
        let row = &mut self.rows[index];
        let corner = UVec2::new(row.used, row.top);
        row.used += cell.x + gap;
        Some(URect::from_corners(corner, corner + cell))
    }
}

#[cfg(test)]
mod tests;
