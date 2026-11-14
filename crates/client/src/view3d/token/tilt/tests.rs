use super::*;

const RADIUS: f32 = 8.0;
const SINK: f32 = 0.6;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn flat_ground_lies_flat_where_it_is() {
    let tilt = BaseTilt::fit(RADIUS, SINK, |_, _| 5.0);
    assert!(close(tilt.bottom, 5.0));
    assert!(tilt.normal.abs_diff_eq(Vec3::Y, 1e-5));
    assert!(tilt.rotation().abs_diff_eq(Quat::IDENTITY, 1e-5));
}

#[test]
fn an_even_slope_is_followed_without_lift() {
    // Rising a half to the east and a quarter to the south.
    let tilt = BaseTilt::fit(RADIUS, SINK, |x, z| 2.0 + 0.5 * x + 0.25 * z);
    assert!(close(tilt.bottom, 2.0));
    let wanted = Vec3::new(-0.5, 1.0, -0.25).normalize();
    assert!(tilt.normal.abs_diff_eq(wanted, 1e-4));
    // The rotation turns the base's up to the slope's.
    assert!((tilt.rotation() * Vec3::Y).abs_diff_eq(wanted, 1e-4));
}

#[test]
fn a_steep_step_up_to_a_flat_is_cleared() {
    // Flat to the west, then steeply up to a building's flat level.
    let ground = |x: f32, _z: f32| (x * 2.0).clamp(0.0, 6.0);
    let tilt = BaseTilt::fit(RADIUS, SINK, ground);
    assert!(tilt.normal.x < 0.0, "leans back from the rise");
    // No point under the base (on its rings or between) rises above its
    // bottom's plane.
    let plane = |x: f32, z: f32| {
        // Height of the bottom plane at (x, z): normal · (p - bottom) = 0.
        let n = tilt.normal;
        tilt.bottom - (n.x * x + n.z * z) / n.y
    };
    for i in 0..64 {
        let angle = i as f32 * TAU / 64.0;
        for reach in [0.0, RADIUS / 2.0, RADIUS] {
            let (x, z) = (reach * angle.cos(), reach * angle.sin());
            assert!(
                ground(x, z) <= plane(x, z) + SINK + 0.05,
                "poking through at {x}, {z}"
            );
        }
    }
}

#[test]
fn a_crease_lifts_the_base_onto_its_sides() {
    // A valley along z: the sides are higher than the middle.
    let tilt = BaseTilt::fit(RADIUS, SINK, |x, _| x.abs());
    assert!(tilt.normal.abs_diff_eq(Vec3::Y, 1e-4));
    assert!(close(tilt.bottom, RADIUS - SINK));
}

#[test]
fn up_runs_along_the_tilt() {
    let tilt = BaseTilt::fit(RADIUS, SINK, |x, _| x);
    let up = tilt.up(1.2);
    assert!(close(up.length(), 1.2));
    assert!(up.x < 0.0 && up.y > 0.0);
}
