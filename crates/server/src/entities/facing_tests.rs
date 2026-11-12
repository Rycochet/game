//! Facing: sneaking up behind a mob, and mobs turning to blows and noises.

use shared::creature::CreatureKind;
use shared::facing::Heading;
use shared::protocol::ServerMessage;
use shared::spot::Spot;

use super::Doing;
use super::tests::{beside_a_mob, run, the_mob};

fn mob_mut(test: &mut crate::connections::tests::TestClient) -> &mut super::Mob {
    &mut test.world.populations.0.values_mut().next().unwrap().mobs[0]
}

#[test]
fn a_mob_facing_away_does_not_notice_someone_behind_it() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    mob_mut(&mut test).heading = Heading::EAST;
    run(&mut test, 40);
    assert_eq!(the_mob(&test).doing, Doing::Idle);

    // Turned round, it sees them.
    mob_mut(&mut test).heading = Heading::WEST;
    run(&mut test, 40);
    assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));
}

#[test]
fn prey_see_almost_all_round() {
    let mut test = beside_a_mob(CreatureKind::Chicken, 3);
    // Half turned away, a chicken still sees them; a kobold wouldn't.
    mob_mut(&mut test).heading = Heading::NORTH;
    let cone = the_mob(&test).cone();
    assert!(cone.sees(Spot::middle_of(super::tests::player(&test))));
}

#[test]
fn a_mob_at_rest_turns_to_fighting_it_hears() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    mob_mut(&mut test).heading = Heading::EAST;
    let at = Spot::middle_of(super::tests::player(&test));
    mob_mut(&mut test).listen(&[at]);
    assert_eq!(the_mob(&test).heading, Heading::WEST);
}

#[test]
fn a_mob_struck_turns_to_whoever_struck_it() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 1);
    mob_mut(&mut test).heading = Heading::EAST;
    let me = super::tests::player(&test);
    mob_mut(&mut test).face(Spot::middle_of(me));
    assert_eq!(the_mob(&test).heading, Heading::WEST);
}

#[test]
fn the_player_is_told_only_what_is_in_their_cone() {
    let mut test = beside_a_mob(CreatureKind::Dwarf, 3);
    let mob = the_mob(&test).id;
    test.client.heading = Heading::WEST;
    test.send(&[]);
    assert!(!test.client.seen_entities.contains_key(&mob), "behind them");
    test.send(&[shared::protocol::ClientMessage::Face(Heading::EAST)]);
    assert!(test.client.seen_entities.contains_key(&mob), "turned to it");
    // Turning in place is told alone.
    mob_mut(&mut test).heading = Heading::NORTH;
    let turned = test.send(&[]).into_iter().any(|answer| {
        matches!(answer, ServerMessage::Turned(turns) if turns == vec![(mob, Heading::NORTH)])
    });
    assert!(turned);
}
