//! `Heraldry`: two tinctures and a charge, for ailettes now and shields and surcoats later.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

/// The colours of heraldry: two metals and five colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tincture {
    Or,
    Argent,
    Gules,
    Azure,
    Vert,
    Sable,
    Purpure,
}

impl Tincture {
    pub const ALL: [Self; 7] = [
        Self::Or,
        Self::Argent,
        Self::Gules,
        Self::Azure,
        Self::Vert,
        Self::Sable,
        Self::Purpure,
    ];

    /// Gold and silver: heraldry puts metal on colour and colour on metal.
    pub fn metal(self) -> bool {
        matches!(self, Self::Or | Self::Argent)
    }
}

/// What is painted on the field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Charge {
    Plain,
    Cross,
    Saltire,
    Pale,
    Fess,
    Bend,
    Chevron,
}

impl Charge {
    pub const ALL: [Self; 7] = [
        Self::Plain,
        Self::Cross,
        Self::Saltire,
        Self::Pale,
        Self::Fess,
        Self::Bend,
        Self::Chevron,
    ];
}

/// A simple coat of arms. Kept apart so shields and surcoats can share it later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Heraldry {
    pub field: Tincture,
    pub charge: Charge,
    /// The charge's colour: the field's when plain.
    pub colour: Tincture,
}

impl Heraldry {
    /// Metal on colour or colour on metal, and nothing to colour when plain.
    pub fn allowed(&self) -> bool {
        if self.charge == Charge::Plain {
            self.colour == self.field
        } else {
            self.colour.metal() != self.field.metal()
        }
    }

    pub fn roll(rng: &mut Rng) -> Self {
        let field = Tincture::ALL[rng.range_u64(0, 7) as usize];
        let charge = rng.pick_weighted(&[
            (Charge::Plain, 10),
            (Charge::Cross, 20),
            (Charge::Saltire, 12),
            (Charge::Pale, 12),
            (Charge::Fess, 14),
            (Charge::Bend, 16),
            (Charge::Chevron, 16),
        ]);
        let colour = if charge == Charge::Plain {
            field
        } else {
            let others: Vec<_> = Tincture::ALL
                .into_iter()
                .filter(|other| other.metal() != field.metal())
                .collect();
            others[rng.range_u64(0, others.len() as u64) as usize]
        };
        Self {
            field,
            charge,
            colour,
        }
    }
}
