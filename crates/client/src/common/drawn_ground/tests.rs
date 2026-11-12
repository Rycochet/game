use super::*;

fn ground() -> DrawnGround {
    let relief = Relief::from_fn((0, 0, 3, 3), |x, y| {
        (x * 50 + y * 20) as i32 + if x == 1 { 60 } else { 0 }
    });
    DrawnGround {
        relief: Some(relief),
        reference: 30,
        land: None,
    }
}

#[test]
fn corners_and_middles_are_drawn_where_the_rule_puts_them() {
    let ground = ground();
    let relief = ground.relief.as_ref().unwrap();
    for y in 0..3 {
        for x in 0..3 {
            let (fx, fy) = (x as f32, y as f32);
            let corner = ground.drawn_between(fx - 0.5, fy - 0.5);
            assert!((corner - ground.corner(x, y)).abs() < 1e-5, "({x}, {y})");
            // The middle is the corners' exact average, the rule's rounded.
            assert!((ground.drawn_between(fx, fy) - ground.drawn(x, y)).abs() < 0.01);
            // Anywhere, the rule in whole units agrees to a unit.
            for (ox, oy) in [(0.2, -0.1), (-0.37, 0.3), (0.1, 0.45)] {
                let units = relief.at(((fx + ox) * 240.0) as i64, ((fy + oy) * 240.0) as i64);
                let drawn = ground.drawn_between(fx + ox, fy + oy) * 240.0 + 30.0;
                assert!(
                    (drawn - units as f32).abs() <= 1.5,
                    "{drawn} against {units}"
                );
            }
        }
    }
}

#[test]
fn flat_ground_is_at_nought() {
    let ground = DrawnGround::default();
    assert!(!ground.slopes());
    assert_eq!(ground.drawn_between(3.3, -2.1), 0.0);
    assert!(ground.covers(1000, 1000));
}
