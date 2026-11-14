use super::show::ITEM_POPUP;

#[test]
fn the_popup_is_good_markup() {
    ITEM_POPUP.assert_good();
}
