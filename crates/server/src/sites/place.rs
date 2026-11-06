//! What a visit is to, and the cells it shares with another map.

use shared::poi::{ChunkPos, Poi};
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
    /// wrapping at `width`. `cutouts`: the pois set into it there, and the
    /// cells they take (seeing one makes the poi known).
    Wilderness {
        centre: BlockPos,
        width: i64,
        cutouts: Vec<(ChunkPos, (i64, i64, i64, i64))>,
    },
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
