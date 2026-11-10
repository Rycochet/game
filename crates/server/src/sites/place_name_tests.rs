use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};

use crate::connections::tests::TestClient;

/// What entering a site of a kind tells as its name.
fn name_on_entering(kind: PoiKind) -> Option<String> {
    let mut test = TestClient::new();
    let answers = test.send(&[ClientMessage::DevEnterSite {
        kind,
        seed: 4,
        level: 0,
        ruin: 0,
    }]);
    let ServerMessage::EnterLevel(info) = &answers[0] else {
        panic!("{:?}", answers[0]);
    };
    info.place_name.clone()
}

#[test]
fn towns_are_named_on_entering_and_nothing_else_is() {
    for kind in [
        PoiKind::Hamlet,
        PoiKind::Village,
        PoiKind::Town,
        PoiKind::City,
    ] {
        let name = name_on_entering(kind).expect("a town's name");
        assert!(!name.is_empty());
        // The same each time.
        assert_eq!(name_on_entering(kind), Some(name));
    }
    assert_eq!(name_on_entering(PoiKind::Castle), None);
}
