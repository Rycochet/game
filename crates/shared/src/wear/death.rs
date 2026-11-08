//! What a body drops on dying, by its difficulty (`shared::difficulty`): on
//! Hard, everything but what is worn against the skin alone; on Ironman,
//! everything; on Easy and Normal, nothing.

use crate::creature::Layer;
use crate::difficulty::Difficulty;
use crate::item::{Inventory, Item, ItemCategory};

/// Whether dying on Hard leaves this worn thing on: worn on the Skin layer
/// and no other (nothing padded), and not jewellery. Held things are never on
/// the Skin layer.
pub fn kept_on_hard(item: &Item) -> bool {
    let skin_only = item
        .kind
        .def()
        .wear
        .is_some_and(|wear| wear.layer == Layer::Skin);
    skin_only && item.padded().is_empty() && item.kind.category() != ItemCategory::Jewellery
}

impl Inventory {
    /// Takes out what dying on this difficulty drops, each with all in it:
    /// everything carried and worn, but on Hard what `kept_on_hard` keeps on.
    /// What was thrown away stays in the bin.
    pub fn drop_on_death(&mut self, difficulty: Difficulty) -> Vec<Item> {
        if !difficulty.drops_on_death() {
            return Vec::new();
        }
        let keeps = |item: &Item| difficulty == Difficulty::Hard && kept_on_hard(item);
        let mut dropped: Vec<Item> = self.items.drain(..).map(|placed| placed.item).collect();
        let (kept, worn): (Vec<Item>, Vec<Item>) =
            std::mem::take(&mut self.worn).into_iter().partition(keeps);
        self.worn = kept;
        dropped.extend(worn);
        dropped
    }
}
