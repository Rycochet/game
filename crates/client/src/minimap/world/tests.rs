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
