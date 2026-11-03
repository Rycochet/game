use super::super::canvas::{Canvas, polygon_distance};
use super::stairs_icon::stairs_corners;
use super::*;

#[test]
fn the_notch_runs_from_inside_the_border_to_just_past_it_towards_north() {
    let frame = RoundFrame {
        centre: (50.0, 50.0),
        radius: 30.0,
        border: 5.0,
        notch: 10.0,
        north: (1.0, 0.0),
    };
    let corners = frame.notch_corners();
    assert_eq!(corners[0], (90.0, 50.0));
    // Its base is on the inside line of the border.
    assert_eq!(corners[1].0, 75.0);
    assert_eq!(corners[2].0, 75.0);
    assert!(frame.distance(50.0, 50.0) < 0.0);
    // Out past the circle on that side, not on the other, and its foot spreads
    // along the border.
    assert!(frame.distance(85.0, 50.0) < 0.0);
    assert!(frame.distance(15.0, 50.0) > 0.0);
    assert!(frame.distance(80.0, 54.0) < 0.0);
}

#[test]
fn the_frame_cuts_the_picture_round_and_borders_it() {
    let mut pixels = vec![9; 4 * 40 * 40];
    let mut canvas = Canvas {
        pixels: &mut pixels,
        width: 40,
        height: 40,
    };
    RoundFrame {
        centre: (20.0, 20.0),
        radius: 10.0,
        border: 3.0,
        notch: 5.0,
        north: (0.0, -1.0),
    }
    .draw(&mut canvas);
    let alpha = |x: usize, y: usize| pixels[(y * 40 + x) * 4 + 3];
    assert_eq!(alpha(0, 0), 0);
    // Over the map, and on the border (solid).
    assert_eq!(pixels[(20 * 40 + 20) * 4], 9);
    assert_eq!(alpha(20, 11), 255);
    assert_ne!(pixels[(11 * 40 + 20) * 4], 9);
}

#[test]
fn the_compass_stands_on_its_foot_with_its_arrow_above() {
    let mut pixels = vec![0; 4 * 100 * 100];
    let mut canvas = Canvas {
        pixels: &mut pixels,
        width: 100,
        height: 100,
    };
    compass(&mut canvas, (50.0, 90.0), 30.0, 5.0);
    let solid = |x: usize, y: usize| pixels[(y * 100 + x) * 4 + 3] == 255;
    // The stem, the crossbar, and the arrow above them.
    assert!(solid(50, 80));
    assert!(solid(42, 80));
    assert!(!solid(40, 85));
    assert!(solid(50, 55));
}

#[test]
fn stairs_are_highest_on_the_left_going_down_and_the_right_going_up() {
    let height_of = |down: bool, x: f32| {
        let corners = stairs_corners((0.0, 0.0), 8.0, down);
        (-40..40)
            .filter(|y| polygon_distance(&corners, x, *y as f32 / 10.0 + 0.05) < 0.0)
            .count()
    };
    // Four steps, 2 wide and 2 high, across 8.
    let down: Vec<_> = [-3.0, -1.0, 1.0, 3.0]
        .iter()
        .map(|&x| height_of(true, x))
        .collect();
    assert_eq!(down, [80, 60, 40, 20]);
    let up: Vec<_> = [-3.0, -1.0, 1.0, 3.0]
        .iter()
        .map(|&x| height_of(false, x))
        .collect();
    assert_eq!(up, [20, 40, 60, 80]);
}

#[test]
fn the_arrow_beside_the_mark_is_white_edged_in_black_and_points_its_way() {
    let mut pixels = vec![100; 4 * 40 * 40];
    let mut canvas = Canvas {
        pixels: &mut pixels,
        width: 40,
        height: 40,
    };
    floor_hint(&mut canvas, (10.0, 20.0), 5.0, 16.0, true);
    let colour = |x: usize, y: usize| pixels[(y * 40 + x) * 4];
    let white = |y: usize| (0..40).filter(|&x| colour(x, y) == 255).count();
    // Up: the head, at the top, is wider than the shaft.
    assert!(white(19) > white(25), "{} {}", white(19), white(25));
    assert!(white(25) > 0);
    assert!((0..40).any(|x| colour(x, 25) == 0));
}

#[test]
fn the_arrow_sits_on_the_edge_towards_the_player() {
    let size = (100.0, 100.0);
    assert_eq!(arrow_place(size, (60.0, 40.0), 10.0), None);
    let (place, direction) = arrow_place(size, (500.0, 50.0), 10.0).unwrap();
    assert_eq!(place, (90.0, 50.0));
    assert_eq!(direction, (1.0, 0.0));
    let (place, _) = arrow_place(size, (-150.0, -150.0), 10.0).unwrap();
    assert_eq!(place, (10.0, 10.0));
}
