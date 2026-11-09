//! What a kind's limbs let it do.

use super::{Layout, Role};
use crate::creature::CreatureKind;

/// Whether a kind has hands to work a door's handle with: arms (not wings),
/// or a person's upper body on a beast's. A kind whose body isn't drawn yet has
/// none. (Some dogs learn doors, but rarely enough to leave out.)
pub fn has_hands(kind: CreatureKind) -> bool {
    Layout::of(kind).is_some_and(|layout| {
        layout.upper.is_some() || layout.limbs().any(|limb| limb.role == Role::Arm)
    })
}
