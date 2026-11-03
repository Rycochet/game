use super::*;

#[test]
fn the_filter_ignores_case_and_matches_anywhere() {
    assert!(matches_filter("Teleport Mode", "PORT"));
    assert!(matches_filter("Teleport Mode", ""));
    assert!(!matches_filter("Teleport Mode", "stairs"));
}

#[test]
fn arrows_move_through_the_columns_and_stop_at_the_ends() {
    use HighlightStep::*;
    let go = |from, step| next_highlight(from, 10, 4, step);
    assert_eq!(go(None, Down), Some(0));
    assert_eq!(go(None, Up), Some(9));
    assert_eq!(go(Some(3), Down), Some(4));
    assert_eq!(go(Some(9), Down), Some(9));
    assert_eq!(go(Some(0), Up), Some(0));
    assert_eq!(go(Some(1), Right), Some(5));
    assert_eq!(go(Some(5), Left), Some(1));
    assert_eq!(go(Some(1), Left), Some(1));
    // No such row in the next column: stays.
    assert_eq!(go(Some(7), Right), Some(7));
    assert_eq!(next_highlight(None, 0, 4, Down), None);
}
