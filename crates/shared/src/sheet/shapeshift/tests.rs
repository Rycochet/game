use super::super::{Attribute, Build};
use super::*;

/// A human built leaning one way, so nothing is just the median.
fn person() -> Sheet {
    let mut build = Build::CENTRED;
    build.turn = 40;
    if !build.is_valid() {
        build = Build::CENTRED;
    }
    Sheet::built(CreatureKind::Human, build)
}

#[test]
fn a_form_takes_only_the_body_row() {
    let mut sheet = person();
    let before = sheet.clone();
    sheet.shapeshift(CreatureKind::Human, CreatureKind::Horse);
    let horse = Attributes::for_kind(CreatureKind::Horse, &sheet.build);
    for which in Row::Body.attributes() {
        assert_eq!(sheet.attributes.get(which), horse.get(which), "{which:?}");
    }
    for row in [Row::Mind, Row::Spirit] {
        for which in row.attributes() {
            assert_eq!(sheet.attributes.get(which), before.attributes.get(which));
        }
    }
    assert_eq!(sheet.build, before.build);
    assert_eq!(sheet.skills, before.skills);
    assert_eq!(sheet.appearance, before.appearance);
    assert_eq!(sheet.own_kind(CreatureKind::Horse), CreatureKind::Human);
}

#[test]
fn changing_back_gives_exactly_what_it_had() {
    let mut sheet = person();
    // As if it had grown stronger since it was made.
    sheet.attributes.set(Attribute::Strength, 123);
    let before = sheet.clone();
    sheet.shapeshift(CreatureKind::Human, CreatureKind::Wolf);
    sheet.shapeshift(CreatureKind::Wolf, CreatureKind::Horse);
    assert_eq!(sheet.own_kind(CreatureKind::Horse), CreatureKind::Human);
    sheet.shapeshift(CreatureKind::Horse, CreatureKind::Human);
    assert_eq!(sheet, before);
    assert_eq!(sheet.own_kind(CreatureKind::Human), CreatureKind::Human);
}
