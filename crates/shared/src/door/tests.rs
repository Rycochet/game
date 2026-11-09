use super::*;
use crate::dungeon::{Cell, Level, Sight, line_is_clear};
use crate::poi::{ChunkPos, Poi, PoiKind};
use crate::rng::Seed;

/// A room 5 wide split by a wall running north-south at x = 0, with a door at
/// (0, 0): west of it x = -2..-1, east x = 1..2.
fn split_room() -> Level {
    let mut level = Level::new(-3, -3, 7, 7);
    for y in -2..=2 {
        for x in -2..=2 {
            level.set(x, y, if x == 0 { Cell::Wall } else { Cell::Floor });
        }
    }
    level.set(0, 0, Cell::Door);
    level
}

fn poi(kind: PoiKind, seed: u64) -> Poi {
    Poi {
        kind,
        chunk: ChunkPos { x: 0, y: 0 },
        x: 0,
        y: 0,
        seed: Seed::new(seed),
        desert: false,
    }
}

#[test]
fn a_door_with_no_swing_set_is_open_as_before() {
    let level = split_room();
    assert!(level.is_walkable(0, 0));
    assert!(line_is_clear(&level, (-1, 0), (1, 0)));
}

#[test]
fn a_closed_door_blocks_walking_sight_and_light() {
    let mut level = split_room();
    level.set_swing(0, 0, Swing::Closed);
    assert!(!level.is_walkable(0, 0));
    assert!(!level.is_open(0, 0));
    assert!(!line_is_clear(&level, (-1, 0), (1, 0)));
    // Not even past its edge, diagonally.
    assert!(!line_is_clear(&level, (-1, -1), (1, 1)));
    let mut sight = Sight::new(&level);
    sight.add_view(&level, -1, 0, None);
    assert!(sight.is_seen(0, 0), "the closed door itself is seen");
    assert!(!sight.is_seen(1, 0));
}

#[test]
fn an_open_door_lets_everything_through_again() {
    let mut level = split_room();
    level.set_swing(0, 0, Swing::Closed);
    level.set_swing(0, 0, Swing::Open(Side::East));
    assert!(level.is_walkable(0, 0));
    assert!(line_is_clear(&level, (-1, 0), (1, 0)));
    level.set_swing(0, 0, Swing::Broken(Side::West));
    assert!(level.is_walkable(0, 0));
}

#[test]
fn a_secret_door_is_a_wall_until_it_stands_open() {
    let mut level = split_room();
    level.set(0, 0, Cell::SecretDoor);
    assert!(level.is_wall(0, 0) && !level.is_walkable(0, 0));
    level.set_swing(0, 0, Swing::Open(Side::West));
    assert!(!level.is_wall(0, 0) && level.is_opening(0, 0));
    assert!(level.is_walkable(0, 0));
    assert!(line_is_clear(&level, (-1, 0), (1, 0)));
}

#[test]
fn routes_can_go_through_closed_doors_as_if_open() {
    let mut level = split_room();
    level.set_swing(0, 0, Swing::Closed);
    let open = level.with_doors_open();
    assert!(open.is_walkable(0, 0));
    assert!(!level.is_walkable(0, 0), "the level itself is unchanged");
}

#[test]
fn sides_are_away_from_and_towards_the_opener() {
    // A wall running north-south: someone to the west.
    let from = Side::of(false, (0, 0), (-1, 0));
    assert_eq!(from, Side::West);
    assert_eq!(from.opposite(), Side::East);
    assert!(from.fits(false) && !from.fits(true));
    // A wall running east-west: someone to the north, diagonally.
    assert_eq!(Side::of(true, (0, 0), (1, -1)), Side::North);
    assert_eq!(Side::North.beside((3, 4)), (3, 3));
}

#[test]
fn starting_doors_are_the_same_every_time_and_secret_ones_closed() {
    let poi = poi(PoiKind::StoneCottage, 7);
    for x in 0..50 {
        let a = starting_door(&poi, false, 0, (x, 3), false, false, true);
        let b = starting_door(&poi, false, 0, (x, 3), false, false, true);
        assert_eq!(a, b);
        assert_eq!(a.quality, Quality::Common);
        if let Swing::Open(side) | Swing::Broken(side) = a.swing {
            assert!(side.fits(true));
        }
        let secret = starting_door(&poi, false, 0, (x, 3), true, false, true);
        assert_eq!(secret.swing, Swing::Closed);
    }
}

/// How many of 1000 doors start closed, and how many broken.
fn tally(kind: PoiKind, ruined: bool, outside: bool) -> (u32, u32) {
    let poi = poi(kind, 11);
    let mut closed = 0;
    let mut broken = 0;
    for x in 0..1000 {
        match starting_door(&poi, ruined, 0, (x, 0), false, outside, false).swing {
            Swing::Closed => closed += 1,
            Swing::Broken(_) => broken += 1,
            Swing::Open(_) => {}
        }
    }
    (closed, broken)
}

#[test]
fn most_doors_start_closed_more_so_outside_and_at_herbalists() {
    let (inside, broken) = tally(PoiKind::StoneCottage, false, false);
    assert!((620..780).contains(&inside), "{inside}");
    assert_eq!(broken, 0);
    let (outside, _) = tally(PoiKind::StoneCottage, false, true);
    assert!(outside > inside + 100, "{outside}");
    let (herbalist, _) = tally(PoiKind::HerbalistHut, false, false);
    assert!(herbalist > inside + 50, "{herbalist}");
}

#[test]
fn ruins_have_broken_doors_and_fewer_closed() {
    let (closed, broken) = tally(PoiKind::StoneCottage, true, false);
    assert!((230..370).contains(&broken), "{broken}");
    assert!(closed < 400, "{closed}");
}

#[test]
fn a_door_leading_outside_is_known_by_the_ground_beside_it() {
    let mut level = split_room();
    assert!(!door_is_outside(&level, 0, 0));
    level.set(1, 0, Cell::Ground);
    assert!(door_is_outside(&level, 0, 0));
}
