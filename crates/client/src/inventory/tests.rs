use super::hold_items::is_click;
use super::*;

#[test]
fn only_a_quick_press_in_one_place_is_a_click() {
    let from = Vec2::new(100.0, 100.0);
    assert!(is_click(from, from + Vec2::X, 0.1));
    assert!(!is_click(from, from + Vec2::X, CLICK_SECONDS));
    assert!(!is_click(from, from + Vec2::X * CLICK_PIXELS, 0.1));
}
