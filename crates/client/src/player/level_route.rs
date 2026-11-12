//! Routes on a level or in the Hall for the player's journeys: at any angle,
//! for their size, round steep slopes and (for a good swimmer) across water
//! (`shared::pathfinding::find_sized_path`).

use shared::clearance::RoomMap;
use shared::creature::Size;
use shared::dungeon::Level;
use shared::movement::{Gait, LevelWay};
use shared::pathfinding::{SizedPath, find_sized_path};
use shared::relief::Relief;
use shared::spot::Spot;

/// The quickest route from `from` to the middle of cell `goal` through the
/// cells `known` accepts, on the level's ground (`relief`) for a player of
/// `size` with this `gait`: through closed doors as if open (they are opened
/// on the way), and through one known to be locked only if there is no other
/// way.
pub(super) fn level_route(
    level: &Level,
    (relief, gait): (Option<&Relief>, Gait),
    from: Spot,
    size: Size,
    goal: (i64, i64),
    known: impl Fn(i64, i64) -> bool + Copy,
) -> SizedPath {
    let open = level.with_all_doors_open();
    let rooms = RoomMap::new_by(&open, size, gait.swims_deep());
    let way = LevelWay {
        level: &open,
        relief,
        terrain: None,
        gait,
    };
    let to = Spot::middle_of(goal);
    let unlocked = |x, y| known(x, y) && !level.is_locked(x, y);
    let path = find_sized_path(&way, &rooms, from, to, None, unlocked);
    if path.reached {
        return path;
    }
    let tried = find_sized_path(&way, &rooms, from, to, None, known);
    if tried.reached { tried } else { path }
}
