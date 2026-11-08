//! Finding a way across one level: the quickest route of legal steps from one
//! point to another (A*), for the player, monsters and spells. See
//! `docs/context/shared/pathfinding.md`.
//!
//! Every move comes from `movement::step_on_level` and is priced by
//! `movement::step_cost`, so a path is one the server would accept and its
//! cost is the time it takes to walk.

// Split into files (`cargo xtask lint` keeps files small).
mod distance_map;
mod find_path;
mod measure;
mod path;
mod straighten;
mod world_path;

pub use distance_map::DistanceMap;
pub use find_path::{find_path, find_path_by};
use measure::{distance_squared, estimate};
pub use path::Path;
pub use straighten::{straighten, straighten_by};
pub use world_path::{WORLD_ROUTE_REACH, find_world_path};

/// The order steps are tried in, so equally quick routes always come out the
/// same.
const DIRECTIONS: [(i64, i64); 8] = [
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, 0),
    (1, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
];

#[cfg(test)]
mod road_tests;
#[cfg(test)]
mod tests;
