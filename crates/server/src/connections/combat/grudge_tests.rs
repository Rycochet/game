//! Mobs remember being hit (owner, 2026-11-07): a friendly one cools to
//! neutral, a neutral one turns unfriendly, and a second blow always does.

use shared::ability::Ability;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::mob::FIRST_HARM_OPINION;
use shared::protocol::{ClientMessage, KindChange, ServerMessage};
use shared::sheet::Feeling;

use super::tests::{beside, mob_health, run};
use crate::connections::tests::TestClient;
use crate::entities::Doing;
use crate::storage::SiteId;

/// Attacks the mob once and lets the blow land, without mobs acting; what the
/// player is told of its standing afterwards, if anything.
fn hit(test: &mut TestClient, mob: Id) -> Option<Feeling> {
    test.send(&[ClientMessage::UseAbility {
        ability: Ability::Attack,
        target: Some(mob),
    }]);
    run(test, 30, false);
    let answers = test.send(&[]);
    answers.iter().find_map(|answer| match answer {
        ServerMessage::EntitySeen(look) if look.id == mob => Some(look.feeling),
        _ => None,
    })
}

fn doing(test: &TestClient) -> Doing {
    test.world.populations.0.values().next().unwrap().mobs[0].doing
}

/// What is saved of the mob's opinion of the player.
fn saved_opinion(test: &mut TestClient, mob: Id) -> i64 {
    let site = SiteId::Dev {
        kind: shared::poi::PoiKind::Castle,
        seed: 7,
        ruin: 0,
    };
    let character = test.client.character_id.unwrap();
    let record = &test.world.site(site).levels[&0].records[&mob];
    record.memory.opinion(&character)
}

#[test]
fn a_neutral_mob_hit_once_turns_on_the_player_and_saves_it() {
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    let full = mob_health(&test, mob).unwrap();
    assert_eq!(hit(&mut test, mob), Some(Feeling::Unfriendly));
    let harm = full.milli() - mob_health(&test, mob).unwrap().milli();
    let first = i64::from(harm.max(FIRST_HARM_OPINION));
    assert_eq!(saved_opinion(&mut test, mob), -first);
    assert!(matches!(doing(&test), Doing::Going { after, .. } if after == test.client.entity_id()));
}

#[test]
fn a_friendly_mob_forgives_one_blow_but_not_two() {
    let (mut test, mob) = beside(CreatureKind::Dwarf);
    test.send(&[ClientMessage::DevSetCreature {
        kind: CreatureKind::Dwarf,
        how: KindChange::Permanent,
    }]);
    test.send(&[]);
    assert_eq!(hit(&mut test, mob), Some(Feeling::Neutral));
    // Cooled, not angry: it doesn't fight back.
    assert!(!matches!(doing(&test), Doing::Going { .. }));
    assert!(saved_opinion(&mut test, mob) <= -i64::from(FIRST_HARM_OPINION));
    assert_eq!(hit(&mut test, mob), Some(Feeling::Unfriendly));
    assert!(matches!(doing(&test), Doing::Going { .. }));
}

#[test]
fn a_timid_mob_hit_turns_unfriendly_and_keeps_away() {
    let (mut test, mob) = beside(CreatureKind::Chicken);
    assert_eq!(hit(&mut test, mob), Some(Feeling::Unfriendly));
    assert!(matches!(doing(&test), Doing::Fleeing { .. }));
}
