//! `Foes`: what a character remembers of fighting each kind of creature: the 5-second
//! blocks spent fighting each (`skill::Windows`; a block counts if they hit one of
//! that kind, or were hit by one), which make practice against it fall off
//! (`skill::familiarity`), and how many they have helped kill.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;

/// What is remembered of one kind of foe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Foe {
    /// The 5-second blocks spent fighting this kind.
    #[serde(default)]
    pub fought: u32,
    /// How many of this kind they have helped kill.
    #[serde(default)]
    pub defeated: u32,
}

/// Each kind of creature fought, and what is remembered of it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Foes(pub BTreeMap<CreatureKind, Foe>);

impl Foes {
    /// What is remembered of this kind: nothing if it has never been fought.
    pub fn of(&self, kind: CreatureKind) -> Foe {
        self.0.get(&kind).copied().unwrap_or_default()
    }

    /// Another block spent fighting this kind.
    pub fn fought(&mut self, kind: CreatureKind) {
        let foe = self.0.entry(kind).or_default();
        foe.fought = foe.fought.saturating_add(1);
    }

    /// Another of this kind killed, with their help.
    pub fn defeated(&mut self, kind: CreatureKind) {
        let foe = self.0.entry(kind).or_default();
        foe.defeated = foe.defeated.saturating_add(1);
    }
}
