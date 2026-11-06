//! Kinds of creature: the same for players and, later, mobs. What each kind's
//! body is like, and so what it can wear. See `docs/context/shared/creature.md`.
//!
//! A table here, as with kinds of item. Different bodies use different locations
//! (`Forelegs` isn't `Legs`), so trousers don't fit a horse, and a centaur is a
//! human top on those.

// Split into files (`cargo xtask lint` keeps files small).
mod animals;
mod bodies;
mod def;
mod guesses;
mod kind;
mod layer;
mod names;
mod peoples;
mod senses;
mod size;
mod taurs;

pub use def::{BodyPart, CreatureDef};
pub use kind::CreatureKind;
pub use layer::{Layer, Location};
pub use senses::{Acuity, DarkSight, Senses, Temper};
pub use size::Size;

impl CreatureKind {
    /// Every kind, alphabetically.
    pub const ALL: [Self; 19] = [
        Self::Beggar,
        Self::Birdfolk,
        Self::Cattle,
        Self::Centaur,
        Self::Chicken,
        Self::Drider,
        Self::Dwarf,
        Self::Elf,
        Self::Giant,
        Self::Goblin,
        Self::Halfling,
        Self::Horse,
        Self::Human,
        Self::Kobold,
        Self::Lizardfolk,
        Self::Minotaur,
        Self::Orc,
        Self::ScorpionMan,
        Self::Wolf,
    ];
}

#[cfg(test)]
mod tests;
