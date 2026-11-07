//! Telling onlookers who was harmed (`ServerMessage::Harmed`).

use shared::id::Id;
use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};

use crate::combat::Harm;
use crate::connections::tests::TestClient;

/// A mob the player sees, in a castle: they stand on it.
fn a_mob_in_sight(test: &mut TestClient) -> Id {
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: 7,
        level: 0,
        ruin: 0,
    }]);
    let mob = test.world.populations.0.values().next().unwrap().mobs[0].clone();
    test.send(&[ClientMessage::DevTeleport {
        x: mob.position.0,
        y: mob.position.1,
    }]);
    assert!(test.client.seen_entities.contains_key(&mob.id));
    mob.id
}

fn harmed(answers: &[ServerMessage]) -> Vec<(Id, u16)> {
    answers
        .iter()
        .filter_map(|answer| match answer {
            ServerMessage::Harmed { id, left } => Some((*id, *left)),
            _ => None,
        })
        .collect()
}

#[test]
fn onlookers_are_told_who_was_harmed_unless_they_struck_or_took_the_blow() {
    let mut test = TestClient::new();
    let mob = a_mob_in_sight(&mut test);
    let me = test.client.entity_id();
    let someone_else = Id::for_test(1);
    test.world.harmed = vec![
        // Struck by someone else: told.
        Harm {
            id: mob,
            left: 700,
            told: someone_else,
        },
        // Struck by this player, who knows from `AbilityLanded`.
        Harm {
            id: mob,
            left: 600,
            told: me,
        },
        // Out of sight.
        Harm {
            id: Id::for_test(2),
            left: 500,
            told: someone_else,
        },
    ];
    let answers = test.send(&[]);
    assert_eq!(harmed(&answers), vec![(mob, 700)]);
    // Told once only.
    assert!(test.world.harmed.is_empty());
    assert!(harmed(&test.send(&[])).is_empty());
}
