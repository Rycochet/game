//! Players in sites: entering them, moving about, taking stairs and leaving,
//! and sending each player the cells of their level as they come to see
//! them (or are about to). What a player has seen is kept with their
//! character; each site keeps its generator version, when its levels were
//! last visited and players' changes (`storage::SiteSave`).

use bevy::prelude::*;
use shared::dungeon::Sight;
use shared::movement::{Step, step_on_level};
use shared::poi::Poi;
use shared::protocol::SeenStairs;
use shared::site::{Exit, Site, SiteLevel};

use crate::storage::{CellMasks, SiteId, VisitSave};

// Split into files (`cargo xtask lint` keeps files small).
mod cells;
mod dev_area;
mod enter;
mod keepers;
mod leave;
mod levels;
mod moves;
mod place;
mod remembered;
mod sight;
mod trade;
mod wild;

use cells::{add_masks, all_cells, beside, level_chunks, nearby_cells, send_cells};
pub(super) use dev_area::generate_area;
use enter::enter_site;
pub(super) use enter::{dev_enter, enter_poi, generate, resume};
use keepers::{NewKeepers, keepers_of};
pub(super) use keepers::{give_to_keeper, keeper_building, keeper_stock, take_from_keeper};
use leave::leave_for_poi;
pub(super) use leave::{forget_site, killed, leave, put_out, use_exit};
use levels::{NewVisit, arrive_again, enter_level};
pub(super) use moves::{set_cell, stepped, teleport};
use place::{Mirror, Place};
pub(super) use sight::{forget_level, reveal_level, truesight_changed};
use sight::{memory, update_sight};
pub(super) use trade::open_trade;
pub(super) use wild::{area_level, explore, follow, leave_wilderness};

/// How far in a town others can be made out, in cells.
const TOWN_SIGHT: u32 = 40;

/// A player's stay on a site level, or in the wilderness.
pub(crate) struct Visit {
    place: Place,
    id: SiteId,
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
    /// Its cells that are another map's too.
    mirrors: Vec<Mirror>,
    /// In the wilderness, which live area's mobs are about the player
    /// (`entities::WildAreas`); 0 until one is given.
    wild_area: u32,
}

impl Visit {
    /// Where the player is on the level.
    pub(crate) fn position(&self) -> (i64, i64) {
        self.position
    }

    /// Puts the player somewhere else, for tests.
    #[cfg(test)]
    pub(crate) fn move_for_test(&mut self, at: (i64, i64)) {
        self.position = at;
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

    /// Whose mobs are about the player: the level's, or in the wilderness their
    /// live area's.
    pub(crate) fn population(&self) -> (SiteId, u32) {
        if self.is_wilderness() {
            (self.id, self.wild_area)
        } else {
            (self.id, self.index)
        }
    }

    /// The wilderness's live area the player is in.
    pub(crate) fn wild_area(&self) -> u32 {
        self.wild_area
    }

    pub(crate) fn set_wild_area(&mut self, area: u32) {
        self.wild_area = area;
    }

    /// Which site it is, for the debug API.
    #[cfg(feature = "debug-api")]
    pub(crate) fn site_id(&self) -> SiteId {
        self.id
    }

    /// The level's cells as the server has them (for reach, and the debug API).
    pub(crate) fn cells(&self) -> &shared::dungeon::Level {
        &self.level.level
    }

    /// Whether something at this cell can be seen from where the player is.
    pub(crate) fn can_see(&mut self, x: i64, y: i64) -> bool {
        if self.view_from != Some(self.position) {
            let (from_x, from_y) = self.position;
            let level = &self.level.level;
            self.view = Sight::new(level);
            self.view.add_view(level, from_x, from_y, self.reach());
            self.view_from = Some(self.position);
        }
        self.view.is_seen(x, y)
    }

    /// Which site it is.
    pub(crate) fn id(&self) -> SiteId {
        self.id
    }

    /// The poi it is behind, if a site's.
    pub(crate) fn poi(&self) -> Option<&Poi> {
        match &self.place {
            Place::Site { poi, .. } => Some(poi),
            Place::Wilderness { .. } => None,
        }
    }

    /// Its site, if a site's.
    pub(crate) fn site(&self) -> Option<&Site> {
        match &self.place {
            Place::Site { site, .. } => Some(site),
            Place::Wilderness { .. } => None,
        }
    }

    /// Whether it is in the wilderness.
    pub(crate) fn is_wilderness(&self) -> bool {
        matches!(self.place, Place::Wilderness { .. })
    }

    /// Where a cell's column is kept (remembered, changed): where it is round
    /// the planet in the wilderness (see `wild`), else itself.
    pub(crate) fn kept_at(&self, x: i64) -> i64 {
        self.wraps_at().map_or(x, |width| x.rem_euclid(width))
    }

    /// Whether a cell is another map's too, or a poi's cutout in the
    /// wilderness (see `sight::learnt_elsewhere`).
    fn shares(&self, x: i64, y: i64) -> bool {
        let inside = |(min_x, min_y, width, height): (i64, i64, i64, i64)| {
            (min_x..min_x + width).contains(&x) && (min_y..min_y + height).contains(&y)
        };
        self.mirrors.iter().any(|mirror| inside(mirror.bounds))
            || match &self.place {
                Place::Wilderness { cutouts, .. } => cutouts.iter().any(|&(_, b)| inside(b)),
                Place::Site { .. } => false,
            }
    }

    /// Where its cells wrap round the planet: the wilderness's width.
    pub(crate) fn wraps_at(&self) -> Option<i64> {
        match self.place {
            Place::Wilderness { width, .. } => Some(width),
            Place::Site { .. } => None,
        }
    }

    /// Whether they came into the site from the wilderness.
    pub(crate) fn came_from_wilderness(&self) -> bool {
        matches!(
            self.place,
            Place::Site {
                from_wilderness: true,
                ..
            }
        )
    }

    /// Whether the site is a town, whose layout players always know.
    fn is_town(&self) -> bool {
        self.site().is_some_and(|site| site.kind().is_town())
    }

    /// How far can be seen, in cells: open towns and the wilderness have a
    /// limit, or sight would be worked out across all of them.
    fn reach(&self) -> Option<u32> {
        match &self.place {
            Place::Wilderness { .. } => Some(crate::wilderness::SIGHT),
            Place::Site { .. } => self.is_town().then_some(TOWN_SIGHT),
        }
    }

    /// What is saved with the character.
    pub(crate) fn save(&self) -> VisitSave {
        VisitSave {
            site: self.id,
            level: self.index,
            position: self.position,
            from_wilderness: self.came_from_wilderness(),
        }
    }
}

/// Where on a level a player arrives.
enum Arrival {
    /// Coming from the world map or another level.
    From(Exit),
    /// Back where they were (a saved visit), if it is still walkable.
    At((i64, i64)),
    /// Where they last walked out of the site, if it is still walkable and
    /// their map of the site is still the one they had: otherwise at its
    /// entrance, on its first level.
    LeftAt((i64, i64)),
}

#[cfg(test)]
mod dev_area_tests;
#[cfg(test)]
mod left_at_tests;
#[cfg(test)]
mod tests;
