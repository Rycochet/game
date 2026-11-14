//! `Change`: how older saves kept a raised or lowered most, read only to be
//! turned into an effect (`bonus::Effect`) by `Sheet::settled`.

use serde::{Deserialize, Serialize};

use super::Vital;

/// Something raising or lowering the most of one: what the debug menu's Bonus
/// made before effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Change {
    pub vital: Vital,
    pub amount: i32,
    /// The action tick it ends at, or `None` for good.
    pub until: Option<u64>,
}
