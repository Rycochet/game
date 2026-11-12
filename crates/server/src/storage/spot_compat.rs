//! Reading positions on levels saved before they were in units (free
//! positions, PR 3): a saved cell is read as its middle, so no save is wiped.

use serde::{Deserialize, Deserializer};
use shared::spot::Spot;

/// A position as saved: in units now, or a whole cell in older saves. The
/// cell is tried first: RON would read a bare pair as a `Spot` too.
#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum SpotOrCell {
    Cell((i64, i64)),
    Spot(Spot),
}

impl From<SpotOrCell> for Spot {
    fn from(saved: SpotOrCell) -> Self {
        match saved {
            SpotOrCell::Spot(spot) => spot,
            SpotOrCell::Cell(cell) => Spot::middle_of(cell),
        }
    }
}

/// For `#[serde(deserialize_with)]` on a `Spot`.
pub fn spot<'de, D: Deserializer<'de>>(reader: D) -> Result<Spot, D::Error> {
    SpotOrCell::deserialize(reader).map(Spot::from)
}

/// For `#[serde(deserialize_with)]` on an `Option<Spot>` (with `default`).
pub fn optional_spot<'de, D: Deserializer<'de>>(reader: D) -> Result<Option<Spot>, D::Error> {
    Option::<SpotOrCell>::deserialize(reader).map(|saved| saved.map(Spot::from))
}
