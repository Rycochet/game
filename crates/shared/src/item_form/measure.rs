//! What forms say about the size and weight of their items.

use super::ItemForm;
use crate::creature::Size;
use crate::item::Shape;

impl ItemForm {
    /// The cells its item takes up at an ordinary size, for forms that decide it.
    pub fn shape(&self) -> Option<Shape> {
        match self {
            Self::Blade(blade) => Some(blade.shape()),
            Self::Shoulders(shoulders) => Some(shoulders.shape()),
            Self::Torso(torso) => Some(torso.shape()),
            Self::Head(head) => Some(head.shape()),
            Self::Arms(arms) => Some(arms.shape()),
            Self::Legs(legs) => Some(legs.shape()),
            Self::Gem(_) | Self::Ring(_) | Self::Hands(_) => None,
        }
    }

    /// What its item weighs made at this size, without anything in it, for forms that
    /// decide it.
    pub fn grams_at(&self, size: Size) -> Option<u32> {
        match self {
            Self::Blade(blade) => Some(blade.grams_at(size)),
            Self::Hands(hands) => Some(hands.grams_at(size)),
            Self::Shoulders(shoulders) => Some(shoulders.grams_at(size)),
            Self::Torso(torso) => Some(torso.grams_at(size)),
            Self::Head(head) => Some(head.grams_at(size)),
            Self::Arms(arms) => Some(arms.grams_at(size)),
            Self::Legs(legs) => Some(legs.grams_at(size)),
            Self::Gem(_) | Self::Ring(_) => None,
        }
    }
}
