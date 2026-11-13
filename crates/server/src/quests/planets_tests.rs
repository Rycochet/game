use super::accept::accept;
use super::planets::planets;
use super::progress::complete_step;
use super::tests::{NOW, a_town};
use crate::connections::tests::TestClient;

#[test]
fn the_hall_hears_of_planets_with_quests_and_those_to_hand_in() {
    let mut test = TestClient::new();
    // (The test client joins no world of its own.)
    test.client
        .world
        .get_or_insert(shared::world::WorldSeed::new(1).id());
    assert!(planets(&test.client).is_empty(), "none taken yet");
    let (_, quest) = a_town(&mut test);
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    let told = planets(&test.client);
    assert_eq!(told.len(), 1);
    assert_eq!(Some(told[0].world), test.client.world);
    assert!(!told[0].ready, "nothing done yet");

    for step in 0..quest.steps.len() {
        assert!(complete_step(
            &mut test.client,
            &mut test.world,
            quest.id,
            step
        ));
    }
    assert!(planets(&test.client)[0].ready, "every step done");
}
