use shared::ability::Ability;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, Landed, ServerMessage};
use shared::vitals::{Points, Vital};

use super::{cancel_ability, land_ability};
use crate::connections::messages::act;
use crate::connections::tests::TestClient;

/// A player in a castle with a mob of this kind right beside them, east.
fn beside(kind: CreatureKind) -> (TestClient, Id) {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: 7,
        level: 0,
        ruin: 0,
    }]);
    let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(1);
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    let population = test.world.populations.0.values_mut().next().unwrap();
    let mob = &mut population.mobs[0];
    mob.kind = kind;
    mob.position = (x + 1, y);
    mob.home = (x + 1, y);
    let id = mob.id;
    (test, id)
}

/// Lets ticks go by with the player's ability landing and what they asked
/// for done, and mobs acting if `mobs`.
fn run(test: &mut TestClient, ticks: u64, mobs: bool) {
    for _ in 0..ticks {
        test.world.action_clock.advance(1);
        land_ability(&mut test.client, &mut test.world);
        act(&mut test.client, &mut test.world);
        if mobs {
            crate::entities::run_mobs(std::slice::from_mut(&mut test.client), &mut test.world);
        }
    }
}

fn mob_health(test: &TestClient, id: Id) -> Option<Points> {
    let tick = test.world.action_clock.ticks();
    let mob = test
        .world
        .populations
        .0
        .values()
        .flat_map(|population| &population.mobs)
        .find(|mob| mob.id == id)?;
    Some(mob.health(tick))
}

fn attack(target: Id) -> ClientMessage {
    ClientMessage::UseAbility {
        ability: Ability::Attack,
        target,
    }
}

fn received(test: &mut TestClient) -> Vec<ServerMessage> {
    std::iter::from_fn(|| test.end.receive().unwrap()).collect()
}

#[test]
fn an_attack_winds_up_then_harms_the_mob_which_turns_on_the_player() {
    // A dwarf minds its own business until it is hit.
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    let full = mob_health(&test, mob).unwrap();
    let answers = test.send(&[attack(mob)]);
    assert!(answers.iter().any(|message| matches!(
        message,
        ServerMessage::AbilityStarted { who: None, lands_in, ready_in, .. }
            if lands_in < ready_in
    )));
    assert_eq!(mob_health(&test, mob), Some(full), "not before it lands");
    run(&mut test, 20, false);
    assert!(mob_health(&test, mob).unwrap() < full);
    assert!(received(&mut test).iter().any(|message| matches!(
        message,
        ServerMessage::AbilityLanded {
            landed: Landed::Hit { .. },
            ..
        }
    )));
    let attacked_by = test.client.entity_id();
    let one = &test.world.populations.0.values().next().unwrap().mobs[0];
    assert_eq!(one.target, Some(attacked_by));
    // Practice from the blow.
    assert!(!test.client.sheet.skills.is_empty());
}

#[test]
fn a_cancelled_attack_does_no_harm_and_soon_lets_the_player_act() {
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    let full = mob_health(&test, mob).unwrap();
    test.send(&[attack(mob)]);
    assert!(cancel_ability(&mut test.client, &test.world));
    run(&mut test, 20, false);
    assert_eq!(mob_health(&test, mob), Some(full));
    assert!(received(&mut test).iter().any(|message| matches!(
        message,
        ServerMessage::AbilityLanded {
            landed: Landed::Cancelled,
            ..
        }
    )));
}

#[test]
fn a_mob_out_of_reach_is_not_attacked() {
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    test.world.populations.0.values_mut().next().unwrap().mobs[0]
        .position
        .0 += 3;
    let answers = test.send(&[attack(mob)]);
    assert!(
        !answers
            .iter()
            .any(|message| matches!(message, ServerMessage::AbilityStarted { .. }))
    );
}

#[test]
fn blows_kill_a_mob_for_good() {
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    for _ in 0..200 {
        if mob_health(&test, mob).is_none() {
            break;
        }
        test.send(&[attack(mob)]);
        run(&mut test, 10, false);
    }
    assert_eq!(mob_health(&test, mob), None);
    assert!(test.world.died.contains(&mob));
}

#[test]
fn an_unfriendly_mob_attacks_and_a_player_killed_is_put_out_whole() {
    let (mut test, mob) = beside(CreatureKind::Kobold);
    run(&mut test, 40, true);
    let tick = test.world.action_clock.ticks();
    let limits = test.client.limits(tick);
    let health = test.client.sheet.vitals.at(Vital::Health, limits, tick);
    assert!(health < Points::whole(limits.health), "the kobold hit them");
    assert!(received(&mut test).iter().any(|message| matches!(
        message,
        ServerMessage::AbilityStarted { who: Some(who), target: None, .. } if *who == mob
    )));
    // Next to nothing left: the next blow kills.
    let tick = test.world.action_clock.ticks();
    let left = test.client.sheet.vitals.at(Vital::Health, limits, tick);
    let harm = Points(left.milli().saturating_sub(1));
    test.client.sheet.vitals.hurt(harm, limits, tick, 0);
    run(&mut test, 40, true);
    assert!(test.client.visit.is_none(), "on the world map");
    let tick = test.world.action_clock.ticks();
    let health = test.client.sheet.vitals.at(Vital::Health, limits, tick);
    assert_eq!(health, Points::whole(limits.health));
    assert!(
        received(&mut test)
            .iter()
            .any(|message| matches!(message, ServerMessage::Killed))
    );
}
