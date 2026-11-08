//! Whether an action's keys are down, from Bevy's `ButtonInput<KeyCode>` or
//! from one key's event. A binding with modifiers needs them held; one without
//! works whatever is held, unless another binding of the same key needing more
//! of what is held matches instead (Shift + G, not G, while Shift is down).

use bevy::prelude::*;

use super::binding::sides;
use super::{Action, Binding, Controls, Mods};

impl Controls {
    /// Whether some binding of `key` needs more of `held` than `binding` does.
    fn outdone(&self, binding: Binding, held: Mods) -> bool {
        self.all().any(|(_, other)| {
            other.key == binding.key
                && other.mods != binding.mods
                && other.mods.contains(binding.mods)
                && other.fits(held)
        })
    }

    /// The action's bindings that count with `held` down.
    fn counting(&self, action: Action, held: Mods) -> impl Iterator<Item = Binding> + '_ {
        self.keys(action)
            .into_iter()
            .flatten()
            .filter(move |binding| binding.fits(held) && !self.outdone(*binding, held))
    }

    /// Whether one of its keys is held down.
    pub fn pressed(&self, keys: &ButtonInput<KeyCode>, action: Action) -> bool {
        let held = Mods::held(keys);
        self.counting(action, held)
            .any(|binding| keys.any_pressed(sides(binding.key)))
    }

    /// Whether one of its keys went down this frame.
    pub fn just_pressed(&self, keys: &ButtonInput<KeyCode>, action: Action) -> bool {
        let held = Mods::held(keys);
        self.counting(action, held)
            .any(|binding| keys.any_just_pressed(sides(binding.key)))
    }

    /// Held, or tapped and let go within this frame (which `pressed` misses).
    pub fn active(&self, keys: &ButtonInput<KeyCode>, action: Action) -> bool {
        self.pressed(keys, action) || self.just_pressed(keys, action)
    }

    /// As `just_pressed`, and takes the press, so nothing after sees it.
    pub fn clear_just_pressed(&self, keys: &mut ButtonInput<KeyCode>, action: Action) -> bool {
        let held = Mods::held(keys);
        let bound: Vec<Binding> = self.counting(action, held).collect();
        let mut taken = false;
        for binding in bound {
            for side in sides(binding.key) {
                taken |= keys.clear_just_pressed(side);
            }
        }
        taken
    }

    /// Lets go of all its keys, whatever is held with them, so nothing after
    /// sees them down, held or pressed, until they are pressed again.
    pub fn release(&self, keys: &mut ButtonInput<KeyCode>, action: Action) {
        for binding in self.keys(action).into_iter().flatten() {
            for side in sides(binding.key) {
                keys.reset(side);
            }
        }
    }

    // Only Full Screen uses it, natively.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    /// Whether a key going down, with `held` down (as its own events say), is
    /// one of the action's.
    pub fn is_key_of(&self, action: Action, key: KeyCode, held: Mods) -> bool {
        let key = sides(key)[0];
        self.counting(action, held)
            .any(|binding| binding.key == key)
    }

    /// Whether the key is in any of the action's bindings, whatever is held.
    pub fn uses_key(&self, action: Action, key: KeyCode) -> bool {
        let key = sides(key)[0];
        self.keys(action)
            .into_iter()
            .flatten()
            .any(|binding| binding.key == key)
    }
}
