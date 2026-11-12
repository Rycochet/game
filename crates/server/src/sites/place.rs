//! What a visit is to, and the cells it shares with another map.

use shared::movement::FlatGround;
use shared::poi::{ChunkPos, Poi, PoiKind};
use shared::site::Site;
use shared::wilderness::BlockPos;

use crate::storage::SiteId;

/// What a visit is to.
pub(crate) enum Place {
    /// A level of the site behind a poi; `from_wilderness` if they walked in
    /// from the wilderness, where leaving takes them back.
    Site {
        poi: Poi,
        site: Box<Site>,
        from_wilderness: bool,
    },
    /// The wilderness, its level a window of blocks round `centre`, its cells
    /// wrapping at `width`. `cutouts`: the pois set into it there. `in_poi`:
    /// the poi whose cutout the player stands in, as last told to the client.
    Wilderness {
        centre: BlockPos,
        width: i64,
        cutouts: Vec<SetIn>,
        in_poi: Option<Poi>,
    },
}

/// A poi set into a wilderness window, in the window's cells.
#[derive(Clone, Debug)]
pub(crate) struct SetIn {
    pub(crate) chunk: ChunkPos,
    pub(crate) kind: PoiKind,
    /// The cells its cutout takes: `(min_x, min_y, width, height)`.
    pub(crate) bounds: (i64, i64, i64, i64),
    /// Its way in: its first level's entrance, or the way down into a site
    /// entered from above. Seeing it makes the poi known (`wild::found`).
    pub(crate) entrance: Option<(i64, i64)>,
    /// The cells round it (`Cutout::border`).
    pub(crate) border: Vec<(i64, i64)>,
    /// Its covered ground kept level (`wilderness::Cutout::flats`).
    pub(crate) flats: Vec<FlatGround>,
}

impl Place {
    /// The ground kept level in the wilderness: its cutouts' (none elsewhere).
    #[cfg(test)]
    pub(crate) fn flats(&self) -> Vec<FlatGround> {
        match self {
            Place::Wilderness { cutouts, .. } => cutouts
                .iter()
                .flat_map(|set_in| set_in.flats.iter().copied())
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl SetIn {
    /// Whether a cell is in its cutout's bounds.
    pub(crate) fn takes(&self, (x, y): (i64, i64)) -> bool {
        let (min_x, min_y, width, height) = self.bounds;
        (min_x..min_x + width).contains(&x) && (min_y..min_y + height).contains(&y)
    }
}

/// Cells of a visit's level that are also another map's: a site's first level
/// set into the wilderness. What is seen or changed of them is kept for both.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Mirror {
    /// The other map: its site and level.
    pub(crate) id: SiteId,
    pub(crate) index: u32,
    /// Added to a cell here to give the other map's.
    pub(crate) offset: (i64, i64),
    /// The cells it covers here: `(min_x, min_y, width, height)`.
    pub(crate) bounds: (i64, i64, i64, i64),
}

impl Mirror {
    /// The other map's cell for one here, if this covers it.
    pub(crate) fn of(&self, x: i64, y: i64) -> Option<(i64, i64)> {
        let (min_x, min_y, width, height) = self.bounds;
        let inside = (min_x..min_x + width).contains(&x) && (min_y..min_y + height).contains(&y);
        inside.then_some((x + self.offset.0, y + self.offset.1))
    }
}
