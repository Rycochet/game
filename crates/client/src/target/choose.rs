//! Which creature to target: the rules, apart from Bevy so tests can drive them.

use shared::id::Id;
use shared::sheet::Feeling;

/// A creature the player can see, as the rules need it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Seen {
    pub(super) id: Id,
    pub(super) at: (i64, i64),
    pub(super) feeling: Feeling,
}

impl Seen {
    fn is_enemy(&self) -> bool {
        self.feeling == Feeling::Unfriendly
    }
}

/// How far apart two cells are, squared, for putting creatures in order.
fn distance(a: (i64, i64), b: (i64, i64)) -> i64 {
    let (dx, dy) = (a.0 - b.0, a.1 - b.1);
    dx * dx + dy * dy
}

/// The ones Tab goes through, nearest first (ties by id, so the order is the
/// same every time): enemies only, unless none are in sight (owner).
pub(super) fn in_order(player: (i64, i64), seen: &[Seen]) -> Vec<Id> {
    let enemies = seen.iter().any(Seen::is_enemy);
    let mut chosen: Vec<&Seen> = seen
        .iter()
        .filter(|one| !enemies || one.is_enemy())
        .collect();
    chosen.sort_by_key(|one| (distance(player, one.at), one.id));
    chosen.into_iter().map(|one| one.id).collect()
}

/// The next one out (`outwards`) or in from `current`, round again at the end;
/// the nearest going out, or the furthest going in, if `current` isn't among
/// them.
pub(super) fn cycle(order: &[Id], current: Option<Id>, outwards: bool) -> Option<Id> {
    if order.is_empty() {
        return None;
    }
    let last = order.len() - 1;
    let at = current.and_then(|current| order.iter().position(|&id| id == current));
    let next = match (at, outwards) {
        (None, true) => 0,
        (None, false) => last,
        (Some(at), true) if at == last => 0,
        (Some(at), true) => at + 1,
        (Some(0), false) => last,
        (Some(at), false) => at - 1,
    };
    Some(order[next])
}

/// The nearest enemy in sight, other than `except` (one that has just died).
pub(super) fn nearest_enemy(player: (i64, i64), seen: &[Seen], except: Id) -> Option<Id> {
    seen.iter()
        .filter(|one| one.is_enemy() && one.id != except)
        .min_by_key(|one| (distance(player, one.at), one.id))
        .map(|one| one.id)
}
