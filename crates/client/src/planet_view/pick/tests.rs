use super::*;

const WIDTH: i64 = 4000;

/// A camera `distance` out along `from`, looking at the globe with north up.
fn camera(from: Vec3, distance: f32) -> Transform {
    Transform::from_translation(from.normalize() * distance).looking_at(Vec3::ZERO, Vec3::Y)
}

fn sight(camera: &Transform) -> Sight<'_> {
    Sight {
        camera,
        globe: Quat::IDENTITY,
        across: 800.0,
        ring: 5.0 / 1024.0,
    }
}

#[test]
fn the_picture_runs_round_the_equator_from_x_east_and_down_from_the_north_pole() {
    assert!(on_globe(Vec2::new(0.0, 0.5)).distance(Vec3::X) < 1e-6);
    assert!(on_globe(Vec2::new(0.0, 0.0)).distance(Vec3::Y) < 1e-6);
    assert!(on_globe(Vec2::new(0.3, 1.0)).distance(-Vec3::Y) < 1e-6);
    // A quarter of the way round faces -Z, three quarters +Z.
    assert!(on_globe(Vec2::new(0.25, 0.5)).distance(-Vec3::Z) < 1e-6);
    assert!(on_globe(Vec2::new(0.75, 0.5)).distance(Vec3::Z) < 1e-6);
}

#[test]
fn gates_land_on_the_picture_as_their_rings_are_drawn() {
    assert_eq!(gate_spot((0, 0), WIDTH), Vec2::new(0.0, 0.5));
    assert_eq!(
        gate_spot((WIDTH / 2, -WIDTH / 4), WIDTH),
        Vec2::new(0.5, 0.0)
    );
    // Round the world wraps.
    assert_eq!(gate_spot((WIDTH + 1000, 0), WIDTH), Vec2::new(0.25, 0.5));
}

#[test]
fn the_middle_of_the_globe_is_the_middle_of_the_picture() {
    let camera = camera(Vec3::new(0.3, 0.4, 1.0), 6.0);
    let middle = on_picture(Vec3::ZERO, &camera).unwrap();
    assert!(middle.distance(Vec2::splat(0.5)) < 1e-5, "{middle}");
    // North is up the picture, east (from +Z, -X is to the left) to the right.
    let camera = self::camera(Vec3::Z, 6.0);
    assert!(on_picture(Vec3::Y, &camera).unwrap().y < 0.5);
    assert!(on_picture(Vec3::X, &camera).unwrap().x > 0.5);
    // Behind the camera shows nowhere.
    assert_eq!(on_picture(Vec3::Z * 7.0, &camera), None);
}

#[test]
fn the_gate_facing_the_camera_is_found_where_it_shows() {
    // Gate 1 faces +Z (three quarters round), gate 0 faces +X.
    let spots = [Vec2::new(0.0, 0.5), Vec2::new(0.75, 0.5)];
    let camera = camera(Vec3::Z, 6.0);
    let sight = sight(&camera);
    assert_eq!(gate_under(Vec2::splat(0.5), &spots, &sight), Some(1));
    // A few pixels off still counts, far off doesn't.
    assert_eq!(gate_under(Vec2::new(0.505, 0.5), &spots, &sight), Some(1));
    assert_eq!(gate_under(Vec2::new(0.6, 0.5), &spots, &sight), None);
    // Gate 0 is on the edge, and with the globe turned a quarter it faces the
    // camera instead.
    let turned = Sight {
        globe: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
        ..sight
    };
    assert_eq!(gate_under(Vec2::splat(0.5), &spots, &turned), Some(0));
}

#[test]
fn gates_round_the_back_cant_be_picked() {
    let spots = [Vec2::new(0.25, 0.5)];
    let camera = camera(Vec3::Z, 6.0);
    assert_eq!(gate_under(Vec2::splat(0.5), &spots, &sight(&camera)), None);
}
