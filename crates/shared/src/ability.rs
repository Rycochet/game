//! Abilities: named things a creature can do, which will hook onto skills. See
//! `docs/context/shared/ability.md`.
//!
//! Active ones are done when asked (and can go on the action bar), toggles
//! turned on and off there; passive ones work by themselves. Every creature
//! knows Attack, Heal and Sneak; the rest come with its kind.

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;
use crate::sheet::Attribute;

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
    /// Mana turned into Health, on yourself or another (`magic`): every
    /// creature knows it for now, as there is no way to learn spells yet.
    Heal,
    /// Moving slowly and quietly, under the pace Sneaking is learned at
    /// (`energy::SNEAK_TICKS_PER_CELL`): a toggle every creature knows.
    Sneak,
}

/// Whether an ability is done when asked, or works by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    /// Done when asked: it can go on the action bar.
    Active,
    /// Always at work: it can't go on the action bar.
    Passive,
    /// Turned on and off when asked, on the action bar: on until turned off.
    Toggle,
}

impl Ability {
    /// Every ability, in the order they are listed to players.
    pub const ALL: [Ability; 7] = [
        Self::Attack,
        Self::Heal,
        Self::Sneak,
        Self::Flutter,
        Self::Kick,
        Self::NightVision,
        Self::Trample,
    ];

    /// What it is called.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Attack => "Attack",
            Self::Heal => "Heal",
            Self::Flutter => "Flutter",
            Self::Kick => "Kick",
            Self::NightVision => "Night Vision",
            Self::Trample => "Trample",
            Self::Sneak => "Sneak",
        }
    }

    /// Whether it is done when asked or works by itself.
    pub const fn use_kind(self) -> Use {
        match self {
            Self::NightVision => Use::Passive,
            Self::Sneak => Use::Toggle,
            Self::Attack | Self::Heal | Self::Flutter | Self::Kick | Self::Trample => Use::Active,
        }
    }

    /// The attributes it leans on (first guesses, owner OK 2026-11-07), which colour
    /// it: Attack as the combat skills (it may follow the weapon held's skill later),
    /// the kinds' own by what their bodies do.
    pub const fn leans(self) -> &'static [Attribute] {
        use Attribute::*;
        match self {
            Self::Attack | Self::Kick => &[Strength, Agility],
            Self::Trample => &[Strength, Toughness],
            Self::Flutter => &[Agility],
            Self::NightVision => &[Perception],
            Self::Sneak => &[Agility],
            // As its skill, Healing, under Divine.
            Self::Heal => &[Empathy],
        }
    }

    /// What it does, in a line for its tip.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Attack => "Strikes your target with what you hold, or without.",
            Self::Heal => "Heals you or your allies, and tries to keep you alive if you ask.",
            Self::Flutter => "Takes to the air for a moment.",
            Self::Kick => "Kicks your target a step away, or the closed door beside you.",
            Self::NightVision => "Sees in the dark.",
            Self::Trample => "Charges down whoever is in the way.",
            Self::Sneak => "Moves slowly and quietly while it is on.",
        }
    }

    /// The Mana it spends, in whole points (`magic`).
    pub const fn mana_cost(self) -> u32 {
        match self {
            Self::Heal => crate::magic::HEAL_MANA,
            _ => 0,
        }
    }

    /// Whether it is done (or turned on and off) when asked, and so can go on
    /// the action bar.
    pub const fn is_active(self) -> bool {
        matches!(self.use_kind(), Use::Active | Use::Toggle)
    }

    /// Whether it is turned on and off rather than done.
    pub const fn is_toggle(self) -> bool {
        matches!(self.use_kind(), Use::Toggle)
    }

    /// Whether it may be made automatic on the action bar, to be done whenever it
    /// can be (owner, 2026-11-07): only what is safe to leave on, as it waits
    /// until it can be done. Attack (a target, in reach, an enemy) and Heal
    /// (on yourself, at a quarter of your Health) so far.
    pub const fn can_be_automatic(self) -> bool {
        matches!(self, Self::Attack | Self::Heal)
    }

    /// Whether a creature in a body of kind `kind` can do it now. Only whether it
    /// is active and known, for now; later also what it costs and how soon it can
    /// be done again.
    pub fn can_use(self, kind: CreatureKind) -> bool {
        self.is_active() && known(kind).contains(&self)
    }
}

/// What every creature knows, whatever its kind.
const EVERYONE_KNOWS: [Ability; 3] = [Ability::Attack, Ability::Heal, Ability::Sneak];

/// The abilities a creature in a body of kind `kind` knows, in `Ability::ALL`'s
/// order: Attack, Heal, Sneak, and its kind's own. Shapeshifted, the form's.
pub fn known(kind: CreatureKind) -> Vec<Ability> {
    let own = kind.def().abilities;
    Ability::ALL
        .into_iter()
        .filter(|ability| {
            EVERYONE_KNOWS.contains(ability)
                || own.contains(ability)
                // Anyone with legs can kick (owner, 2026-11-09).
                || (*ability == Ability::Kick && crate::body::has_legs(kind))
        })
        .collect()
}

#[cfg(test)]
mod tests;
