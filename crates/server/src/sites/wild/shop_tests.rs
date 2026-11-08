use shared::poi::Poi;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::shop::sells;
use shared::site::{Generator, Site};

use super::tests::{chunk_near_start, position};
use crate::connections::tests::TestClient;
use crate::storage::SiteId;

/// Whether a poi's town has a shop.
fn has_shop(poi: &Poi) -> bool {
    let generator = Generator::for_site(poi.kind, poi.seed, poi.desert);
    let site = Site::with_generator(poi.kind, poi.seed, generator.version(), generator);
    let first = site.expect("a site").level(0);
    let mut town = first.buildings.iter();
    town.any(|b| !sells(b.look.kind).is_empty())
}

fn stock_in(answers: &[ServerMessage]) -> Option<usize> {
    answers.iter().find_map(|answer| match answer {
        ServerMessage::TradeStock { items, .. } => Some(items.len()),
        _ => None,
    })
}

#[test]
fn a_town_set_into_the_wilderness_trades_with_its_own_keepers() {
    let mut test = TestClient::new();
    let plan = chunk_near_start(&test, |c| c.poi().kind.is_town() && has_shop(c.poi()));
    let poi = *plan.cutout().unwrap().poi();
    test.client.character.world_position = (poi.x, poi.y);
    let answers = test.send(&[ClientMessage::Down]);
    let keepers = answers.iter().find_map(|answer| match answer {
        ServerMessage::Keepers(keepers) if !keepers.is_empty() => Some(keepers.clone()),
        _ => None,
    });
    let keepers = keepers.expect("the town's keepers, with its buildings");
    let buildings = answers.iter().find_map(|answer| match answer {
        ServerMessage::Buildings(buildings) => Some(buildings.clone()),
        _ => None,
    });
    let buildings = buildings.expect("the town's buildings");
    // The shop nearest where they arrived, so its door is in the window.
    let (x, y) = position(&test);
    let (building, with) = keepers
        .iter()
        .copied()
        .min_by_key(|(id, _)| {
            let look = buildings.iter().find(|b| b.id == *id).expect("its look");
            (look.door.0 - x).abs() + (look.door.1 - y).abs()
        })
        .unwrap();
    let look = buildings.iter().find(|b| b.id == building).unwrap();
    let front = (look.door.0 + look.faces.0, look.door.1 + look.faces.1);
    let open = ClientMessage::OpenTrade { with };

    let far = (front.0 + 30, front.1);
    test.send(&[ClientMessage::DevTeleport { x: far.0, y: far.1 }]);
    assert_eq!(position(&test), far);
    assert_eq!(stock_in(&test.send(std::slice::from_ref(&open))), None);

    test.send(&[ClientMessage::DevTeleport {
        x: front.0,
        y: front.1,
    }]);
    assert_eq!(position(&test), front);
    let stock = stock_in(&test.send(std::slice::from_ref(&open))).expect("it opens");
    assert!(stock > 0);
    // The keeper and stock are the town's own, kept in its save.
    let saved = test.world.site(SiteId::Poi(poi.chunk)).levels.get(&0);
    let saved = saved.expect("the town's first level");
    assert_eq!(saved.buildings[&building].keeper, Some(with));
    assert_eq!(saved.records[&with].carried.len(), stock);
}
