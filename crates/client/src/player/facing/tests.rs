use super::*;

#[test]
fn turn_left_goes_from_east_towards_north_and_right_towards_south() {
    let eighth = i32::from(EIGHTH);
    let left = turned_by_hand(Heading::EAST, -1, eighth);
    let right = turned_by_hand(Heading::EAST, 1, eighth);
    assert_eq!(left.apart(Heading::NORTH), EIGHTH);
    assert_eq!(right.apart(Heading::SOUTH), EIGHTH);
    assert_eq!(
        turned_by_hand(Heading::NORTH, -1, 2 * eighth),
        Heading::WEST
    );
}
