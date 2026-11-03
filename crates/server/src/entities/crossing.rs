//! Mobs following players up and down stairs.

use super::{Crossing, Doing, Hop, Populations, SETTLING_STEPS, Someone, nearest_free};

/// Takes a mob from one level to another. It stays where it is if there is
/// nowhere to go: it can only be somewhere nobody has been since, going forward,
/// as a level with its mobs away is kept.
pub(super) fn cross(
    populations: &mut Populations,
    crossing: Crossing,
    tick: u64,
    players: &[Someone],
) {
    let Crossing {
        mob,
        site,
        from_level,
        to_level,
        from_stairs,
        arrives,
        going_back,
    } = crossing;
    let settle = Doing::Settling {
        steps_left: SETTLING_STEPS,
    };
    let Some(from) = populations.0.get_mut(&(site, from_level)) else {
        return;
    };
    let Some(index) = from.mobs.iter().position(|one| one.id == mob) else {
        return;
    };
    if !populations.0.contains_key(&(site, to_level)) {
        let from = populations
            .0
            .get_mut(&(site, from_level))
            .expect("just used");
        from.mobs[index].doing = settle;
        from.due.insert((tick + 1, mob));
        return;
    }
    // Someone may be standing there: it arrives beside them, or if it can't, waits
    // at the stairs and tries again.
    let there = populations.0.get(&(site, to_level)).expect("just checked");
    let occupied = |spot: (i64, i64)| {
        there.mobs.iter().any(|one| one.position == spot)
            || players
                .iter()
                .any(|one| one.level == Some((site, to_level)) && one.position == spot)
    };
    let landing = if occupied(arrives) {
        nearest_free(&there.level, arrives, occupied)
    } else {
        Some(arrives)
    };
    let Some(landing) = landing else {
        let from = populations
            .0
            .get_mut(&(site, from_level))
            .expect("just used");
        from.due.insert((tick + 1, mob));
        return;
    };
    let from = populations
        .0
        .get_mut(&(site, from_level))
        .expect("just used");
    let mut moving = from.mobs.remove(index);
    from.due.retain(|&(_, id)| id != mob);
    let home = moving.home_level;
    if going_back {
        moving.way_back.pop();
    } else {
        moving.way_back.push(Hop {
            stairs: arrives,
            to_level: from_level,
            arrives: from_stairs,
        });
        if let Doing::Following { after, gap, .. } = moving.doing {
            moving.doing = Doing::Going {
                after,
                last_seen: arrives,
                gap,
            };
        }
    }
    moving.position = landing;
    let back_home = to_level == home && moving.way_back.is_empty();
    let left_home = from_level == home && !going_back && moving.way_back.len() == 1;
    let ready = moving.energy.ready_at(tick + 1, moving.speed);
    let there = populations
        .0
        .get_mut(&(site, to_level))
        .expect("checked above");
    there.due.insert((ready, moving.id));
    there.mobs.push(moving);
    if back_home {
        there.away = there.away.saturating_sub(1);
    }
    if left_home && let Some(home_level) = populations.0.get_mut(&(site, home)) {
        home_level.away += 1;
    }
}
