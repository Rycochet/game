//! What the pointer names another creature as.

use shared::mob::EntityLook;

use crate::common::HoverName;

/// What the pointer names another as: a player by their character's name, a
/// mob by its kind.
pub(super) fn hover_name(look: &EntityLook) -> HoverName {
    HoverName(
        look.name
            .clone()
            .unwrap_or_else(|| look.kind.def().name.to_owned()),
    )
}
