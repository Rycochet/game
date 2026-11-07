//! Layers of clothes, and the places on a body they go.

use serde::{Deserialize, Serialize};

/// Clothing layers, from the skin out. A layer covers those under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Layer {
    Skin,
    Middle,
    Outer,
    /// Cloth over armour: surcoats, tabards, a horse's caparison.
    Over,
}

impl Layer {
    pub const ALL: [Self; 4] = [Self::Skin, Self::Middle, Self::Outer, Self::Over];
}

/// Somewhere on a body that clothes can go.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Location {
    Head,
    Neck,
    Shoulders,
    Arms,
    Wings,
    Torso,
    Waist,
    Hands,
    Fingers,
    Legs,
    Feet,
    Forelegs,
    Hindlegs,
    Tail,
    /// For packs and cloaks: armour for the back is on `Torso`, so it never stops a
    /// pack going on.
    Back,
    /// What is held to fight with: two places, one for each hand, though neither is
    /// the left or the right (owner, 2026-11-07). A two-handed weapon takes both.
    Held,
}
