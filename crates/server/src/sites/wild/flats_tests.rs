//! A wilderness visit's step heights keep a town's covered cells level and
//! let its open ones follow the land.

use shared::movement::WildHeights;
use shared::protocol::ClientMessage;

use super::tests::chunk_near_start;
use crate::connections::tests::TestClient;

#[test]
fn a_towns_buildings_are_level_and_its_streets_follow_the_land() {
    let mut test = TestClient::new();
    let plan = chunk_near_start(&test, |c| c.poi().kind.is_town());
    let poi = *plan.cutout().unwrap().poi();
    test.client.character.world_position = (poi.x, poi.y);
    test.send(&[ClientMessage::EnterSite { chunk: poi.chunk }]);
    let visit = test.client.visit.as_ref().expect("in the wilderness");
    let bounds = visit.level.level.bounds();
    let flats = visit.flats();
    let heights =
        WildHeights::new(&test.world.ground, bounds).with_flats(&flats, &visit.level.level);
    let land = WildHeights::new(&test.world.ground, bounds);
    assert!(!flats.is_empty(), "a town's buildings");
    let (min_x, min_y, width, height) = bounds;
    let mut covered = 0;
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let cell = visit.level.level.cell(x, y);
            match flats.iter().find(|flat| flat.contains(x, y)) {
                Some(flat) if !cell.is_outside() => {
                    assert_eq!(heights.at(x, y), flat.height);
                    covered += 1;
                }
                _ => assert_eq!(heights.at(x, y), land.at(x, y), "({x}, {y})"),
            }
        }
    }
    assert!(covered > 0);
}
