//! How tall a creature stands beside its kind's own usual height, from its Height look
//! (owner, 2026-11-14): 5% a step away from Average, so Very short is 90% and Very
//! tall 110%. Only its model changes: its footprint and rules go by its kind.

use crate::creature::CreatureKind;

use super::{Appearance, Trait};

/// Thousandths, as `body::Layout::scaled` takes.
pub const HEIGHT_WHOLE: i32 = 1000;

/// Each Height choice's share of the kind's usual height, in thousandths, in the
/// choices' order: Average, Very short, Short, Tall, Very tall.
pub const HEIGHT_SHARES: [i32; 5] = [1000, 900, 950, 1050, 1100];

impl Appearance {
    /// How tall it stands beside its kind's usual height, in thousandths: its Height
    /// look's share, or the whole for a kind without one.
    pub fn height_share(&self, kind: CreatureKind) -> i32 {
        if !kind.def().looks.contains(&Trait::Height) {
            return HEIGHT_WHOLE;
        }
        let chosen = self.0.get(&Trait::Height).copied().unwrap_or(0);
        HEIGHT_SHARES
            .get(usize::from(chosen))
            .copied()
            .unwrap_or(HEIGHT_WHOLE)
    }
}

#[cfg(test)]
mod tests;
