//! The action bar: ten slots, each holding an active ability or nothing, kept per
//! character. Keys 1 to 0 use them, and each slot can be set to use its ability by
//! itself (`auto`). The client changes it with `BarChange`s, which the server
//! checks with the same `ActionBar::apply`.

use serde::{Deserialize, Serialize};

use super::Ability;
use crate::creature::CreatureKind;

/// How many slots the bar has.
pub const BAR_SLOTS: usize = 10;

/// One slot's ability, and whether it is used by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarSlot {
    pub ability: Ability,
    /// Used by itself rather than when its key is pressed (right-click or a long
    /// press toggles it). Forgotten when the ability leaves the bar.
    pub auto: bool,
}

/// A character's action bar.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionBar {
    pub slots: [Option<BarSlot>; BAR_SLOTS],
}

/// A change the player asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarChange {
    /// Put an ability in a slot, from the Abilities page, replacing what was there.
    /// If it was already in another slot, it moves, keeping `auto`.
    Place { slot: u8, ability: Ability },
    /// Take the ability out of a slot (dropped on the Abilities page).
    Clear { slot: u8 },
    /// Swap two slots, each ability keeping its `auto`.
    Swap { a: u8, b: u8 },
    /// Turn a slot's `auto` on or off.
    ToggleAuto { slot: u8 },
}

/// Why a change was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarRefusal {
    /// There is no such slot.
    NoSlot,
    /// Passive abilities can't go on the bar.
    Passive,
    /// The creature doesn't know it.
    Unknown,
    /// There is nothing in the slot to make automatic.
    Empty,
    /// It can't be made automatic (`Ability::can_be_automatic`).
    NotAutomatic,
}

/// A new character's: Attack in the first slot.
impl Default for ActionBar {
    fn default() -> Self {
        let mut slots = [None; BAR_SLOTS];
        slots[0] = Some(BarSlot {
            ability: Ability::Attack,
            auto: false,
        });
        Self { slots }
    }
}

impl ActionBar {
    /// Makes the change for a creature in a body of kind `kind`, or says why not
    /// (and changes nothing). An ability it no longer knows (after shapeshifting)
    /// stays where it is, only it can't be used.
    pub fn apply(&mut self, change: BarChange, kind: CreatureKind) -> Result<(), BarRefusal> {
        match change {
            BarChange::Place { slot, ability } => {
                let slot = index(slot)?;
                if !ability.is_active() {
                    return Err(BarRefusal::Passive);
                }
                if !super::known(kind).contains(&ability) {
                    return Err(BarRefusal::Unknown);
                }
                let auto = match self.slot_of(ability) {
                    Some(old) => self.slots[old].take().is_some_and(|old| old.auto),
                    None => false,
                };
                self.slots[slot] = Some(BarSlot { ability, auto });
            }
            BarChange::Clear { slot } => self.slots[index(slot)?] = None,
            BarChange::Swap { a, b } => {
                let (a, b) = (index(a)?, index(b)?);
                self.slots.swap(a, b);
            }
            BarChange::ToggleAuto { slot } => {
                let held = self.slots[index(slot)?].as_mut().ok_or(BarRefusal::Empty)?;
                if !held.auto && !held.ability.can_be_automatic() {
                    return Err(BarRefusal::NotAutomatic);
                }
                held.auto = !held.auto;
            }
        }
        Ok(())
    }

    /// The slot holding this ability, if any.
    pub fn slot_of(&self, ability: Ability) -> Option<usize> {
        self.slots
            .iter()
            .position(|held| held.is_some_and(|held| held.ability == ability))
    }
}

/// A slot's place, if there is such a slot.
fn index(slot: u8) -> Result<usize, BarRefusal> {
    let slot = slot as usize;
    if slot < BAR_SLOTS {
        Ok(slot)
    } else {
        Err(BarRefusal::NoSlot)
    }
}
