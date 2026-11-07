//! Territorial mobs: they keep strangers out of the ground round their home,
//! standing between them and it, and turn on anyone who comes in.

use shared::creature::Temper;
use shared::dungeon::{Level, line_is_clear};
use shared::sheet::Feeling;

use super::{Doing, Mob, Spot, cells_apart};

/// How far round its home a territorial mob counts as its own, in cells.
pub(super) const TERRITORY: i64 = 6;
/// How far from its home someone it is angry with may get before it gives up
/// chasing them and goes back to guarding, in cells. From home rather than from
/// the mob, as someone no faster than it could never get further from it.
pub(super) const LEASH: i64 = 8;

pub(super) fn is_territorial(mob: &Mob) -> bool {
    mob.kind.temper() == Temper::Territorial
}

/// Remembers anyone it can see inside its territory, unless it likes them, and
/// lets go of old trespasses. A trespass makes it angry for a while.
pub(super) fn mind_territory(mob: &mut Mob, level: &Level, spots: &[Spot], tick: u64) {
    mob.memory.forget_old(tick);
    if !is_territorial(mob) {
        return;
    }
    let sees_within = mob.kind.senses().sight_range();
    for spot in spots {
        let Some(character) = spot.character else {
            continue;
        };
        let inside = cells_apart(spot.position, mob.home) <= TERRITORY;
        let seen = cells_apart(spot.position, mob.position) <= sees_within
            && line_is_clear(level, mob.position, spot.position);
        if inside && seen && mob.feeling(spot.kind, Some(&character)) != Feeling::Friendly {
            mob.memory.remember_trespass(character, tick);
        }
    }
}

/// What a territorial mob does about someone it can see, in place of its kind's
/// usual reaction: it guards against strangers, and chases those it is angry with
/// only while they are near its home.
pub(super) fn territorial_doing(mob: &Mob, spot: &Spot, feeling: Feeling) -> Option<Doing> {
    let guard = Doing::Guarding {
        against: spot.id,
        last_seen: spot.position,
    };
    match feeling {
        Feeling::None | Feeling::Friendly => None,
        Feeling::Neutral => Some(guard),
        Feeling::Unfriendly if cells_apart(spot.position, mob.home) > LEASH => Some(guard),
        Feeling::Unfriendly => Some(Doing::Going {
            after: spot.id,
            last_seen: spot.position,
            gap: super::ATTACK_GAP,
        }),
    }
}

/// Where to stand against someone: between them and home, at the territory's edge,
/// or nearer home if they are nearer.
pub(super) fn guard_spot(home: (i64, i64), intruder: (i64, i64)) -> (i64, i64) {
    let (dx, dy) = (intruder.0 - home.0, intruder.1 - home.1);
    let apart = dx.abs().max(dy.abs());
    if apart == 0 {
        return home;
    }
    let out = TERRITORY.min(apart - 1).max(0);
    (home.0 + dx * out / apart, home.1 + dy * out / apart)
}
