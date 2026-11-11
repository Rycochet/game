//! What a character knows of a poi they have seen, kept with the character.

use serde::{Deserialize, Serialize};

/// What a character knows of a poi they have seen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnownPoi {
    /// Whether they have learned where the chunk's poi is. Towns are always
    /// known, and nothing teaches a character the others yet.
    pub known: bool,
    /// Whether they have been inside.
    pub visited: bool,
    /// Whether they have learned a place's name (a town's, castle's or
    /// fort's): by going into it, or (later) from being told of it
    /// (`world_map::learn_place_name`). Missing from older saves, where a place
    /// they have visited counts as named.
    #[serde(default)]
    pub named: bool,
}

impl KnownPoi {
    /// Whether they know what the poi is called (a town's, castle's or fort's
    /// name).
    pub fn knows_name(&self) -> bool {
        self.named || self.visited
    }
}
