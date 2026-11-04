//! The kinds of road, from the smallest to the largest.

use serde::{Deserialize, Serialize};

use crate::poi::PoiKind;

/// What a road is like. Ordered smallest first, so the larger of two is `max`:
/// where roads overlap, the larger is drawn on top (owner's rule).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RoadKind {
    /// A footpath, between hamlets: the colour of the land it crosses.
    Path,
    /// A dirt track, between villages.
    Track,
    /// A stone roadway, between towns.
    Road,
    /// A paved highway, between cities. Rare, as cities are.
    Highway,
}

impl RoadKind {
    /// Every kind, smallest first.
    pub const ALL: [RoadKind; 4] = [Self::Path, Self::Track, Self::Road, Self::Highway];

    /// The road a town of this kind builds, or `None` if it isn't a town.
    pub fn of_town(kind: PoiKind) -> Option<Self> {
        match kind {
            PoiKind::Hamlet => Some(Self::Path),
            PoiKind::Village => Some(Self::Track),
            PoiKind::Town => Some(Self::Road),
            PoiKind::City => Some(Self::Highway),
            _ => None,
        }
    }

    /// The road between two towns: the one the smaller of them builds (owner's
    /// choice), so a highway only joins two cities.
    pub fn between(a: PoiKind, b: PoiKind) -> Option<Self> {
        Some(Self::of_town(a)?.min(Self::of_town(b)?))
    }

    /// How much climbing counts when a road of this kind is laid, in percent of
    /// what it costs walkers. Paths follow exactly the walkers' rules; larger
    /// roads mind slopes less, so they run straighter (owner: highways are 20%
    /// more likely to change elevation).
    pub fn slope_percent(self) -> u32 {
        match self {
            Self::Path => 100,
            Self::Track => 93,
            Self::Road => 87,
            Self::Highway => 80,
        }
    }
}
