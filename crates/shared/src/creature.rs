//! Kinds of creature: the same for players and, later, mobs. What each kind's
//! body is like, and so what it can wear. See `docs/context/shared/creature.md`.
//!
//! A table here, as with kinds of item. Different bodies use different locations
//! (`Forelegs` isn't `Legs`), so trousers don't fit a horse, and a centaur is a
//! human top on those.

// Split into files (`cargo xtask lint` keeps files small).
mod bodies;
mod def;
mod kind;
mod layer;
mod peoples;
mod size;

pub use def::{BodyPart, CreatureDef};
pub use kind::CreatureKind;
pub use layer::{Layer, Location};
pub use size::Size;

impl CreatureKind {
    /// Every kind, alphabetically.
    pub const ALL: [Self; 15] = [
        Self::Beggar,
        Self::Birdfolk,
        Self::Centaur,
        Self::Chicken,
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
    ];
}

#[cfg(test)]
mod tests;
