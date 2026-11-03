use bevy::prelude::*;

use super::*;

#[test]
fn typing_narrows_the_options_keeping_their_indexes() {
    let options: Vec<String> = ["Castle", "Fort", "Stone Tower"].map(String::from).into();
    assert_eq!(on_show(&options, ""), [0, 1, 2]);
    assert_eq!(on_show(&options, "T"), [0, 1, 2]);
    assert_eq!(on_show(&options, "tower"), [2]);
    assert!(on_show(&options, "cave").is_empty());
}

#[test]
fn the_rows_drawn_follow_the_highlight_and_stop_at_the_end() {
    assert_eq!(window_start(0, Some(3), 40), 0);
    assert_eq!(window_start(0, Some(ROWS), 40), 1);
    assert_eq!(window_start(10, Some(4), 40), 4);
    assert_eq!(window_start(100, None, 40), 40 - ROWS);
    assert_eq!(window_start(5, None, 3), 0);
}

#[test]
fn a_list_goes_under_its_button_or_above_if_there_is_no_room() {
    let window = Vec2::new(800.0, 600.0);
    let button = Rect::new(10.0, 100.0, 110.0, 130.0);
    assert_eq!(
        list_corner(button, 200.0, 100.0, window),
        Vec2::new(10.0, 130.0 + GAP)
    );
    let low = Rect::new(750.0, 500.0, 790.0, 530.0);
    let corner = list_corner(low, 200.0, 100.0, window);
    assert_eq!(corner, Vec2::new(700.0, 500.0 - GAP - 200.0));
}
