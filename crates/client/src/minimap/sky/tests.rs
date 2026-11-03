use super::*;
use shared::celestial::{Body, ColourName};

fn body(kind: BodyKind, direction: [f64; 3]) -> BodyNow {
    BodyNow {
        body: Body {
            kind,
            colour: [1.0, 0.5, 0.2],
            colour_name: Some(ColourName::Orange),
            size: 1.0,
        },
        direction,
        lit: 1.0,
        waxing: true,
        along: direction[0],
        light: [0.0; 3],
    }
}

fn at(direction: [f64; 3]) -> Vec2 {
    icons(&[body(BodyKind::Sun, direction)], Vec2::ZERO, 100.0, 1.0)[0].at
}

#[test]
fn they_stay_above_the_middle_and_rise_from_behind_the_map() {
    // Whichever way, low down they are wholly behind the circle.
    let sun_radius = radius(&body(BodyKind::Sun, [0.0, 1.0, 0.0]));
    for direction in [[1.0, 0.001, 0.0], [0.0, 0.001, 1.0], [0.0, 0.001, -1.0]] {
        assert!(
            at(direction).length() + sun_radius <= 100.0,
            "{direction:?}"
        );
    }
    for up in [0.01, 0.1, 0.25, 0.6, 0.9] {
        let flat = (1.0_f64 - up * up).sqrt();
        for way in [flat, -flat] {
            let found = at([way, up, 0.0]);
            assert!(found.y <= 0.0, "{found}");
        }
    }
    // Straight overhead: at the top of the circle, lifted.
    let top = at([0.0, 1.0, 0.0]);
    assert!((top - Vec2::new(0.0, -100.0 - LIFT)).length() < 0.01);
    assert!(top.y.abs() + SUN_LARGEST <= 100.0 + BAND);
}

#[test]
fn once_risen_they_keep_to_a_circle_as_big_as_the_map() {
    let centre = Vec2::new(0.0, -LIFT);
    for (east, up) in [(0.95, 0.31), (0.8, 0.6), (0.3, 0.95), (-0.8, 0.6)] {
        let found = at([east, up, 0.0]);
        assert!((found.distance(centre) - 100.0).abs() < 0.01);
        assert_eq!(found.x > 0.0, east > 0.0);
    }
    // Low to the side, it is still partly behind the map's edge.
    let low = at([0.95, 0.31, 0.0]);
    assert!(low.length() - 100.0 < radius(&body(BodyKind::Sun, [0.0, 1.0, 0.0])));
}

#[test]
fn they_go_by_their_way_across_the_sky_not_the_compass() {
    // Early on its way it is on the left, whichever way the world turns.
    let mut early = body(BodyKind::Sun, [0.8, 0.6, 0.0]);
    early.along = -0.8;
    let found = icons(&[early], Vec2::ZERO, 100.0, 1.0)[0].at;
    assert!(found.x < 0.0);
    early.along = 0.8;
    let found = icons(&[early], Vec2::ZERO, 100.0, 1.0)[0].at;
    assert!(found.x > 0.0);
}

#[test]
fn those_below_the_horizon_are_not_shown_nor_those_still_behind_the_map() {
    let now = [
        body(BodyKind::Sun, [1.0, -0.1, 0.0]),
        body(BodyKind::Moon, [1.0, 0.0001, 0.0]),
    ];
    let shown = icons(&now, Vec2::ZERO, 100.0, 1.0);
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].number, 1);
    assert_eq!(under(&shown, Vec2::ZERO, 100.0, shown[0].at), None);
    let high = [body(BodyKind::Moon, [0.5, 0.86, 0.0])];
    let shown = icons(&high, Vec2::ZERO, 100.0, 1.0);
    assert_eq!(under(&shown, Vec2::ZERO, 100.0, shown[0].at), Some(0));
    assert_eq!(under(&shown, Vec2::ZERO, 100.0, Vec2::ZERO), None);
}

#[test]
fn nothing_is_drawn_on_the_border_or_notch() {
    const SIZE: u32 = 80;
    let map = RoundFrame {
        centre: (40.0, 60.0),
        radius: 30.0,
        border: 3.0,
        notch: 6.0,
        north: (0.0, -1.0),
    };
    // Straddling the top of the outline, over the notch, and off to its side.
    for x in [40.0, 48.0, 56.0] {
        let at = Vec2::new(x, 28.0);
        for kind in [BodyKind::Sun, BodyKind::Moon] {
            let mut pixels = vec![0; (SIZE * SIZE * 4) as usize];
            let mut canvas = Canvas {
                pixels: &mut pixels,
                width: SIZE,
                height: SIZE,
            };
            let shown = [Icon {
                number: 0,
                at,
                radius: 10.0,
            }];
            draw(
                &mut canvas,
                &shown,
                &[body(kind, [0.0, 1.0, 0.0])],
                &map,
                1.0,
            );
            let mut drawn = 0;
            for (index, pixel) in pixels.as_chunks::<4>().0.iter().enumerate() {
                if pixel[3] > 0 {
                    drawn += 1;
                    let (x, y) = (index as u32 % SIZE, index as u32 / SIZE);
                    let distance = map.distance(x as f32 + 0.5, y as f32 + 0.5);
                    assert!(distance >= -0.5, "{kind:?} at {x},{y}: {distance}");
                }
            }
            assert!(drawn > 0, "{kind:?} at {at}");
        }
    }
}

#[test]
fn they_are_named_by_colour_if_they_have_one() {
    let mut moon = body(BodyKind::Moon, [0.0, 1.0, 0.0]);
    assert_eq!(name(&moon), "Orange moon");
    moon.body.colour_name = None;
    assert_eq!(name(&moon), "Moon");
}
