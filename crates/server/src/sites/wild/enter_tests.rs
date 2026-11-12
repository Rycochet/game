//! Going into a poi from the world map arrives in the wilderness: near
//! enough by the tokens' overlap, a town sent whole, and old saves on a first
//! level set in carried over.

use shared::poi::{ENTER_DISTANCE, Poi};
use shared::protocol::{ClientMessage, ServerMessage};

use super::tests::{chunk_near_start, position};
use crate::connections::tests::TestClient;
use crate::sites::cells_in;
use crate::storage::{SiteId, VisitSave};

/// A town near the start, with the player at it on the world map.
fn at_a_town(test: &mut TestClient) -> Poi {
    let plan = chunk_near_start(test, |c| c.poi().kind.is_town());
    let poi = *plan.cutout().unwrap().poi();
    test.client.character.world_position = (poi.x, poi.y);
    poi
}

fn entrance(test: &TestClient, poi: &Poi) -> (i64, i64) {
    let chunk = test.world.wilderness.chunk(poi.chunk).expect("worked out");
    chunk.entrance.expect("a town has an entrance")
}

#[test]
fn down_goes_into_a_poi_whose_token_overlaps_the_players() {
    let mut test = TestClient::new();
    let poi = at_a_town(&mut test);
    // Not straight at it, but near enough: 7² + 8² = 113 ≤ 11².
    test.client.character.world_position = (poi.x + 7, poi.y + 8);
    test.send(&[ClientMessage::Down]);
    assert_eq!(position(&test), entrance(&test, &poi));
    test.send(&[ClientMessage::Up]);

    // One point further, and Down explores where they stand.
    let far = (poi.x + ENTER_DISTANCE + 1, poi.y);
    test.client.character.world_position = far;
    test.send(&[ClientMessage::Down]);
    if let Some(visit) = &test.client.visit {
        assert_ne!(visit.position(), entrance(&test, &poi));
    }
}

#[test]
fn clicking_a_poi_goes_into_the_wilderness_at_its_entrance() {
    let mut test = TestClient::new();
    let poi = at_a_town(&mut test);
    test.send(&[ClientMessage::EnterSite { chunk: poi.chunk }]);
    let visit = test.client.visit.as_ref().expect("gone in");
    assert!(visit.is_wilderness());
    assert_eq!(visit.position(), entrance(&test, &poi));
    assert!(test.client.character.pois[&poi.chunk].visited);
}

#[test]
fn a_town_stepped_into_is_sent_whole() {
    let mut test = TestClient::new();
    let poi = at_a_town(&mut test);
    let answers = test.send(&[ClientMessage::EnterSite { chunk: poi.chunk }]);
    let visit = test.client.visit.as_ref().unwrap();
    let town = visit_cutouts(&test)
        .iter()
        .find(|set_in| set_in.chunk == poi.chunk)
        .expect("the town set in")
        .bounds;
    // The window takes in all of it.
    let (wx, wy, wc, wr) = visit.level.level.bounds();
    let (tx, ty, tc, tr) = town;
    assert!(wx <= tx && wy <= ty && tx + tc <= wx + wc && ty + tr <= wy + wr);
    let mut sent = crate::storage::CellMasks::new();
    for answer in &answers {
        if let ServerMessage::LevelCells(chunk) = answer
            && chunk.remembered
        {
            *sent.entry(chunk.chunk).or_default() |= chunk.mask;
        }
    }
    for (chunk, mask) in cells_in(town) {
        let got = sent.get(&chunk).copied().unwrap_or(0);
        assert_eq!(got & mask, mask, "all of {chunk:?}");
    }
}

fn visit_cutouts(test: &TestClient) -> Vec<crate::sites::SetIn> {
    match &test.client.visit.as_ref().unwrap().place {
        crate::sites::Place::Wilderness { cutouts, .. } => cutouts.clone(),
        crate::sites::Place::Site { .. } => Vec::new(),
    }
}

#[test]
fn a_saved_visit_to_a_first_level_set_in_comes_back_in_the_wilderness() {
    let mut test = TestClient::new();
    let poi = at_a_town(&mut test);
    let plan = chunk_near_start(&test, |c| c.poi().chunk == poi.chunk);
    let (ox, oy) = plan.cutout().unwrap().offset().expect("set in whole");
    super::ensure_chunk(&mut test.world, poi.chunk);
    let (ex, ey) = entrance(&test, &poi);
    let saved = VisitSave {
        site: SiteId::Poi(poi.chunk),
        level: 0,
        position: shared::spot::Spot::middle_of((ex - ox, ey - oy)),
        from_wilderness: false,
    };
    crate::sites::resume(&mut test.client, &mut test.world, saved);
    let visit = test.client.visit.as_ref().expect("back in");
    assert!(visit.is_wilderness());
    assert_eq!(visit.position(), (ex, ey));
}

#[test]
fn the_border_round_a_known_poi_is_sent_and_leads_out() {
    let mut test = TestClient::new();
    let poi = at_a_town(&mut test);
    let answers = test.send(&[ClientMessage::EnterSite { chunk: poi.chunk }]);
    let border = answers.iter().find_map(|answer| match answer {
        ServerMessage::Borders(cells) => Some(cells.clone()),
        _ => None,
    });
    let border = border.expect("sent with the window");
    let plan = chunk_near_start(&test, |c| c.poi().chunk == poi.chunk);
    let mut expected = plan.cutout().unwrap().border();
    let mut got = border.clone();
    expected.sort_unstable();
    got.sort_unstable();
    assert_eq!(got, expected, "the town is away from the seam");
    // Walking onto it does nothing; Up there leaves for the world map.
    let level = &test.client.visit.as_ref().unwrap().level.level;
    let &(x, y) = border
        .iter()
        .find(|&&(x, y)| level.is_walkable(x, y) && level.cell(x, y).is_outside())
        .expect("open ground round the town");
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    assert_eq!(position(&test), (x, y));
    let answers = test.send(&[ClientMessage::Up]);
    assert!(matches!(
        answers[..],
        [ServerMessage::OnWorldMap { .. }, ..]
    ));
}
