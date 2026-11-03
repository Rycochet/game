//! What may be secret about an item: magic, and being blessed or cursed (`Favour`, `Known`).

use serde::{Deserialize, Serialize};

use crate::item::Item;

/// How likely something that can be magical is to be, in percent. First guess.
const MAGICAL_PERCENT: u64 = 5;

/// How likely something wearable is to be blessed, and to be cursed, in percent.
/// First guesses.
const BLESSED_PERCENT: u64 = 5;
const CURSED_PERCENT: u64 = 5;

/// What can lie on an item besides magic: never both, so there is one field for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Favour {
    /// Only its wearer can take it off.
    Blessed,
    /// Nobody can take it off, once it is on.
    Cursed,
}

/// What has been found out about an item, for everyone from then on. What hasn't
/// isn't sent to clients: see `Item::hide_secrets`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Known {
    pub magic: bool,
    pub favour: bool,
}

impl Item {
    /// Whether this could be magical: only what is worn, better than common.
    pub fn can_be_magical(&self) -> bool {
        self.kind.def().wear.is_some() && self.quality.is_some_and(|q| q.magic_reach() > 0)
    }

    /// Whether it is magical. What is flagged but can't be counts for nothing.
    pub fn is_magical(&self) -> bool {
        self.made.magical && self.can_be_magical()
    }

    /// Rolls whether a new thing to wear is magical, and blessed or cursed, going by
    /// its id. Only for what is found: not for what is asked for by name.
    pub fn roll_secrets(&mut self) {
        if self.kind.def().wear.is_none() {
            return;
        }
        let mut rng = self.id.seed().derive("item secrets").rng();
        self.made.magical = rng.range_u64(0, 100) < MAGICAL_PERCENT && self.can_be_magical();
        let favour = rng.range_u64(0, 100);
        self.favour = if favour < BLESSED_PERCENT {
            Some(Favour::Blessed)
        } else if favour < BLESSED_PERCENT + CURSED_PERCENT {
            Some(Favour::Cursed)
        } else {
            None
        };
    }

    /// As a client may see it, and everything in it: what hasn't been found out
    /// looks as if it wasn't there.
    pub fn hide_secrets(&mut self) {
        if !self.known.magic {
            self.made.magical = false;
        }
        if !self.known.favour {
            self.favour = None;
        }
        for placed in &mut self.contents {
            placed.item.hide_secrets();
        }
    }
}
