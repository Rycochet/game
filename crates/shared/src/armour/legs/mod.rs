//! Armour for the legs: cuisses, poleyns, greaves, schynbalds, leg harness, chausses,
//! haidate, suneate, the dizçek, padded chausses and leg guards of every construction.
//! One item covers both legs (Legs has one place); a Classical greave may cover one.
//! See `docs/context/shared/armour.md`.

mod grid;
mod materials;
mod names;
mod parts;
mod roll;
mod tables;
mod weight;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use parts::{Cover, Knee, LegsKind, Ornament, Reinforce};
pub use roll::roll;

use super::arms::Sides;
use super::{Construction, Dye, Finish, Style};
use crate::material::Material;

/// A piece of leg armour: everything about it that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LegsForm {
    /// Leg armour, footed chausses or padded chausses: the item kind it was made for.
    pub kind: LegsKind,
    pub construction: Construction,
    pub style: Style,
    /// Which parts of the leg it covers.
    pub cover: Cover,
    /// Round the leg (hinged or laced), or over its front and outside only, strapped
    /// or tied behind: schynbalds, open-backed cuisses, demi-chausses, haidate.
    pub closed: bool,
    pub knee: Knee,
    /// Plates strapped over mail chausses, made as one item (the owner's decision).
    pub reinforce: Reinforce,
    /// Both legs, or one: a single Classical greave.
    pub sides: Sides,
    /// Fantasy spikes on the knees or fins down the shins.
    pub ornament: Ornament,
    /// Leather with rows of rivet heads.
    pub studded: bool,
    /// The plates, rings, scales, leather or padding.
    pub material: Material,
    pub finish: Finish,
    /// The colour of the cloth a piece is built on or covered with.
    pub cloth: Option<Dye>,
    /// Edges of another metal.
    pub trim: Option<Material>,
}

impl LegsForm {
    /// Japanese pieces on cloth, tied on: suneate and haidate.
    pub fn japanese(&self) -> bool {
        self.style == Style::Japanese
    }

    /// Holds its shape: plate and hardened leather, and splints.
    pub fn rigid(&self) -> bool {
        match self.construction {
            Construction::Plate | Construction::Splint => true,
            Construction::Leather => self.material == Material::HardenedLeather,
            _ => false,
        }
    }

    /// Bronze greaves cast as a coil of rings (Vietnam's Đông Sơn, kept with the
    /// Classical bronzes as the arms' coiled guards are).
    pub fn coiled(&self) -> bool {
        self.style == Style::Classical
            && self.construction == Construction::Plate
            && self.cover == Cover::Shin
            && self.closed
    }

    /// Covers the foot too: footed chausses.
    pub fn footed(&self) -> bool {
        self.kind == LegsKind::Chausses
    }
}
