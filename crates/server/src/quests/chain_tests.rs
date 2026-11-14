//! Chains between a town's givers ("I heard you helped X"), and quests that
//! wait for a moon: the hints a giver gives, and what they say once met.

use shared::poi::{CHUNK_SIZE, ChunkPos};
use shared::protocol::{Heard, Hint, OfferState};
use shared::quest::{Need, Quest, QuestId};

use super::needs::{Verdict, judge};
use super::tests::NOW;
use super::{cache, offers};
use crate::connections::tests::TestClient;

/// Every town's quests within `chunks` of the start, nearest chunk first.
fn quests_near(test: &mut TestClient, chunks: i64) -> Vec<Quest> {
    let (x, y) = test.client.world_position();
    let reach = chunks * CHUNK_SIZE;
    let mut towns: Vec<ChunkPos> = (test.world.pois)
        .in_area((x - reach, y - reach), (x + reach, y + reach))
        .into_iter()
        .filter(|poi| poi.kind.is_town())
        .map(|poi| poi.chunk)
        .collect();
    towns.sort_unstable();
    (towns.into_iter())
        .filter_map(|chunk| cache::town(&mut test.world, chunk))
        .flat_map(|town| town.quests.iter().cloned().collect::<Vec<_>>())
        .collect()
}

/// The other giver's quest `quest` needs done first, in its own town.
fn neighbour_need(quest: &Quest) -> Option<QuestId> {
    (quest.requirements.iter()).find_map(|requirement| match requirement.need {
        Need::Done(id)
            if id.town == quest.id.town
                && quest.id.earlier() != Some(id)
                && quest.requirements.len() == 1 =>
        {
            Some(id)
        }
        _ => None,
    })
}

#[test]
fn a_giver_names_the_neighbour_to_help_then_has_heard_of_it() {
    let mut test = TestClient::new();
    let quests = quests_near(&mut test, 24);
    let (quest, needed) = (quests.iter())
        .find_map(|quest| Some((quest.clone(), neighbour_need(quest)?)))
        .expect("a giver needing a neighbour's quest near the start");
    let Verdict::Missing(hints) = judge(&quest, &test.client, &test.world) else {
        panic!("not yet");
    };
    assert!(matches!(&hints[..], [Hint::Neighbour { .. }]), "{hints:?}");

    test.client.character.quests.done.insert(needed, NOW);
    assert_eq!(judge(&quest, &test.client, &test.world), Verdict::Met);
    let offered = offers::giver_offers(
        &test.client,
        &mut test.world,
        quest.id.town,
        quest.building,
        NOW,
    );
    let offer = offered.iter().find(|offer| offer.quest.id == quest.id);
    assert!(
        matches!(
            offer.map(|offer| &offer.state),
            Some(OfferState::Available {
                heard: Some(Heard::Neighbour { .. }),
                ..
            })
        ),
        "{:?}",
        offer.map(|offer| &offer.state)
    );
}

#[test]
fn a_moon_is_waited_for_until_it_is_up() {
    let mut test = TestClient::new();
    assert!(test.world.moon_count() > 0, "the test world has moons");
    let quests = quests_near(&mut test, 40);
    let quest = (quests.into_iter())
        .find(|quest| {
            (quest.requirements.iter()).all(|r| matches!(r.need, Need::Moon { .. }))
                && !quest.requirements.is_empty()
        })
        .expect("a quest waiting only for a moon near the start");
    let Need::Moon { moon } = quest.requirements[0].need else {
        unreachable!()
    };
    let (mut up, mut down) = (false, false);
    for hour in 0..96 {
        test.world.set_time(Some(f64::from(hour) * 60.0), None);
        match judge(&quest, &test.client, &test.world) {
            Verdict::Met => {
                assert!(test.world.moon_up(moon));
                up = true;
            }
            Verdict::Missing(hints) => {
                assert!(matches!(&hints[..], [Hint::Moon { .. }]), "{hints:?}");
                down = true;
            }
            Verdict::Hidden => panic!("a moon hides nothing"),
        }
    }
    assert!(up && down, "up {up}, down {down}");
}
