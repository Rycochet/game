//! How a character changes its kind.

use serde::{Deserialize, Serialize};

/// What `ClientMessage::DevSetCreature` does: becomes the kind for good, or
/// shapeshifts into its body for a while (`Sheet::shapeshift`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KindChange {
    /// A new creature of that kind: its own kind, a fresh sheet.
    #[default]
    Permanent,
    /// Its body only: the form's Body row, its own Mind and Spirit, kept to change
    /// back into.
    Temporary,
}
