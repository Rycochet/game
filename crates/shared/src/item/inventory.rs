//! `Inventory`: everything a character carries, in their own grid and what they wear.

mod changes;
mod cost;
mod lookup;
mod parts;
mod room;
mod rules;
mod sort;
mod stacking;

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;
use crate::item::item::weight_of;
use crate::item::{INVENTORY_SIZE, Item, Placed};

/// Everything a character carries: what is in their own grid and what they wear.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    /// What kind of creature is carrying it all, which decides what they can wear.
    pub creature: CreatureKind,
    /// They have had their first clothes.
    pub outfitted: bool,
    /// What is in their own grid, not worn. Saved as `pockets` before it was
    /// named the inventory: the alias reads those, until the item wipe.
    #[serde(alias = "pockets")]
    pub items: Vec<Placed>,
    /// How big their own grid is, from their kind of creature.
    #[serde(alias = "pockets_size")]
    pub size: (u8, u8),
    /// What they have on: see `shared::wear`.
    pub worn: Vec<Item>,
    /// What they last threw away: it can be taken back until something else is
    /// thrown away, which destroys it. It weighs nothing, being left behind.
    #[serde(default)]
    pub bin: Option<Item>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            creature: CreatureKind::default(),
            outfitted: false,
            items: Vec::new(),
            size: INVENTORY_SIZE,
            worn: Vec::new(),
            bin: None,
        }
    }
}

impl Inventory {
    /// What it all weighs, in grams.
    pub fn total_weight(&self) -> u32 {
        weight_of(&self.items) + self.worn.iter().map(Item::total_weight).sum::<u32>()
    }

    /// What the creature weighs with all it carries and wears, in grams.
    pub fn weight_with_creature(&self) -> u32 {
        self.creature.def().weight + self.total_weight()
    }
}
