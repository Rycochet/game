use shared::protocol::ServerMessage;
use shared::quest::{DungeonWay, HIDDEN_KIND};

use super::accept::accept;
use super::dungeon::site_of;
use super::dungeon_tests::a_dungeon;
use super::tests::NOW;
use crate::connections::tests::TestClient;
use crate::storage::SiteId;

/// The poi the client is told is in `chunk`, after asking for it again.
fn told_of(test: &mut TestClient, chunk: shared::poi::ChunkPos) -> Option<shared::poi::PoiKind> {
    test.client.world_chunks.insert(chunk);
    assert!(crate::world_map::send_chunk(
        &mut test.client,
        &test.world,
        chunk
    ));
    let told = test
        .send(&[])
        .into_iter()
        .rev()
        .find_map(|message| match message {
            ServerMessage::WorldChunk(seen) if seen.chunk == chunk => Some(seen.poi),
            _ => None,
        });
    told.expect("the chunk").map(|poi| poi.kind)
}

#[test]
fn a_hidden_entrance_is_on_the_questers_map_alone_and_goes_in_from_there() {
    let mut test = TestClient::new();
    let (_, quest) = a_dungeon(&mut test, DungeonWay::Hidden);
    let poi = quest.dungeon.unwrap().poi;
    // Nobody sees it before taking the quest.
    assert_eq!(told_of(&mut test, poi.chunk), None);
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    // Taking it doesn't go in, but puts the entrance on their map.
    assert!(test.client.visit.is_none());
    assert_eq!(told_of(&mut test, poi.chunk), Some(HIDDEN_KIND));
    // Too far away: nothing.
    let site = site_of(&test.client, quest.id).unwrap();
    let in_site =
        |test: &TestClient| test.client.visit.as_ref().map(crate::sites::Visit::id) == Some(site);
    test.client.character.world_position = (poi.x + 40, poi.y);
    assert!(crate::sites::enter_poi(
        &mut test.client,
        &mut test.world,
        poi.chunk
    ));
    assert!(!in_site(&test));
    // At it: in, by clicking it or by Down.
    test.client.character.world_position = (poi.x, poi.y);
    assert!(crate::sites::enter_poi(
        &mut test.client,
        &mut test.world,
        poi.chunk
    ));
    assert!(in_site(&test));
    assert!(crate::sites::leave_dungeon(
        &mut test.client,
        &mut test.world
    ));
    assert!(crate::sites::explore(&mut test.client, &mut test.world));
    assert!(in_site(&test));
    assert!(test.world.populations.0.contains_key(&(site, 0)));
    // Out by its way up: on the world map at the entrance, remembering it.
    assert!(crate::sites::leave_dungeon(
        &mut test.client,
        &mut test.world
    ));
    assert!(test.client.visit.is_none());
    assert_eq!(test.client.character.world_position, (poi.x, poi.y));
    assert!(test.client.character.site_versions.contains_key(&site));
    // A reset keeps the entrance; ending the quest takes it away.
    let left = test.client.character.quests.dungeons[&quest.id]
        .left
        .unwrap();
    let reset = super::dungeon_reset::reset_due(
        &mut test.client,
        &mut test.world,
        left + super::dungeon::RESET_AFTER,
    );
    assert!(reset);
    assert!(!test.world.sites.contains_key(&site));
    assert_eq!(told_of(&mut test, poi.chunk), Some(HIDDEN_KIND));
    assert!(super::give_up::end(
        &mut test.client,
        &mut test.world,
        quest.id,
        false,
        NOW
    ));
    assert_eq!(told_of(&mut test, poi.chunk), None);
    assert!(matches!(site, SiteId::Quest { .. }));
}
