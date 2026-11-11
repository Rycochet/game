//! The keys 1 to 0 (or whatever Settings has) using the bar's slots. (The
//! ability being dragged is drawn by `dom`.)

use bevy::prelude::*;
use shared::ability::BAR_SLOTS;

use super::use_ability::UseSlot;
use crate::common::{Action, Preferences};

/// A slot's key uses it.
pub(super) fn use_by_key(
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    mut used: MessageWriter<UseSlot>,
) {
    for slot in 0..BAR_SLOTS as u8 {
        if preferences
            .controls
            .just_pressed(&keys, Action::UseSlot(slot))
        {
            used.write(UseSlot(slot));
        }
    }
}
