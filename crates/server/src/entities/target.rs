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

    /// Whether it has given up fighting: at rest, settling or going home. Who
    /// harmed it is forgotten then (`HarmedBy`), so a later fight starts afresh.
    pub(crate) fn gave_up(&self) -> bool {
        matches!(
            self,
            Self::Idle | Self::Settling { .. } | Self::GoingHome { .. }
        )
    }
}

#[cfg(test)]
mod tests;
