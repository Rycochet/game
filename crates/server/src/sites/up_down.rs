//! The Down and Up keys: down or up the stairs the player stands on, and
//! otherwise exploring from the world map (Down) or leaving for it (Up).

use shared::site::StairsDirection;

use super::{explore, leave_by_key, use_exit};
use crate::ServerWorld;
use crate::connections::Client;

/// Which way the stairs the player stands on go, if they stand on any.
fn stairs_here(client: &Client) -> Option<StairsDirection> {
    let visit = client.visit.as_ref()?;
    let (x, y) = visit.position;
    let stairs = visit.level.stairs.iter().find(|s| (s.x, s.y) == (x, y))?;
    Some(stairs.direction)
}

/// The Down key: down the stairs underfoot, or into the wilderness from the
/// world map. Anything else is ignored.
pub(crate) fn go_down(client: &mut Client, world: &mut ServerWorld) -> bool {
    if client.visit.is_none() {
        return explore(client, world);
    }
    match stairs_here(client) {
        Some(StairsDirection::Down) => use_exit(client, world),
        _ => true,
    }
}

/// The Up key: up the stairs underfoot, or out to the world map where the
/// player stands (`leave_by_key`).
pub(crate) fn go_up(client: &mut Client, world: &mut ServerWorld) -> bool {
    match stairs_here(client) {
        Some(StairsDirection::Up) => use_exit(client, world),
        _ => leave_by_key(client, world),
    }
}
