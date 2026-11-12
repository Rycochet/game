use super::*;

#[test]
fn cells_and_middles_agree_with_relief() {
    assert_eq!(CELL_SPAN, 240);
    assert_eq!(Spot::middle_of((2, -3)), Spot::new(480, -720));
    assert_eq!(Spot::new(480, -720).cell(), (2, -3));
    // A cell runs 120 units either side of its middle; the edge belongs east.
    assert_eq!(Spot::new(119, -119).cell(), (0, 0));
    assert_eq!(Spot::new(120, -120).cell(), (1, 0));
    assert_eq!(Spot::new(-121, 0).cell(), (-1, 0));
}

#[test]
fn distances_are_whole_square_roots() {
    let a = Spot::new(0, 0);
    assert_eq!(a.distance(Spot::new(300, 400)), 500);
    assert_eq!(a.distance(Spot::new(240, 240)), 339);
    assert_eq!(a.distance_squared(Spot::new(240, 240)), 115_200);
    // Far along the Hall, with its unwrapped `x`.
    let far = Spot::new((1 << 33) * CELL_SPAN, 0);
    assert_eq!(far.distance(far.offset(-480, 0)), 480);
    assert_eq!(root(u128::from(u64::MAX) * u128::from(u64::MAX)), u64::MAX);
}

#[test]
fn towards_cuts_a_move_evenly() {
    let (a, b) = (Spot::new(0, 0), Spot::new(1000, -500));
    assert_eq!(a.towards(b, 0, 4), a);
    assert_eq!(a.towards(b, 4, 4), b);
    assert_eq!(a.towards(b, 1, 4), Spot::new(250, -125));
}
