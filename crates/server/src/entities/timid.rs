//! Timid mobs (owner, 2026-11-07): they keep a distance from everyone, more
//! the less they like them, and only step away while someone is nearer than
//! that. Caught next to someone they are unfriendly to, they strike once and
//! then step away, so one backed into a corner fights.

use shared::creature::Temper;
use shared::sheet::Feeling;

use super::{Doing, Mob, Spot, cells_apart};

/// Friends are kept one empty cell away.
const FRIEND_APART: i64 = 2;
/// Strangers, five cells.
const STRANGER_APART: i64 = 5;
/// Enemies, ten cells.
const ENEMY_APART: i64 = 10;

pub(super) fn is_timid(mob: &Mob) -> bool {
    mob.kind.temper() == Temper::Timid
}

/// How many cells apart a timid mob keeps from someone it feels this way about.
pub(super) fn keeps_apart(feeling: Feeling) -> i64 {
    match feeling {
        Feeling::None => 0,
        Feeling::Friendly => FRIEND_APART,
        Feeling::Neutral => STRANGER_APART,
        Feeling::Unfriendly => ENEMY_APART,
    }
}

/// Whether someone is nearer than this mob, timid or not, wants them.
pub(super) fn too_near(mob: &Mob, spot: &Spot) -> bool {
    let feeling = mob.feeling(spot.kind, spot.character.as_ref());
    cells_apart(mob.position, spot.position) < keeps_apart(feeling)
}

/// Who a timid mob that is keeping away would strike at now, if anyone: the one
/// it is keeping away from, if it is unfriendly to them and hasn't just struck.
/// Whether they are in reach is for the caller.
pub(super) fn strikes_at<'a>(mob: &Mob, spots: &'a [Spot]) -> Option<&'a Spot> {
    let Doing::Fleeing {
        from,
        struck: false,
    } = mob.doing
    else {
        return None;
    };
    let spot = spots.iter().find(|spot| spot.id == from)?;
    let feeling = mob.feeling(spot.kind, spot.character.as_ref());
    (feeling == Feeling::Unfriendly).then_some(spot)
}
