//! Who a mob has in its sights.

use shared::id::Id;

use super::{ATTACK_GAP, Doing};

impl Doing {
    /// Who this has the mob going after as an enemy, to get close to them: seen,
    /// followed down stairs or tracked by scent. Coming over to be friendly,
    /// fleeing, guarding, settling and going home have nobody in its sights.
    pub(crate) fn target(&self) -> Option<Id> {
        match *self {
            Self::Going { after, gap, .. }
            | Self::Following { after, gap, .. }
            | Self::Tracking { after, gap, .. }
                if gap == ATTACK_GAP =>
            {
                Some(after)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
