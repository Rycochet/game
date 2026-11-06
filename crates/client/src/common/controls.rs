//! Which keys do what (Settings, Controls): two bindings for each action, kept
//! in `Preferences`. Esc is not here: it always opens the menu and closes what
//! is on top, so it can't be changed.

mod action;
mod binding;
mod input;
mod key_names;
mod saved;

pub use action::{Action, Group};
pub use binding::{Binding, Mods};

/// Bindings each action has (owner: two keys each; a gamepad's button will be
/// a third, of its own).
pub const SLOTS: usize = 2;

/// The keys of every action. The default is the game's own; only the actions
/// changed from it are saved (`saved`), so actions added later get theirs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Controls {
    keys: [[Option<Binding>; SLOTS]; Action::ALL.len()],
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            keys: Action::ALL.map(Action::defaults),
        }
    }
}

impl Controls {
    /// The action's bindings, in order.
    pub fn keys(&self, action: Action) -> [Option<Binding>; SLOTS] {
        self.keys[action as usize]
    }

    /// Gives the action this binding (or none) in this slot. Whatever else had
    /// the same binding loses it: those places are returned, so they can show it.
    pub fn set(
        &mut self,
        action: Action,
        slot: usize,
        binding: Option<Binding>,
    ) -> Vec<(Action, usize)> {
        let mut lost = Vec::new();
        if let Some(binding) = binding {
            for other in Action::ALL {
                for other_slot in 0..SLOTS {
                    let same_place = other == action && other_slot == slot;
                    let place = &mut self.keys[other as usize][other_slot];
                    if !same_place && *place == Some(binding) {
                        *place = None;
                        lost.push((other, other_slot));
                    }
                }
            }
        }
        self.keys[action as usize][slot] = binding;
        lost
    }

    /// Every binding, with whose it is.
    fn all(&self) -> impl Iterator<Item = (Action, Binding)> + '_ {
        Action::ALL.into_iter().flat_map(move |action| {
            self.keys(action)
                .into_iter()
                .flatten()
                .map(move |binding| (action, binding))
        })
    }
}

#[cfg(test)]
mod tests;
