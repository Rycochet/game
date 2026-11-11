use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::rng::Seed;
use shared::site::Site;

use crate::connections::tests::TestClient;

/// What entering a level of a site tells as its name.
fn name_on_entering(kind: PoiKind, seed: u64, level: u32, ruin: u8) -> Option<String> {
    let mut test = TestClient::new();
    let answers = test.send(&[ClientMessage::DevEnterSite {
        kind,
        seed,
        level,
        ruin,
    }]);
    let ServerMessage::EnterLevel(info) = &answers[0] else {
        panic!("{:?}", answers[0]);
    };
    info.place_name.clone()
}

#[test]
fn towns_castles_and_forts_are_named_and_nothing_else_is() {
    for kind in [
        PoiKind::Hamlet,
        PoiKind::Village,
        PoiKind::Town,
        PoiKind::City,
        PoiKind::Castle,
        PoiKind::Fort,
    ] {
        let name = name_on_entering(kind, 4, 0, 0).expect("a name");
        assert!(!name.is_empty());
        // The same each time.
        assert_eq!(name_on_entering(kind, 4, 0, 0), Some(name));
    }
    assert_eq!(name_on_entering(PoiKind::RockCave, 4, 0, 0), None);
    assert_eq!(name_on_entering(PoiKind::StoneTower, 4, 0, 0), None);
}

#[test]
fn a_castle_keeps_its_name_on_every_floor_and_in_ruin() {
    let seed = (1..)
        .find(|&seed| Site::new(PoiKind::Castle, Seed::new(seed)).levels() > 1)
        .unwrap();
    let name = name_on_entering(PoiKind::Castle, seed, 0, 0).expect("a name");
    assert!(name.starts_with("Castle "), "{name}");
    assert_eq!(
        name_on_entering(PoiKind::Castle, seed, 1, 0),
        Some(name.clone())
    );
    assert_eq!(name_on_entering(PoiKind::Castle, seed, 0, 3), Some(name));
}
