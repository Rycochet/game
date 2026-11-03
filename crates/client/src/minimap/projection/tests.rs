use super::*;

fn turned(right: (f64, f64), down: (f64, f64)) -> Projection {
    Projection {
        centre: (100.0, 200.0),
        middle: (50.0, 50.0),
        units_per_pixel: 2.0,
        right,
        down,
    }
}

#[test]
fn grid_and_picture_positions_turn_into_each_other() {
    let north_up = turned((1.0, 0.0), (0.0, 1.0));
    assert_eq!(north_up.to_grid(60.0, 40.0), (120.0, 180.0));
    assert_eq!(north_up.to_picture(120.0, 180.0), (60.0, 40.0));
    // East at the top: right on the picture is south.
    let east_up = turned((0.0, 1.0), (-1.0, 0.0));
    assert_eq!(east_up.to_grid(60.0, 40.0), (120.0, 220.0));
    assert_eq!(east_up.to_picture(120.0, 220.0), (60.0, 40.0));
}

#[test]
fn a_drag_stops_at_the_edge_of_what_is_allowed() {
    let inside = |(x, y): (f64, f64)| x < 10.0 && y < 10.0;
    let (x, y) = slide((0.0, 0.0), (100.0, 5.0), inside);
    // It stops short in x, and still moves in y.
    assert!((9.0..10.0).contains(&x));
    assert_eq!(y, 5.0);
    assert_eq!(slide((0.0, 0.0), (3.0, 4.0), inside), (3.0, 4.0));
}
