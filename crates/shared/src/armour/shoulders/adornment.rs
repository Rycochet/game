//! `Adornment` and `Finish`: how shoulder armour is decorated.

use serde::{Deserialize, Serialize};

/// Something fantastic added on top, whatever the style (the owner's list).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Adornment {
    None,
    /// This many spikes along the top: 1, 3 or 5.
    Spikes(u8),
    /// A pair of curved horns.
    Horns,
    /// A beast's head, the arm coming out of its mouth, as on Chinese generals' armour.
    BeastHead,
    Skull,
}

/// How leather is finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Finish {
    Plain,
    /// Pressed and cut patterns.
    Tooled,
    /// Rows of rivet heads: studded leather.
    Studded,
    /// Rings sewn flat on it: ring mail.
    RingMail,
}
