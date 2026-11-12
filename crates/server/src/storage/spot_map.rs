//! Things kept by where they are on a level, in units: bags on the ground, and
//! the bags a character remembers. Saves from before bags left cells (free
//! positions, PR 6) keep them by cell, read as the cell's middle.

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use shared::spot::Spot;

use super::spot_compat::SpotOrCell;

/// A map by `Spot`, saved as a plain map, whose older saves' cell keys load as
/// their middles. (A newtype: a wrapper struct, so it can read itself its own
/// way while working like the map inside, through `Deref`.)
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SpotMap<V>(pub BTreeMap<Spot, V>);

impl<V> Default for SpotMap<V> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<V> Deref for SpotMap<V> {
    type Target = BTreeMap<Spot, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for SpotMap<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V> FromIterator<(Spot, V)> for SpotMap<V> {
    fn from_iter<I: IntoIterator<Item = (Spot, V)>>(entries: I) -> Self {
        Self(entries.into_iter().collect())
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for SpotMap<V> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_map(Entries(PhantomData))
    }
}

/// Reads the entries one by one, each key a spot or an old save's cell.
struct Entries<V>(PhantomData<V>);

impl<'de, V: Deserialize<'de>> Visitor<'de> for Entries<V> {
    type Value = SpotMap<V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map by spot or cell")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
        let mut map = BTreeMap::new();
        while let Some((key, value)) = entries.next_entry::<SpotOrCell, V>()? {
            map.insert(Spot::from(key), value);
        }
        Ok(SpotMap(map))
    }
}
