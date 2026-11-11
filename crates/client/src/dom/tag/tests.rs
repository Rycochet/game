use super::*;

#[test]
fn every_tag_is_found_by_its_name() {
    for (name, tag) in ALL {
        assert_eq!(Tag::named(name), Some(tag));
        assert_eq!(tag.name(), name);
    }
    assert_eq!(Tag::named("blink"), None);
}

#[test]
fn table_parts_belong_in_tables() {
    assert_eq!(Tag::Td.belongs_in(), [Tag::Tr]);
    assert_eq!(Tag::Tr.belongs_in(), [Tag::Table]);
    assert_eq!(Tag::Table.holds(), Holds::Only(&[Tag::Tr]));
    assert!(Tag::Div.belongs_in().is_empty());
}
