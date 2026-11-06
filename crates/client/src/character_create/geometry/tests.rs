use super::*;

#[test]
fn body_starts_at_the_top_and_the_others_a_third_of_a_turn_on() {
    assert!(centre(Row::Body, 0).y < MIDDLE.y);
    assert!((centre(Row::Body, 0).x - MIDDLE.x).abs() < 1e-5);
    // Mind is lower right, Spirit lower left: clockwise, as the shared rules count.
    assert!(centre(Row::Mind, 0).x > MIDDLE.x && centre(Row::Mind, 0).y > MIDDLE.y);
    assert!(centre(Row::Spirit, 0).x < MIDDLE.x);
    // Turned half way, Body is at the bottom.
    assert!(centre(Row::Body, 180).y > MIDDLE.y);
}

#[test]
fn everything_stays_inside_the_square() {
    for turn in (0..360).step_by(15) {
        for row in Row::ALL {
            for corner in corners(row, turn) {
                assert!((0.0..=1.0).contains(&corner.x) && (0.0..=1.0).contains(&corner.y));
            }
        }
    }
}

#[test]
fn a_dot_goes_back_to_the_same_weights() {
    let corners = corners(Row::Mind, 37);
    for dot in [Dot::CENTRE, Dot([1000, 0, 0]), Dot([200, 500, 300])] {
        let build = Build {
            dots: [Dot::CENTRE, dot, Dot::CENTRE],
            turn: 37,
        };
        let found = dot_at(dot_place(Row::Mind, &build), corners);
        for (a, b) in found.0.iter().zip(dot.0) {
            assert!(a.abs_diff(b) <= 1, "{found:?} {dot:?}");
        }
        assert!(found.is_valid());
    }
}

#[test]
fn a_point_outside_gives_the_nearest_dot_inside() {
    let corners = corners(Row::Body, 0);
    // Far beyond the first corner: all of it there.
    let far = corners[0] + (corners[0] - centre(Row::Body, 0)) * 3.0;
    assert_eq!(dot_at(far, corners), Dot([1000, 0, 0]));
    assert!(!near_triangle(far, corners, 0.01));
    assert!(near_triangle(centre(Row::Body, 0), corners, 0.0));
}

#[test]
fn turning_wraps_round() {
    assert_eq!(turned_by(350, 20.0), 10);
    assert_eq!(turned_by(5, -10.0), 355);
    assert!((angle_round_middle(MIDDLE + Vec2::new(0.0, -1.0))).abs() < 1e-4);
    assert!((angle_round_middle(MIDDLE + Vec2::new(1.0, 0.0)) - 90.0).abs() < 1e-4);
}

#[test]
fn shapes_measure_inside_and_out_even_when_flat() {
    let triangle = [Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)];
    assert!(distance_to_shape(Vec2::new(0.2, 0.2), &triangle) < 0.0);
    assert!((distance_to_shape(Vec2::new(-1.0, 0.0), &triangle) - 1.0).abs() < 1e-5);
    // All three at one point: a round dot.
    let point = [Vec2::ZERO; 3];
    assert!((distance_to_shape(Vec2::new(0.3, 0.4), &point) - 0.5).abs() < 1e-5);
    // Two at one point: a line.
    let line = [Vec2::ZERO, Vec2::ZERO, Vec2::new(1.0, 0.0)];
    assert!((distance_to_shape(Vec2::new(0.5, 0.25), &line) - 0.25).abs() < 1e-5);
}

#[test]
fn a_chart_reaches_its_corners_at_the_most_shown() {
    let corners = corners(Row::Spirit, 90);
    let most = SHOWN_MOST as u16;
    assert_eq!(chart(Row::Spirit, 90, [most; 3]), corners);
    let none = chart(Row::Spirit, 90, [0; 3]);
    assert!(
        none.iter()
            .all(|&point| point.distance(centre(Row::Spirit, 90)) < 1e-5)
    );
}
