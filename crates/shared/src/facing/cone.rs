//! What one creature can see by the way it faces: a cone ahead and a circle
//! all round. Walls are `dungeon::Sight`'s business; this is only the angle.

use crate::spot::Spot;

use super::{Field, Heading};

/// A creature's field of view where it stands. Shared, so the server's sight,
/// mobs' noticing and the client's drawing go by the same rule, and so an
/// ability can one day draw an enemy's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Cone {
    pub at: Spot,
    pub heading: Heading,
    pub field: Field,
}

impl Cone {
    pub fn new(at: Spot, heading: Heading, field: Field) -> Self {
        Self { at, heading, field }
    }

    /// Whether `to` is inside it: within the near circle, or within half the
    /// cone's width of the heading. Distance is not limited here.
    pub fn sees(&self, to: Spot) -> bool {
        if self.field.is_all_round()
            || self.at.distance_squared(to) <= u128::from(self.field.near).pow(2)
        {
            return true;
        }
        Heading::towards(self.at, to).is_none_or(|way| way.apart(self.heading) <= self.field.half)
    }

    /// Whether the middle of cell `cell` is inside it.
    pub fn sees_cell(&self, cell: (i64, i64)) -> bool {
        self.sees(Spot::middle_of(cell))
    }
}
