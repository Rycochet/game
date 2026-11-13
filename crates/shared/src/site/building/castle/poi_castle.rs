//! A castle as the world map shows it, beside its wall (`PoiWall`).

use serde::{Deserialize, Serialize};

use super::Castle;

/// What the world map shows of a castle besides its wall: its towers (where
/// they stand comes from the wall, by `tower_spots`) and its moat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PoiCastle {
    /// How many floors its tallest tower has.
    pub tower_floors: u8,
    /// How wide its moat is, in cells; 0 for none.
    pub moat: u8,
}

impl PoiCastle {
    pub(in crate::site) fn of(castle: &Castle) -> Self {
        Self {
            tower_floors: castle.tower_floors() as u8,
            moat: castle.moat as u8,
        }
    }
}
