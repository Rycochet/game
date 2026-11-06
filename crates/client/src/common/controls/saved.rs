//! How `Controls` are saved: only the actions changed from their defaults, by
//! name, each with both its bindings (`Some("Shift+KeyG")`, or `None` for an
//! empty slot). Names that aren't known (an action since removed, a key
//! mistyped) are left out rather than losing the rest. An action not saved gets
//! its defaults, less any that another action has been given since.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Action, Binding, Controls, SLOTS};

type Saved = BTreeMap<String, [Option<String>; SLOTS]>;

impl Controls {
    fn to_saved(&self) -> Saved {
        Action::ALL
            .into_iter()
            .filter(|&action| self.keys(action) != action.defaults())
            .map(|action| {
                let names = self.keys(action).map(|slot| slot.map(Binding::to_name));
                (action.id().to_string(), names)
            })
            .collect()
    }

    fn from_saved(saved: &Saved) -> Self {
        let mut controls = Self::default();
        let mut changed = Vec::new();
        for (id, names) in saved {
            let Some(action) = Action::from_id(id) else {
                continue;
            };
            controls.keys[action.index()] = names
                .clone()
                .map(|name| name.and_then(|name| Binding::from_name(&name)));
            changed.push(action);
        }
        // A default (perhaps of an action added since) the player gave to
        // something else stays with that.
        for action in Action::ALL {
            if changed.contains(&action) {
                continue;
            }
            for slot in 0..SLOTS {
                let taken = controls.keys[action.index()][slot].is_some_and(|binding| {
                    changed
                        .iter()
                        .any(|&other| controls.keys(other).contains(&Some(binding)))
                });
                if taken {
                    controls.keys[action.index()][slot] = None;
                }
            }
        }
        controls
    }
}

impl Serialize for Controls {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_saved().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Controls {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_saved(&Saved::deserialize(deserializer)?))
    }
}
