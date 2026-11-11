use super::*;

#[test]
fn reach_counts_a_diagonal_as_one() {
    let from = MapPosition { x: 0, y: 0 };
    assert!(in_reach(from, MapPosition { x: 2, y: -2 }));
    assert!(!in_reach(from, MapPosition { x: 3, y: 0 }));
}

#[test]
fn the_panel_is_good_markup() {
    NEARBY.assert_good();
}
