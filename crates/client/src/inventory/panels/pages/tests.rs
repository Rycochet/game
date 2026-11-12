use super::*;

#[test]
fn the_panel_is_good_markup_with_a_tab_for_each_page() {
    CHARACTER.assert_good();
    let mut at = 0;
    for name in Page::NAMES {
        let tab = format!("<option>{name}</option>");
        let found = CHARACTER.source[at..].find(&tab).expect("in order");
        at += found;
    }
    assert_eq!(Page::ALL.len(), Page::IDS.len());
}
