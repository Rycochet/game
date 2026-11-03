//! Players in sites: entering them, moving about, taking stairs and leaving,
//! and sending each player the cells of their level as they come to see
//! them (or are about to). What a player has seen is kept with their
//! character; each site keeps its generator version, when its levels were
//! last visited and players' changes (`storage::SiteSave`).

use bevy::prelude::*;
use shared::dungeon::Sight;
use shared::movement::{Step, step_on_level};
use shared::poi::{Poi, PoiKind};
use shared::protocol::SeenStairs;
use shared::site::{Exit, Site, SiteLevel};

use crate::storage::{CellMasks, SiteId, VisitSave};

// Split into files (`cargo xtask lint` keeps files small).
mod cells;
mod enter;
mod keepers;
mod leave;
mod levels;
mod moves;
mod sight;
mod trade;

use cells::{add_masks, all_cells, beside, level_chunks, nearby_cells, send_cells};
use enter::enter_site;
pub(super) use enter::{dev_enter, enter_poi, generate, resume};
use keepers::{NewKeepers, keepers_of};
pub(super) use keepers::{give_to_keeper, keeper_building, keeper_stock, take_from_keeper};
use leave::leave_for_poi;
pub(super) use leave::{forget_site, leave, put_out, use_exit};
use levels::{NewVisit, arrive_again, enter_level};
pub(super) use moves::{set_cell, stepped, teleport};
pub(super) use sight::{forget_level, reveal_level, truesight_changed};
use sight::{memory, update_sight};
pub(super) use trade::open_trade;

/// How far in a town others can be made out, in cells.
const TOWN_SIGHT: u32 = 40;

/// A player's stay on a site level.
pub(crate) struct Visit {
    /// The poi the site is behind.
    poi: Poi,
    id: SiteId,
    site: Site,
    index: u32,
    level: SiteLevel,
    /// Its stairs as players see them.
    stairs: Vec<SeenStairs>,
    /// Where the player is.
    position: (i64, i64),
    /// The cells the client has been sent on this visit.
    sent: CellMasks,
    /// The cells already looked out from for what to send, and what was seen
    /// from them, so each step only looks from the cells that are new.
    looked_from: Sight,
    loaded: Sight,
    /// What can be seen now, by the game's rule: who else is in sight. Worked out
    /// from `view_from`, and only again once the player has moved.
    view: Sight,
    view_from: Option<(i64, i64)>,
}

impl Visit {
    /// Where the player is on the level.
    pub(crate) fn position(&self) -> (i64, i64) {
        self.position
    }

    /// The level's first stairs, for tests.
    #[cfg(test)]
    pub(crate) fn first_stairs(&self) -> (i64, i64) {
        let stairs = self.level.stairs[0];
        (stairs.x, stairs.y)
    }

    /// A walkable cell the player can't see, for tests.
    #[cfg(test)]
    pub(crate) fn walkable_out_of_sight(&mut self) -> (i64, i64) {
        const SEARCH: i64 = 200;
        (-SEARCH..SEARCH)
            .flat_map(|y| (-SEARCH..SEARCH).map(move |x| (x, y)))
            .find(|&(x, y)| self.level.level.is_walkable(x, y) && !self.can_see(x, y))
            .expect("a site has more than one room")
    }

    /// A cell with room for this many steps east of it, for tests.
    #[cfg(test)]
    pub(crate) fn room_to_walk_east(&self, steps: i64) -> (i64, i64) {
        const SEARCH: i64 = 200;
        (-SEARCH..SEARCH)
            .flat_map(|y| (-SEARCH..SEARCH).map(move |x| (x, y)))
            .find(|&(x, y)| (0..=steps).all(|dx| self.level.level.is_walkable(x + dx, y)))
            .expect("a site has a room")
    }

    /// Where a step from here lands, if it can be taken.
    pub(crate) fn landing(&self, step: Step) -> Option<(i64, i64)> {
        step_on_level(&self.level.level, self.position, step)
    }

    /// Which level of the site it is.
    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    /// Which site it is, for the debug API.
    #[cfg(feature = "debug-api")]
    pub(crate) fn site_id(&self) -> SiteId {
        self.id
    }

    /// The level's cells as the server has them, for the debug API.
    #[cfg(feature = "debug-api")]
    pub(crate) fn cells(&self) -> &shared::dungeon::Level {
        &self.level.level
    }

    /// Whether something at this cell can be seen from where the player is.
    pub(crate) fn can_see(&mut self, x: i64, y: i64) -> bool {
        if self.view_from != Some(self.position) {
            let (from_x, from_y) = self.position;
            let level = &self.level.level;
            self.view = Sight::new(level);
            // Towns are big, and work out no sight otherwise.
            let reach = self.is_town().then_some(TOWN_SIGHT);
            self.view.add_view(level, from_x, from_y, reach);
            self.view_from = Some(self.position);
        }
        self.view.is_seen(x, y)
    }

    /// Which site it is.
    pub(crate) fn id(&self) -> SiteId {
        self.id
    }

    /// What kind of poi it is behind.
    pub(crate) fn kind(&self) -> PoiKind {
        self.poi.kind
    }

    /// Whether the site is a town, whose layout players always know.
    fn is_town(&self) -> bool {
        self.site.kind().is_town()
    }

    /// What is saved with the character.
    pub(crate) fn save(&self) -> VisitSave {
        VisitSave {
            site: self.id,
            level: self.index,
            position: self.position,
        }
    }
}

/// Where on a level a player arrives.
enum Arrival {
    /// Coming from the world map or another level.
    From(Exit),
    /// Back where they were (a saved visit), if it is still walkable.
    At((i64, i64)),
}

#[cfg(test)]
mod tests;
