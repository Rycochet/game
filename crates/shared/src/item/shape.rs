//! The cells something takes up in a grid, and turning it. See
//! `docs/context/shared/item.md`.

use serde::{Deserialize, Serialize};

use super::MAX_SHAPE_SIDE;
use crate::creature::Size;

/// The cells something takes up, within its width and height: bit `y * 8 + x`
/// of `cells`, so 8 × 8 fits in one number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shape {
    pub width: u8,
    pub height: u8,
    cells: u64,
}

impl Shape {
    /// A rectangle.
    pub const fn rectangle(width: u8, height: u8) -> Self {
        let mut cells = 0;
        let mut y = 0;
        while y < height {
            let mut x = 0;
            while x < width {
                cells |= 1 << (y as u32 * 8 + x as u32);
                x += 1;
            }
            y += 1;
        }
        Self {
            width,
            height,
            cells,
        }
    }

    /// A rectangle without some of its cells (bits as in `cells`).
    pub(super) const fn without(self, missing: u64) -> Self {
        Self {
            cells: self.cells & !missing,
            ..self
        }
    }

    pub fn has(self, x: u8, y: u8) -> bool {
        x < self.width && y < self.height && self.cells >> (y as u32 * 8 + x as u32) & 1 == 1
    }

    /// Its cells, as `(x, y)`.
    pub fn cells(self) -> impl Iterator<Item = (u8, u8)> {
        (0..self.height)
            .flat_map(move |y| (0..self.width).map(move |x| (x, y)))
            .filter(move |&(x, y)| self.has(x, y))
    }

    /// How many cells it takes up.
    pub fn size(self) -> u32 {
        self.cells.count_ones()
    }

    /// Turned this way.
    pub fn turned(self, rotation: Rotation) -> Self {
        let (width, height) = match rotation {
            Rotation::None | Rotation::Half => (self.width, self.height),
            Rotation::Quarter | Rotation::ThreeQuarters => (self.height, self.width),
        };
        let mut cells = 0;
        for (x, y) in self.cells() {
            let (new_x, new_y) = rotation.turn_cell(x, y, self.width, self.height);
            cells |= 1 << (new_y as u32 * 8 + new_x as u32);
        }
        Self {
            width,
            height,
            cells,
        }
    }

    /// For something of this size rather than `Medium`: each side is multiplied,
    /// rounding down when enlarging and up when reducing, to at most
    /// `MAX_SHAPE_SIDE`. Gaps stay where they were, in proportion.
    pub fn scaled(self, size: Size) -> Self {
        let (width, height) = (
            scaled_side(self.width, size),
            scaled_side(self.height, size),
        );
        let mut cells = 0;
        for y in 0..height {
            for x in 0..width {
                let from_x = x as u32 * self.width as u32 / width as u32;
                let from_y = y as u32 * self.height as u32 / height as u32;
                if self.has(from_x as u8, from_y as u8) {
                    cells |= 1 << (y as u32 * 8 + x as u32);
                }
            }
        }
        Self {
            width,
            height,
            cells,
        }
    }

    /// Whether something of this shape can go inside something whose outside is
    /// `outer`: it has fewer cells, and fits within its sides, turned or not.
    /// Fewer every time is what stops things nesting for ever.
    pub fn smaller_than(self, outer: Shape) -> bool {
        let fits_as_it_is = self.width <= outer.width && self.height <= outer.height;
        let fits_turned = self.height <= outer.width && self.width <= outer.height;
        self.size() < outer.size() && (fits_as_it_is || fits_turned)
    }
}

/// How far something is turned, clockwise.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    #[default]
    None,
    Quarter,
    Half,
    ThreeQuarters,
}

impl Rotation {
    pub const ALL: [Self; 4] = [Self::None, Self::Quarter, Self::Half, Self::ThreeQuarters];

    /// A quarter turn further on.
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Quarter,
            Self::Quarter => Self::Half,
            Self::Half => Self::ThreeQuarters,
            Self::ThreeQuarters => Self::None,
        }
    }

    /// Where a cell of something `width` by `height` goes when it is turned.
    pub fn turn_cell(self, x: u8, y: u8, width: u8, height: u8) -> (u8, u8) {
        match self {
            Self::None => (x, y),
            Self::Quarter => (height - 1 - y, x),
            Self::Half => (width - 1 - x, height - 1 - y),
            Self::ThreeQuarters => (y, width - 1 - x),
        }
    }
}

/// One side of a shape at this size: see `Shape::scaled`.
fn scaled_side(side: u8, size: Size) -> u8 {
    let side = side as u32 * size.quarters();
    let rounded = if size < Size::Medium {
        side.div_ceil(4)
    } else {
        side / 4
    };
    rounded.min(MAX_SHAPE_SIDE as u32) as u8
}
