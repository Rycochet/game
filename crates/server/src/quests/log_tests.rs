use shared::protocol::{Marker, QuestAsk};

use super::accept::accept;
use super::ask::answer_quest;
use super::give_up::end;
use super::offers::town_markers;
use super::planets::planets;
use super::progress::complete_step;
use super::tests::{NOW, a_town};
use crate::connections::tests::TestClient;

#[test]
fn a_giver_shows_a_grey_question_mark_while_their_quest_is_under_way() {
    let mut test = TestClient::new();
    let (town, quest) = a_town(&mut test);
    let marker = |test: &mut TestClient| {
        let markers = town_markers(&test.client, &mut test.world, town, NOW);
        (markers.into_iter())
            .find(|&(building, _)| building == quest.building)
            .map(|(_, marker)| marker)
    };
    assert_eq!(marker(&mut test), Some(Marker::Available(quest.kind)));
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    // Taken outranks anything else they have on offer.
    assert_eq!(marker(&mut test), Some(Marker::Taken));
    for step in 0..quest.steps.len() {
        assert!(complete_step(
            &mut test.client,
            &mut test.world,
            quest.id,
            step
        ));
    }
    assert_eq!(marker(&mut test), Some(Marker::Ready(quest.kind)));
}

#[test]
fn only_a_quest_taken_can_be_tracked() {
    let mut test = TestClient::new();
    let (_, quest) = a_town(&mut test);
    let ask = |test: &mut TestClient, ask| {
        assert!(answer_quest(&mut test.client, &mut test.world, ask));
        test.client.character.quests.tracked
    };
    assert_eq!(ask(&mut test, QuestAsk::Track(Some(quest.id))), None);
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    assert_eq!(
        ask(&mut test, QuestAsk::Track(Some(quest.id))),
        Some(quest.id)
    );
    assert_eq!(ask(&mut test, QuestAsk::Track(None)), None);
}

#[test]
fn a_planet_quested_on_stays_listed_with_nothing_taken() {
    let mut test = TestClient::new();
    test.client
        .world
        .get_or_insert(shared::world::WorldSeed::new(1).id());
    let (_, quest) = a_town(&mut test);
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    assert!(end(&mut test.client, &mut test.world, quest.id, false, NOW));
    let told = planets(&test.client);
    assert_eq!(told.len(), 1);
    assert!(told[0].here);
    assert!(!told[0].taken, "given up");
}
