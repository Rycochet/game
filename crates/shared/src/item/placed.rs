//! `Placed`: an item at a place in a grid, and `Holder`, which grid.

use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::item::{Item, Rotation, Shape};

/// An item in a grid: the cell its shape's top left is on, and how it is turned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placed {
    pub item: Item,
    pub x: u8,
    pub y: u8,
    pub rotation: Rotation,
}

impl Placed {
    /// Its shape as it lies.
    pub fn shape(&self) -> Shape {
        self.item.shape().turned(self.rotation)
    }

    /// The grid cells it takes up.
    pub fn cells(&self) -> impl Iterator<Item = (u8, u8)> {
        let (left, top) = (self.x, self.y);
        self.shape().cells().map(move |(x, y)| (left + x, top + y))
    }
}

/// A grid things can be in: a character's pockets or a container's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Holder {
    Pockets,
    Inside(Id),
}
