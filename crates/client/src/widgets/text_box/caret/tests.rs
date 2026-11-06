use super::*;

#[test]
fn the_caret_is_solid_after_moving_then_fades_twice_a_second() {
    assert_eq!(strength(0.0, false), 1.0);
    assert_eq!(strength(0.49, false), 1.0);
    // It starts from solid, so there is no jump.
    assert!((strength(0.5, false) - 1.0).abs() < 1e-5);
    // Half a pulse later it has faded right out, and a whole one, back.
    assert!(strength(0.75, false) < 1e-5);
    assert!((strength(1.0, false) - 1.0).abs() < 1e-5);
    assert!((strength(0.625, false) - 0.5).abs() < 1e-5);
}

#[test]
fn reduce_motion_keeps_it_solid() {
    assert_eq!(strength(0.75, true), 1.0);
}

#[test]
fn writing_slides_to_keep_the_caret_in_sight() {
    // Fits: no sliding.
    assert_eq!(slide(0.0, 50.0, 80.0, 100.0), 0.0);
    // The caret past the right edge slides it left.
    assert_eq!(slide(0.0, 150.0, 200.0, 100.0), 52.0);
    // Back to the start slides it back.
    assert_eq!(slide(52.0, 10.0, 200.0, 100.0), 10.0);
    // Writing deleted from the end leaves no empty room.
    assert_eq!(slide(52.0, 60.0, 90.0, 100.0), 0.0);
}
