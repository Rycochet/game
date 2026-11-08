//! The Leave key in a site: out from any outside cell of the level entered
//! from the world map, but from a cave only at its edge.

use shared::dungeon::Cell;
use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::site::leads_out;

use crate::connections::tests::TestClient;
use crate::storage::SiteId;

fn enter(kind: PoiKind, level: u32) -> TestClient {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevEnterSite {
        kind,
        seed: 4,
        level,
        ruin: 0,
    }]);
    test
}

/// The first cell of the level the player is on that `wanted` picks.
fn find(test: &TestClient, wanted: impl Fn(Cell, bool) -> bool) -> (i64, i64) {
    let level = &test.client.visit.as_ref().unwrap().level.level;
    let (min_x, min_y, width, height) = level.bounds();
    (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .find(|&(x, y)| wanted(level.cell(x, y), leads_out(level, x, y)))
        .expect("such a cell")
}

/// Puts the player at `at` and presses Leave: whether they left.
fn leave_from(test: &mut TestClient, at: (i64, i64)) -> bool {
    test.client.visit.as_mut().unwrap().position = at;
    let answers = test.send(&[ClientMessage::Leave]);
    let left = matches!(answers[..], [ServerMessage::OnWorldMap { .. }, ..]);
    assert_eq!(left, test.client.visit.is_none());
    left
}

#[test]
fn leaves_a_town_from_its_middle_and_comes_back_there() {
    let mut test = enter(PoiKind::Village, 0);
    let middle = find(&test, |cell, edge| cell.is_road() && !edge);
    assert!(leave_from(&mut test, middle));
    let id = SiteId::Dev {
        kind: PoiKind::Village,
        seed: 4,
        ruin: 0,
    };
    assert_eq!(test.client.character.left_at.get(&id), Some(&(0, middle)));
}

#[test]
fn not_from_indoors_or_upstairs() {
    let mut test = enter(PoiKind::Castle, 0);
    let indoors = find(&test, |cell, _| cell == Cell::Floor);
    assert!(!leave_from(&mut test, indoors));
    let mut test = enter(PoiKind::Castle, 1);
    let upstairs = find(&test, |cell, _| cell == Cell::Floor);
    assert!(!leave_from(&mut test, upstairs));
}

#[test]
fn a_cave_only_from_its_edge() {
    let mut test = enter(PoiKind::RockCave, 0);
    let inside = find(&test, |cell, edge| cell == Cell::Ground && !edge);
    assert!(!leave_from(&mut test, inside));
    let edge = find(&test, |_, edge| edge);
    assert!(leave_from(&mut test, edge));
}
