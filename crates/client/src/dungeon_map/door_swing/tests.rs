use super::*;
use shared::door::{Side, Swing};
use shared::dungeon::Cell;

const KEY: Option<(u32, (i64, i64, i64, i64))> = Some((0, (-1, -1, 3, 3)));

/// A level with one door at (0, 0) in a wall running east-west.
fn level_with(swing: Swing) -> Level {
    let mut level = Level::new(-1, -1, 3, 3);
    level.set(0, -1, Cell::Floor);
    level.set(0, 1, Cell::Floor);
    level.set(0, 0, Cell::Door);
    level.set_swing(0, 0, swing);
    level
}

#[test]
fn a_door_first_seen_is_drawn_as_it_hangs() {
    let mut swings = DoorSwings::default();
    swings.update((KEY, &level_with(Swing::Closed), true), 0.0, false);
    assert!(swings.drawn.is_empty());
}

#[test]
fn a_door_opening_swings_there_and_stops() {
    let mut swings = DoorSwings::default();
    swings.update((KEY, &level_with(Swing::Closed), true), 0.0, false);
    let open = level_with(Swing::Open(Side::South));
    assert!(swings.update((KEY, &open, true), 1.0, false));
    let half = SWING_SECONDS / 2.0;
    assert!(swings.update((KEY, &open, false), 1.0 + half, false));
    let pose = swings.drawn[&(0, 0)];
    let end = LeafPose::of(&open, (0, 0));
    assert!(pose.turned > 0.0 && pose.turned < end.turned, "{pose:?}");
    // Eased: more than half way at half the time.
    assert!(pose.turned > end.turned / 2.0);
    assert!(swings.update((KEY, &open, false), 1.0 + SWING_SECONDS, false));
    assert!(swings.drawn.is_empty());
    assert!(
        !swings.update((KEY, &open, false), 2.0, false),
        "nothing moving"
    );
}

#[test]
fn reduce_motion_and_a_new_level_draw_it_at_once() {
    let mut swings = DoorSwings::default();
    swings.update((KEY, &level_with(Swing::Closed), true), 0.0, true);
    swings.update(
        (KEY, &level_with(Swing::Open(Side::North)), true),
        1.0,
        true,
    );
    assert!(swings.drawn.is_empty());
    let mut swings = DoorSwings::default();
    swings.update((KEY, &level_with(Swing::Closed), true), 0.0, false);
    let other = Some((1, (-1, -1, 3, 3)));
    swings.update(
        (other, &level_with(Swing::Open(Side::North)), true),
        1.0,
        false,
    );
    assert!(swings.drawn.is_empty());
}

#[test]
fn a_leaf_turns_through_shut_from_one_side_to_the_other() {
    let north = LeafPose {
        turned: 1.0,
        way: -1.0,
        tilt: 0.0,
    };
    let south = LeafPose { way: 1.0, ..north };
    let middle = LeafPose::between(north, south, 0.5);
    assert!(middle.turned.abs() < 1e-6);
    let later = LeafPose::between(north, south, 0.75);
    assert_eq!(later.way, 1.0);
    let broken = LeafPose { tilt: 1.0, ..south };
    assert_eq!(LeafPose::between(south, broken, 0.5).tilt, 0.5);
}
