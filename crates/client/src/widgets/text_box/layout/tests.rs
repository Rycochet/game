use super::*;

#[test]
fn places_fall_between_the_characters_after_the_prefix() {
    let runs = [
        Run::Prefix(0.0, 10.0),
        Run::Char(0, 10.0, 18.0),
        Run::Char(1, 18.0, 22.0),
        Run::Other,
    ];
    assert_eq!(edges_of(runs, 2), vec![10.0, 18.0, 22.0]);
}

#[test]
fn an_empty_box_has_one_place_after_its_prefix() {
    assert_eq!(edges_of([Run::Prefix(0.0, 7.0)], 0), vec![7.0]);
    assert_eq!(edges_of([], 0), vec![0.0]);
}

#[test]
fn a_character_not_laid_out_yet_takes_no_room() {
    let runs = [Run::Char(0, 0.0, 5.0), Run::Char(2, 5.0, 9.0)];
    assert_eq!(edges_of(runs, 3), vec![0.0, 5.0, 5.0, 9.0]);
}

#[test]
fn the_nearest_place_is_found() {
    let edges = [0.0, 8.0, 12.0, 20.0];
    assert_eq!(nearest(&edges, -5.0), 0);
    assert_eq!(nearest(&edges, 9.0), 1);
    assert_eq!(nearest(&edges, 11.0), 2);
    assert_eq!(nearest(&edges, 100.0), 3);
}
