use shared::ability::Ability;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::magic::HEAL_MANA;
use shared::protocol::{ClientMessage, DevVitals, Landed, ServerMessage};
use shared::vitals::{Points, Vital};

use super::tests::{beside, mob_health, received, run};
use crate::connections::tests::TestClient;

fn heal(target: Option<Id>) -> ClientMessage {
    ClientMessage::UseAbility {
        ability: Ability::Heal,
        target,
    }
}

fn vital(test: &TestClient, vital: Vital) -> Points {
    let tick = test.world.action_clock.ticks();
    let limits = test.client.limits(tick);
    test.client.sheet.vitals.at(vital, limits, tick)
}

fn started(answers: &[ServerMessage]) -> bool {
    answers.iter().any(|message| {
        matches!(
            message,
            ServerMessage::AbilityStarted {
                who: None,
                ability: Ability::Heal,
                ..
            }
        )
    })
}

#[test]
fn healing_yourself_turns_six_mana_into_twelve_health_at_first() {
    let (mut test, _) = beside(CreatureKind::Dwarf);
    assert!(started(&test.send(&[heal(None)])));
    // Hurt while it winds up, so Health doesn't refill before it lands.
    let tick = test.world.action_clock.ticks();
    let most = test.client.limits(tick).health;
    test.send(&[ClientMessage::DevVitals(DevVitals::Hurt(most - 2))]);
    let mana = vital(&test, Vital::Mana);
    run(&mut test, 30, false);
    assert_eq!(vital(&test, Vital::Health), Points::whole(14));
    assert!(vital(&test, Vital::Mana) < mana);
    assert!(received(&mut test).iter().any(|message| matches!(
        message,
        ServerMessage::AbilityLanded {
            target: None,
            landed: Landed::Healed { amount, .. },
            ..
        } if *amount == Points::whole(12)
    )));
    // Practice for Healing.
    let healing = shared::magic::heal_skill();
    assert!(test.client.sheet.skills.practice(healing) > 0);
}

#[test]
fn healing_at_full_health_still_spends_the_mana() {
    let (mut test, _) = beside(CreatureKind::Dwarf);
    let mana = vital(&test, Vital::Mana);
    let me = test.client.entity_id();
    assert!(started(&test.send(&[heal(Some(me))])));
    run(&mut test, 30, false);
    let spent = mana.milli() - vital(&test, Vital::Mana).milli();
    // Mana refills meanwhile, so a little less than all of it shows.
    assert!(spent > Points::whole(HEAL_MANA - 1).milli(), "{spent}");
}

#[test]
fn without_the_mana_it_isnt_begun() {
    let (mut test, _) = beside(CreatureKind::Dwarf);
    let tick = test.world.action_clock.ticks();
    let most = test.client.limits(tick).mana;
    test.send(&[ClientMessage::DevVitals(DevVitals::SpendMana(
        most - HEAL_MANA + 1,
    ))]);
    assert!(!started(&test.send(&[heal(None)])));
}

#[test]
fn a_mob_can_be_healed_even_one_fighting_the_healer() {
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    test.send(&[ClientMessage::UseAbility {
        ability: Ability::Attack,
        target: Some(mob),
    }]);
    run(&mut test, 30, false);
    let hurt = mob_health(&test, mob).unwrap();
    assert!(started(&test.send(&[heal(Some(mob))])));
    run(&mut test, 30, false);
    assert!(mob_health(&test, mob).unwrap() > hurt);
    assert!(received(&mut test).iter().any(|message| matches!(
        message,
        ServerMessage::AbilityLanded {
            target: Some(target),
            landed: Landed::Healed { .. },
            ..
        } if *target == mob
    )));
}
