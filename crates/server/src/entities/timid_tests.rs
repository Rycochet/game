//! Timid mobs keeping their distance, and striking once when caught.

use shared::creature::CreatureKind;
use shared::id::Id;
use shared::sheet::Feeling;
use shared::vitals::Points;

use super::tests::{beside_a_mob, player, run, the_mob};
use super::timid::{keeps_apart, too_near};
use super::{Doing, Spot, cells_apart};

#[test]
fn friends_are_kept_one_cell_off_strangers_five_enemies_ten() {
    let test = beside_a_mob(CreatureKind::Chicken, 4);
    let mob = the_mob(&test);
    let at = |apart, kind| Spot {
        id: Id::for_test(9),
        character: None,
        kind,
        position: (mob.position.0 - apart, mob.position.1),
        scent: 100,
    };
    // Another chicken is a friend.
    assert!(too_near(&mob, &at(1, CreatureKind::Chicken)));
    assert!(!too_near(&mob, &at(2, CreatureKind::Chicken)));
    assert!(too_near(&mob, &at(4, CreatureKind::Human)));
    assert!(!too_near(&mob, &at(5, CreatureKind::Human)));
    assert_eq!(keeps_apart(Feeling::Unfriendly), 10);
}

#[test]
fn a_timid_mob_goes_only_as_far_as_it_wants_and_no_further() {
    let mut test = beside_a_mob(CreatureKind::Chicken, 2);
    for _ in 0..200 {
        run(&mut test, 1);
        if matches!(the_mob(&test).doing, Doing::Settling { .. }) {
            break;
        }
    }
    let mob = the_mob(&test);
    assert!(
        matches!(mob.doing, Doing::Settling { .. }),
        "{:?}",
        mob.doing
    );
    assert_eq!(
        cells_apart(mob.position, player(&test)),
        keeps_apart(Feeling::Neutral)
    );
}

#[test]
fn a_timid_mob_caught_by_an_enemy_strikes_once_then_runs() {
    let mut test = beside_a_mob(CreatureKind::Chicken, 1);
    let tester = test.client.character_id.unwrap();
    let population = test.world.populations.0.values_mut().next().unwrap();
    let mob = &mut population.mobs[0];
    mob.memory
        .remember_harm(tester, Points::whole(1), Feeling::Neutral);
    let mut blows = 0;
    let mut was_striking = false;
    // Until it is far enough to stop keeping away.
    for _ in 0..300 {
        run(&mut test, 1);
        let striking = the_mob(&test).fight.pending.is_some();
        if striking && !was_striking {
            blows += 1;
        }
        was_striking = striking;
        if matches!(the_mob(&test).doing, Doing::Settling { .. }) {
            break;
        }
    }
    assert_eq!(blows, 1);
    assert!(
        cells_apart(the_mob(&test).position, player(&test)) >= keeps_apart(Feeling::Unfriendly)
    );
}

#[test]
fn a_cornered_timid_mob_strikes_again() {
    let mut test = beside_a_mob(CreatureKind::Chicken, 1);
    let tester = test.client.character_id.unwrap();
    let here = player(&test);
    let population = test.world.populations.0.values_mut().next().unwrap();
    let mob = &mut population.mobs[0];
    mob.memory
        .remember_harm(tester, Points::whole(1), Feeling::Neutral);
    // Nowhere to go: walls all round, but where the player stands.
    let at = mob.position;
    population.level = walled_in(&population.level, at, here);
    let mut blows = 0;
    let mut was_striking = false;
    for _ in 0..120 {
        run(&mut test, 1);
        let striking = the_mob(&test).fight.pending.is_some();
        if striking && !was_striking {
            blows += 1;
        }
        was_striking = striking;
    }
    assert!(blows > 1, "{blows}");
}

/// The level with walls all round `at` but towards `open`.
fn walled_in(
    level: &shared::dungeon::Level,
    at: (i64, i64),
    open: (i64, i64),
) -> shared::dungeon::Level {
    let mut level = level.clone();
    for dy in -1..=1 {
        for dx in -1..=1 {
            let cell = (at.0 + dx, at.1 + dy);
            if cell != at && cell != open {
                level.set(cell.0, cell.1, shared::dungeon::Cell::Wall);
            }
        }
    }
    level
}
