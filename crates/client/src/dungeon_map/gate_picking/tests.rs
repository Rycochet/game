use super::*;
use crate::dungeon_map::ground_hit::Ground;

fn setup() -> (Level, Sight) {
    let mut level = Level::test_room();
    level.set(2, 2, Cell::WorldGate);
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 0, 0, None);
    (level, sight)
}

fn ray(origin: Vec3, towards: Vec3) -> Ray3d {
    Ray3d::new(origin, Dir3::new(towards - origin).unwrap())
}

/// A camera south of the gate and above, looking north and down.
fn camera() -> Vec3 {
    Vec3::new(2.0 * CELL_SIZE, 300.0, 2.0 * CELL_SIZE + 300.0)
}

#[test]
fn a_click_on_the_frame_means_the_gate() {
    let (level, sight) = setup();
    // Partway up the frame, where the floor reached is the cell behind it.
    let up_the_frame = Vec3::new(2.0 * CELL_SIZE, CELL_SIZE, 2.0 * CELL_SIZE);
    let through = ray(camera(), up_the_frame);
    assert_eq!(
        gate_clicked(through, &level, &sight, &[], Ground::FLAT),
        Some((2, 2)),
        "the gate"
    );
    assert_ne!(
        crate::dungeon_map::picking::pick(through, &level, &sight, &[], Ground::FLAT),
        Some((2, 2)),
        "the floor alone is behind it"
    );
}

#[test]
fn clicks_beside_or_above_it_are_not_the_gate() {
    let (level, sight) = setup();
    let beside = Vec3::new(3.0 * CELL_SIZE, CELL_SIZE, 2.0 * CELL_SIZE);
    assert_eq!(
        gate_clicked(ray(camera(), beside), &level, &sight, &[], Ground::FLAT),
        None
    );
    let above = Vec3::new(2.0 * CELL_SIZE, 2.0 * CELL_SIZE, 2.0 * CELL_SIZE);
    assert_eq!(
        gate_clicked(ray(camera(), above), &level, &sight, &[], Ground::FLAT),
        None
    );
}

#[test]
fn an_unseen_gate_is_not_clicked() {
    let (level, _) = setup();
    let sight = Sight::new(&level);
    let up_the_frame = Vec3::new(2.0 * CELL_SIZE, CELL_SIZE, 2.0 * CELL_SIZE);
    assert_eq!(
        gate_clicked(
            ray(camera(), up_the_frame),
            &level,
            &sight,
            &[],
            Ground::FLAT
        ),
        None
    );
}
