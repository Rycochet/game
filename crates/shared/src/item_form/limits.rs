//! What whoever asks for an item needs of it, for rolling within.

use crate::creature::Size;
use crate::item::Shape;

/// What whoever asks for an item needs of it. Anything left out doesn't matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// A space it must fit in, turned either way.
    pub fits: Option<(u8, u8)>,
    /// How many cells its long side must at least be.
    pub at_least_long: Option<u8>,
    pub most_grams: Option<u32>,
    pub least_grams: Option<u32>,
    /// Whose size to make it nearest, if it has to be scaled.
    pub wielder: Option<Size>,
}

impl Limits {
    pub const NONE: Self = Self {
        fits: None,
        at_least_long: None,
        most_grams: None,
        least_grams: None,
        wielder: None,
    };

    /// Whether something of this shape and weight will do.
    pub fn allows(&self, shape: Shape, grams: u32) -> bool {
        let long = shape.width.max(shape.height);
        let short = shape.width.min(shape.height);
        let fits = self
            .fits
            .is_none_or(|(one, other)| short <= one.min(other) && long <= one.max(other));
        fits && self.at_least_long.is_none_or(|least| long >= least)
            && self.most_grams.is_none_or(|most| grams <= most)
            && self.least_grams.is_none_or(|least| grams >= least)
    }
}
