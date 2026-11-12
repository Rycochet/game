use shared::combat::{Body, in_reach};
use shared::creature::{CreatureKind, Size};

use super::super::Taken;
use super::super::tests::{beside_a_mob, the_mob};
use super::close_in;

#[test]
fn a_small_mob_in_the_next_cell_closes_in_until_it_reaches() {
    let test = beside_a_mob(CreatureKind::Chicken, 1);
    let mut mob = the_mob(&test);
    let level = test
        .world
        .populations
        .0
        .values()
        .next()
        .unwrap()
        .level
        .clone();
    let player = test.client.visit.as_ref().unwrap().spot();
    let them = Body::on(None, player, Size::Medium);
    let taken: Taken = [player.cell(), mob.position].into_iter().collect();
    let body = |mob: &super::super::Mob| Body::on(None, mob.spot, mob.size());
    // A chicken's beak doesn't reach from the next cell's middle...
    assert!(!in_reach(&level, &body(&mob), 0, &them));
    // ...so it closes in, half a cell at most a turn, and then it does.
    for _ in 0..2 {
        let me = body(&mob);
        if in_reach(&level, &me, 0, &them) {
            break;
        }
        assert!(close_in(&mut mob, &level, (&me, &them), 0, &taken).is_some());
        assert!(me.at.distance(mob.spot) <= shared::spot::HALF_SPAN as u64);
    }
    assert!(in_reach(&level, &body(&mob), 0, &them));
    // Never into them, and no further once it reaches.
    assert!(mob.spot.distance(player) > 100);
    let me = body(&mob);
    assert_eq!(close_in(&mut mob, &level, (&me, &them), 0, &taken), None);
}

#[test]
fn nobody_closes_in_from_further_than_the_next_cell() {
    let test = beside_a_mob(CreatureKind::Chicken, 3);
    let mut mob = the_mob(&test);
    let level = test
        .world
        .populations
        .0
        .values()
        .next()
        .unwrap()
        .level
        .clone();
    let player = test.client.visit.as_ref().unwrap().spot();
    let them = Body::on(None, player, Size::Medium);
    let me = Body::on(None, mob.spot, mob.size());
    let spot = mob.spot;
    assert_eq!(
        close_in(&mut mob, &level, (&me, &them), 0, &Taken::new()),
        None
    );
    assert_eq!(mob.spot, spot);
}
