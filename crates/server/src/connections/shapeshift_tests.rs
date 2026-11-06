//! Changing kind through a connected client: for good, or shapeshifted for a while.

use shared::creature::CreatureKind;
use shared::protocol::{ClientMessage, KindChange};
use shared::sheet::{Attributes, Row, Sheet};

use super::tests::TestClient;

fn change(kind: CreatureKind, how: KindChange) -> ClientMessage {
    ClientMessage::DevSetCreature { kind, how }
}

#[test]
fn a_shapeshift_takes_the_forms_body_row_and_changes_back() {
    let mut test = TestClient::new();
    test.enter_a_castle();
    let before = test.client.sheet.clone();
    test.send(&[change(CreatureKind::Horse, KindChange::Temporary)]);
    assert_eq!(test.client.inventory.creature, CreatureKind::Horse);
    let sheet = &test.client.sheet;
    let horse = Attributes::for_kind(CreatureKind::Horse, &sheet.build);
    for which in Row::Body.attributes() {
        assert_eq!(sheet.attributes.get(which), horse.get(which));
    }
    for which in Row::Mind
        .attributes()
        .into_iter()
        .chain(Row::Spirit.attributes())
    {
        assert_eq!(sheet.attributes.get(which), before.attributes.get(which));
    }
    assert_eq!(sheet.own_kind(CreatureKind::Horse), CreatureKind::Human);
    // Through another form and back to its own: as it was.
    test.send(&[change(CreatureKind::Wolf, KindChange::Temporary)]);
    test.send(&[change(CreatureKind::Human, KindChange::Temporary)]);
    assert_eq!(test.client.sheet.attributes, before.attributes);
    assert_eq!(test.client.sheet.true_form, None);
}

#[test]
fn a_permanent_change_makes_the_new_kind_its_own() {
    let mut test = TestClient::new();
    test.enter_a_castle();
    test.send(&[change(CreatureKind::Horse, KindChange::Temporary)]);
    test.send(&[change(CreatureKind::Kobold, KindChange::Permanent)]);
    let sheet = &test.client.sheet;
    assert_eq!(sheet.true_form, None);
    assert_eq!(sheet.own_kind(CreatureKind::Kobold), CreatureKind::Kobold);
    assert_eq!(
        sheet.attributes,
        Sheet::new(CreatureKind::Kobold).attributes
    );
}
