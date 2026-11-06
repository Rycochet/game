use super::*;

const ITEM: Id = Id::NOBODY;

#[test]
fn an_item_is_asked_after_once_until_its_answer_is_old() {
    let mut known = KnownItems::default();
    known.want(ITEM, 0.0);
    assert_eq!(known.take_wanted().len(), 1);
    known.set(ITEM, KnownItem::Asked(0.0));
    known.want(ITEM, 5.0);
    assert!(known.take_wanted().is_empty(), "being asked");
    known.want(ITEM, 11.0);
    assert_eq!(known.take_wanted().len(), 1, "no answer: asked again");
    known.set(ITEM, KnownItem::Unknown(11.0));
    known.want(ITEM, 60.0);
    assert!(known.take_wanted().is_empty());
    known.want(ITEM, 71.0 + 1.0);
    assert_eq!(known.take_wanted().len(), 1, "a minute on: asked again");
}
