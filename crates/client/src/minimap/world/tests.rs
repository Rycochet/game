use super::*;

#[test]
fn every_column_is_found_with_the_last_one_wider() {
    // 650 wide: ten columns, the last of them 74 wide.
    assert_eq!(columns_between(0, 100, 650), [0, 1]);
    assert_eq!(columns_between(-20, 5, 650), [9, 0]);
    assert_eq!(columns_between(-100, -20, 650), [8, 9]);
    assert_eq!(columns_between(-5, 700, 650).len(), 10);
    assert_eq!(chunk_of(649, 0, 650), (9, 0));
}

#[test]
fn chunks_wrap_round_the_planet_only_east_west() {
    assert_eq!(chunk_of(0, 0, 640), (0, 0));
    assert_eq!(chunk_of(-1, -1, 640), (9, -1));
    assert_eq!(chunk_of(640, 64, 640), (0, 1));
}

#[test]
fn a_chunk_is_coarse_first_and_detailed_only_where_the_zoom_shows_it() {
    let coarse = Tile {
        step: COARSE_STEP,
        across: 16,
        colours: Vec::new(),
    };
    let detailed = Tile {
        step: 1,
        across: 64,
        colours: Vec::new(),
    };
    // Nothing yet: the quick version, however close the zoom.
    assert_eq!(next_step(None, 0.5), Some(COARSE_STEP));
    // A pixel covering fewer points than the coarse step shows the difference.
    assert_eq!(next_step(Some(&coarse), 0.5), Some(1));
    assert_eq!(next_step(Some(&coarse), COARSE_STEP as f64), None);
    assert_eq!(next_step(Some(&detailed), 0.5), None);
}
