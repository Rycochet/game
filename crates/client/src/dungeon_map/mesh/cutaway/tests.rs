use super::*;

#[test]
fn walls_near_the_player_are_stubs() {
    assert_eq!(cut_height((0, 0), (0, 1)), Some(STUB_HEIGHT));
    assert_eq!(cut_height((0, 0), (4, 0)), Some(STUB_HEIGHT));
}

#[test]
fn walls_far_away_stand_whole() {
    assert_eq!(cut_height((0, 0), (6, 0)), None);
    assert_eq!(cut_height((3, 3), (3, 20)), None);
}

#[test]
fn walls_rise_steadily_between() {
    let heights: Vec<f32> = (0..8)
        .map(|x| cut_height((0, 0), (x, 0)).unwrap_or(WALL_HEIGHT))
        .collect();
    assert!(heights.windows(2).all(|pair| pair[1] >= pair[0]));
    let between = cut_height((0, 0), (5, 0)).unwrap();
    assert!(between > STUB_HEIGHT && between < WALL_HEIGHT);
}
