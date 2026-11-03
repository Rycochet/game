//! Which way a wall cell's wall runs, from its neighbours.

use super::{HALF_CELL, WALL_END_UNITS, WALL_HALF_UNITS};

/// Which of a wall cell's four sides the wall runs out to, because it joins
/// another wall (or a door or window in line with it) there.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WallShape {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

impl WallShape {
    /// How far the wall reaches from the cell's centre towards each side, in
    /// units, as `[north, east, south, west]`: to the cell's edge where it
    /// joins something, otherwise just its thickness, except that a wall
    /// joined on one side only carries on a little past the centre.
    pub fn reach(self) -> [i64; 4] {
        let sides = [self.north, self.east, self.south, self.west];
        let joined = sides.iter().filter(|&&side| side).count();
        std::array::from_fn(|i| {
            if sides[i] {
                HALF_CELL
            } else if joined == 1 && sides[(i + 2) % 4] {
                WALL_END_UNITS
            } else {
                WALL_HALF_UNITS
            }
        })
    }
}
