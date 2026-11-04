//! Armour for the hands, fingers included: gauntlets of plate, mail, ring mail, scale,
//! leather and plates in cloth, demi-gauntlets and tekkō on the back of the hand only, and padded
//! gloves worn under them. What a piece is like is rolled once and saved; how big a
//! hand is comes from the body frame, so the same piece is drawn alone and worn. See
//! `docs/context/shared/armour.md`.

use serde::{Deserialize, Serialize};

use super::{Construction, Style};
use crate::material::Material;

mod grid;
mod hand;
mod names;
mod parts;
mod roll;
mod tables;
mod weight;

pub use hand::Hand;
pub use parts::{Cuff, Fingers, HandsPiece, Knuckles};
pub use roll::roll;

// What the files here share, as `super::...`.
use tables::{
    cloths_for, constructions, cuffs_for, fingers_for, knuckles_for, materials_for, may_have_claws,
    may_have_trim, may_lock, needs_cloth,
};

/// A pair of hand pieces: everything about them that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HandsForm {
    pub construction: Construction,
    pub fingers: Fingers,
    pub cuff: Cuff,
    pub knuckles: Knuckles,
    /// Claws on the fingertips (fantasy, after the tekkō-kagi).
    pub claws: bool,
    /// The fingers close and lock on a grip, for tournaments: plate mittens only.
    pub locking: bool,
    /// What it is mostly made of: the plates, rings, scales, leather or padding.
    pub material: Material,
    /// Cloth over the plates (plates in cloth) or under them (a tekkō's base).
    pub cloth: Option<Material>,
    /// Edges of another metal.
    pub trim: Option<Material>,
    pub style: Style,
}

#[cfg(test)]
mod tests;
