use super::*;
use shared::poi::PoiKind;

#[test]
fn a_found_poi_is_named_in_words() {
    assert_eq!(found(PoiKind::Village), "Found: Village");
    assert!(found(PoiKind::HerbalistHut).starts_with("Found: Herbalist"));
}
