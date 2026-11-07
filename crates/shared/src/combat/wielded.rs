//! `Wielded`: what a creature fights with, from what it holds.

use crate::creature::Location;
use crate::item::Inventory;

use super::Weapon;

/// The weapons in a creature's hands: the main hand's always the bigger (owner,
/// 2026-11-07: nobody is left or right handed), and an off hand only when both
/// hold one. Bare hands when nothing is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wielded {
    pub main: Weapon,
    pub off: Option<Weapon>,
}

impl Wielded {
    /// What this inventory holds (`Location::Held`).
    pub fn of(inventory: &Inventory) -> Self {
        let mut held: Vec<Weapon> = inventory
            .worn
            .iter()
            .filter(|item| {
                item.kind
                    .def()
                    .wear
                    .is_some_and(|wear| wear.locations.contains(&Location::Held))
            })
            .filter_map(Weapon::of)
            .collect();
        // Heaviest to swing first; ties by weight, then length, so both sides agree.
        held.sort_by_key(|weapon| {
            std::cmp::Reverse((weapon.handling_grams(), weapon.grams, weapon.length_mm))
        });
        Self {
            main: held.first().copied().unwrap_or(Weapon::UNARMED),
            off: held.get(1).copied(),
        }
    }

    /// Whether it has a weapon in each hand, so dual wielding counts.
    pub fn is_dual(&self) -> bool {
        self.off.is_some()
    }

    /// The weapon making the `n`th attack: hands take turns when dual wielding,
    /// the main hand first. With `true` for the off hand.
    pub fn for_attack(&self, n: u64) -> (Weapon, bool) {
        match self.off {
            Some(off) if n % 2 == 1 => (off, true),
            _ => (self.main, false),
        }
    }
}
