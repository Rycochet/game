//! Abilities: named things a creature can do, which will hook onto skills. See
//! `docs/context/shared/ability.md`.
//!
//! Active ones are done when asked (and can go on the action bar); passive ones
//! work by themselves. Every creature knows Attack; the rest come with its kind.

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;

mod bar;

pub use bar::{ActionBar, BAR_SLOTS, BarChange, BarRefusal, BarSlot};

/// Something a creature can do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Ability {
    Flutter,
    Kick,
    NightVision,
    Trample,
    // New abilities go at the end: messages carry one by its place in this list.
    /// Striking with what is held, or without: every creature knows it.
    Attack,
}

/// Whether an ability is done when asked, or works by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    /// Done when asked: it can go on the action bar.
    Active,
    /// Always at work: it can't go on the action bar.
    Passive,
}

impl Ability {
    /// Every ability, in the order they are listed to players.
    pub const ALL: [Ability; 5] = [
        Self::Attack,
        Self::Flutter,
        Self::Kick,
        Self::NightVision,
        Self::Trample,
    ];

    /// What it is called.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Attack => "Attack",
            Self::Flutter => "Flutter",
            Self::Kick => "Kick",
            Self::NightVision => "Night Vision",
            Self::Trample => "Trample",
        }
    }

    /// Whether it is done when asked or works by itself.
    pub const fn use_kind(self) -> Use {
        match self {
            Self::NightVision => Use::Passive,
            Self::Attack | Self::Flutter | Self::Kick | Self::Trample => Use::Active,
        }
    }

    /// Whether it is done when asked, and so can go on the action bar.
    pub const fn is_active(self) -> bool {
        matches!(self.use_kind(), Use::Active)
    }

    /// Whether a creature in a body of kind `kind` can do it now. Only whether it
    /// is active and known, for now; later also what it costs and how soon it can
    /// be done again.
    pub fn can_use(self, kind: CreatureKind) -> bool {
        self.is_active() && known(kind).contains(&self)
    }
}

/// The abilities a creature in a body of kind `kind` knows, in `Ability::ALL`'s
/// order: Attack, and its kind's own. Shapeshifted, the form's.
pub fn known(kind: CreatureKind) -> Vec<Ability> {
    let own = kind.def().abilities;
    Ability::ALL
        .into_iter()
        .filter(|ability| *ability == Ability::Attack || own.contains(ability))
        .collect()
}

#[cfg(test)]
mod tests;
