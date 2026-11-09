//! Kicking doors: each kick harms one, and enough harm breaks it, open for
//! good, away from whoever broke it. A broken door's lock stays on it, unlocked.

use shared::door::{Side, Swing};
use shared::dungeon::Cell;

use super::act::set_here;
use super::keep_door;
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::update_sight;

/// The door beside the player that Kick would land on: a closed one (one they
/// can tell is a door), the nearest straight before diagonal, then by place.
pub(crate) fn door_to_kick(client: &Client) -> Option<(i64, i64)> {
    let visit = client.visit.as_ref()?;
    let (px, py) = visit.position;
    let known = client.character.doors.get(&visit.id);
    let known = known.and_then(|levels| levels.get(&visit.index));
    (visit.doors.iter())
        .filter(|&(&(x, y), door)| {
            let apart = (x - px).abs().max((y - py).abs());
            let secret = visit.level.level.cell(x, y) == Cell::SecretDoor;
            let found = client.truesight
                || known.is_some_and(|doors| doors.contains_key(&(visit.kept_at(x), y)));
            apart == 1 && door.door.swing.is_shut() && (found || !secret)
        })
        .map(|(&at, _)| at)
        .min_by_key(|&(x, y)| ((x - px).abs() + (y - py).abs(), y, x))
}

/// A kick lands on the door at `at` beside the player with `force` (thousandths
/// of a point): harmed, and broken once that reaches its toughness. How much of
/// it is left, per mille; `None` if there is no closed door there now.
pub(crate) fn kick_door(
    client: &mut Client,
    world: &mut ServerWorld,
    at: (i64, i64),
    force: u32,
) -> Option<u16> {
    let visit = client.visit.as_mut()?;
    let (px, py) = visit.position;
    let beside = (at.0 - px).abs().max((at.1 - py).abs()) == 1;
    let mut door = visit.doors.get(&at).copied().filter(|_| beside)?;
    if !door.door.swing.is_shut() {
        return None;
    }
    let toughness = door.door.toughness().max(1);
    door.door.damage = door.door.damage.saturating_add(force).min(toughness);
    if door.door.damage >= toughness {
        let runs = visit.level.level.opening_runs_east_west(at.0, at.1);
        door.door.swing = Swing::Broken(Side::of(runs, at, (px, py)).opposite());
        if let Some(lock) = &mut door.door.lock {
            lock.locked = false;
        }
    }
    set_here(visit, at, door);
    keep_door(world, door.key, door.door);
    let left = (toughness - door.door.damage) * 1000 / toughness;
    update_sight(client).then_some(left as u16)
}
