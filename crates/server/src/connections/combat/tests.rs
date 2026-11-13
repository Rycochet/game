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
pub(super) fn beside(kind: CreatureKind) -> (TestClient, Id) {
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
    let id = population.remake_for_test(0, kind, (x + 1, y)).id;
    (test, id)
}

/// Lets ticks go by with the player's ability landing and what they asked
/// for done, and mobs acting if `mobs`.
pub(super) fn run(test: &mut TestClient, ticks: u64, mobs: bool) {
    for _ in 0..ticks {
        test.world.action_clock.advance(1);
        land_ability(&mut test.client, &mut test.world);
        act(&mut test.client, &mut test.world);
        if mobs {
            crate::entities::run_mobs(std::slice::from_mut(&mut test.client), &mut test.world);
        }
    }
}

pub(super) fn mob_health(test: &TestClient, id: Id) -> Option<Points> {
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
        target: Some(target),
    }
}

pub(super) fn received(test: &mut TestClient) -> Vec<ServerMessage> {
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
fn an_attack_waiting_for_energy_can_be_withdrawn() {
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    test.time_passes = false;
    test.send(&[attack(mob)]);
    // Landed, but not yet able to act again: the next waits for the energy.
    run(&mut test, 5, false);
    test.send(&[attack(mob)]);
    assert_eq!(test.client.waiting.len(), 1);
    test.send(&[ClientMessage::WithdrawAbility]);
    assert!(test.client.waiting.is_empty());
    run(&mut test, 100, false);
    assert!(
        !received(&mut test)
            .iter()
            .any(|message| matches!(message, ServerMessage::AbilityStarted { who: None, .. }))
    );
}

#[test]
fn a_mob_out_of_reach_is_not_attacked() {
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    let one = &mut test.world.populations.0.values_mut().next().unwrap().mobs[0];
    one.put((one.position.0 + 3, one.position.1));
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
    // The killer shares in it once everyone is told who died.
    crate::entities::sync(std::slice::from_mut(&mut test.client), &mut test.world, &[]);
    let dwarves = test.client.sheet.foes.of(CreatureKind::Dwarf);
    assert_eq!(dwarves.defeated, 1);
    assert!(dwarves.fought > 0);
    assert!(test.world.kills.is_empty());
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
            .any(|message| matches!(message, ServerMessage::Killed { permanent: false }))
    );
}

#[test]
fn blows_practise_once_every_five_seconds() {
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    let melee = shared::skill::SkillId::find("combat/melee").unwrap();
    // Two blows inside one window: practice and a block fought, once.
    for _ in 0..2 {
        test.send(&[attack(mob)]);
        run(&mut test, 30, false);
    }
    assert_eq!(test.client.fight.attacks, 2, "both landed");
    // A quarter of a blow's practice reaches melee, two levels up.
    let per_blow = shared::combat::PRACTICE_PER_BLOW / 4;
    assert_eq!(test.client.sheet.skills.practice(melee), per_blow);
    assert_eq!(test.client.sheet.foes.of(CreatureKind::Dwarf).fought, 1);
    // Once the window has closed, the next blow counts again.
    run(&mut test, shared::skill::WINDOW_TICKS, false);
    test.send(&[attack(mob)]);
    run(&mut test, 30, false);
    assert_eq!(test.client.sheet.skills.practice(melee), 2 * per_blow);
    assert_eq!(test.client.sheet.foes.of(CreatureKind::Dwarf).fought, 2);
}
