use shared::dungeon::Cell;
use shared::dungeon::{Level, RoomId};
use shared::poi::PoiKind;
use shared::protocol::LevelInfo;
use shared::site::BuildingKind;
use shared::site::{BuildingLook, Rect, RoofKind, WallKind};

use super::*;
use crate::common::CurrentDungeon;

/// White between the lines; beyond them, what is known to be there.
#[test]
fn a_window_shows_what_it_looks_onto() {
    let mut dungeon = dungeon();
    dungeon.level.set(2, 1, Cell::Window);
    dungeon.level.set(2, 0, Cell::Ground);
    let look = look_of(&dungeon, 2, 1);
    assert_eq!(
        look,
        CellLook::Window {
            east_west: true,
            sides: [ground_colour(Cell::Ground), WHITE],
        }
    );
    let at = |v: f64| colour_at(look, 0.0, v, 0.0);
    assert_eq!(at(0.0), Some(WHITE));
    assert_eq!(at(-DOOR_HALF_WIDTH + 0.01), Some(HATCH));
    assert_eq!(at(DOOR_HALF_WIDTH - 0.01), Some(HATCH));
    assert_eq!(at(-0.4), Some(ground_colour(Cell::Ground)));
    assert_eq!(at(0.4), Some(WHITE));
}

/// A 5 × 5 level, all remembered: wall but for a floor in the middle.
fn dungeon() -> CurrentDungeon {
    let mut dungeon = CurrentDungeon {
        level: Level::new(0, 0, 5, 5),
        ..Default::default()
    };
    dungeon.enter(LevelInfo {
        kind: PoiKind::Crypt,
        ruined: false,
        building: false,
        index: 0,
        levels: 1,
        basement: false,
        bounds: (0, 0, 5, 5),
        arrival: (2, 2),
        entering_site: true,
    });
    for y in 0..5 {
        for x in 0..5 {
            dungeon.remembered.insert(x, y);
        }
    }
    dungeon.level.set(2, 2, Cell::Floor);
    dungeon
}

fn wall_sides(dungeon: &CurrentDungeon, x: i64, y: i64) -> Sides {
    let CellLook::Wall(sides) = look_of(dungeon, x, y) else {
        panic!("not a wall");
    };
    sides
}

#[test]
fn walls_are_hatched_towards_remembered_floor_only() {
    let mut dungeon = dungeon();
    assert_eq!(look_of(&dungeon, 2, 2), CellLook::Floor(WHITE));
    let south_only = Sides {
        south: true,
        ..Default::default()
    };
    assert_eq!(wall_sides(&dungeon, 2, 1), south_only);
    assert_eq!(wall_sides(&dungeon, 0, 0), Sides::default());

    dungeon.remembered = shared::dungeon::Sight::new(&dungeon.level);
    dungeon.remembered.insert(2, 1);
    assert_eq!(wall_sides(&dungeon, 2, 1), Sides::default());
    // Anything not remembered is bare parchment.
    assert_eq!(look_of(&dungeon, 0, 0), CellLook::Empty);
}

/// Walls meeting only at a corner, with a way between them, each lose that
/// corner; walls with no way between them keep theirs.
#[test]
fn corners_are_cut_where_a_diagonal_goes_between_walls() {
    let mut dungeon = dungeon();
    dungeon.level.set(2, 2, Cell::Wall);
    dungeon.level.set(2, 1, Cell::Floor);
    dungeon.level.set(1, 2, Cell::Floor);
    let look = look_of(&dungeon, 2, 2);
    let cut = wall_sides(&dungeon, 2, 2).cut;
    assert_eq!(cut, [None, None, None, Some(WHITE)]);
    assert_eq!(wall_sides(&dungeon, 1, 1).cut[1], Some(WHITE));
    // The corner shows floor; the rest of the wall doesn't.
    assert_eq!(colour_at(look, -0.45, -0.45, 0.05), Some(WHITE));
    assert_ne!(colour_at(look, 0.45, 0.45, 0.05), Some(WHITE));
    dungeon.level.set(1, 2, Cell::Wall);
    assert_eq!(wall_sides(&dungeon, 2, 2).cut, [None; 4]);
}

#[test]
fn touching_buildings_are_hatched_on_both_sides() {
    let mut dungeon = dungeon();
    dungeon.level.set(2, 2, Cell::Wall);
    dungeon.level.set_room(1, 1, Some(RoomId(1)));
    dungeon.level.set_room(2, 1, Some(RoomId(1)));
    dungeon.level.set_room(3, 1, Some(RoomId(2)));
    let east = Sides {
        east: true,
        ..Default::default()
    };
    let west = Sides {
        west: true,
        ..Default::default()
    };
    assert_eq!(wall_sides(&dungeon, 2, 1), east);
    assert_eq!(wall_sides(&dungeon, 3, 1), west);
    // Within a building, or walls in no building, nothing.
    assert_eq!(wall_sides(&dungeon, 1, 1), Sides::default());
    assert_eq!(wall_sides(&dungeon, 1, 3), Sides::default());
}

#[test]
fn shops_show_a_sign_and_houses_a_door_across_their_wall() {
    let mut dungeon = dungeon();
    dungeon.level.set(2, 2, Cell::BuildingDoor);
    let building = |kind, faces| BuildingLook {
        id: 0,
        kind,
        walls: WallKind::Stone,
        roof: RoofKind::SlateTiles,
        rects: vec![Rect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        }],
        door: (2, 2),
        faces,
    };
    dungeon.buildings = vec![building(BuildingKind::Bank, (0, 1))];
    assert!(matches!(
        look_of(&dungeon, 2, 2),
        CellLook::Sign(BuildingKind::Bank)
    ));
    dungeon.buildings = vec![building(BuildingKind::House, (1, 0))];
    assert_eq!(
        look_of(&dungeon, 2, 2),
        CellLook::HouseDoor { east_west: false }
    );
    // Nothing hatches towards a building's door, from its wall or across
    // the street.
    assert!(!wall_sides(&dungeon, 1, 2).east);
    dungeon.level.set(2, 3, Cell::Floor);
    assert_eq!(look_of(&dungeon, 2, 3), CellLook::Floor(WHITE));
}

#[test]
fn colours_follow_the_owners_drawing() {
    let hatched_north = CellLook::Wall(Sides {
        north: true,
        ..Default::default()
    });
    assert_eq!(colour_at(CellLook::Empty, 0.0, 0.0, 0.05), None);
    assert_eq!(
        colour_at(CellLook::Floor(WHITE), 0.0, 0.0, 0.05),
        Some(WHITE)
    );
    assert_eq!(
        colour_at(CellLook::Floor(WHITE), 0.48, 0.0, 0.05),
        Some(LINE)
    );
    // A wall is parchment, with hatching in the quarter by its edge, and
    // some gaps in it.
    assert_eq!(colour_at(hatched_north, 0.0, 0.0, 0.05), None);
    let band: Vec<_> = (0..20)
        .map(|i| colour_at(hatched_north, i as f64 / 20.0 - 0.5, -0.4, 0.0))
        .collect();
    assert!(band.contains(&Some(HATCH)) && band.contains(&None));
    // A door runs along its wall.
    let door = CellLook::Door { east_west: true };
    assert_eq!(colour_at(door, 0.0, 0.0, 0.05), Some(DOOR));
    assert_eq!(colour_at(door, 0.4, 0.0, 0.05), Some(HATCH));
    assert_eq!(colour_at(door, 0.0, 0.4, 0.05), Some(WHITE));
}

#[test]
fn walls_beside_a_dungeon_door_are_hatched_towards_it() {
    let mut dungeon = dungeon();
    dungeon.level.set(2, 2, Cell::Door);
    dungeon.level.set(2, 1, Cell::Floor);
    dungeon.level.set(2, 3, Cell::Floor);
    assert_eq!(look_of(&dungeon, 2, 2), CellLook::Door { east_west: true });
    assert!(wall_sides(&dungeon, 1, 2).east);
    assert!(wall_sides(&dungeon, 3, 2).west);
}

#[test]
fn tiles_take_the_colour_of_their_ground() {
    let mut dungeon = dungeon();
    dungeon.level.set(2, 2, Cell::Water);
    let CellLook::Floor(water) = look_of(&dungeon, 2, 2) else {
        panic!("water should show");
    };
    assert!(water[2] > water[0]);
    dungeon.level.set(2, 2, Cell::Lava);
    let CellLook::Floor(lava) = look_of(&dungeon, 2, 2) else {
        panic!("lava should show");
    };
    assert!(lava[0] > lava[2]);
    dungeon.level.set(2, 2, Cell::Hole);
    assert_eq!(look_of(&dungeon, 2, 2), CellLook::Floor(HOLE));
    dungeon.level.set(2, 2, Cell::Cobblestone);
    assert!(matches!(look_of(&dungeon, 2, 2), CellLook::Floor(c) if c != WHITE));
}

#[test]
fn looks_list_their_signs() {
    let mut dungeon = dungeon();
    dungeon.level.set(2, 2, Cell::BuildingDoor);
    dungeon.buildings = vec![BuildingLook {
        id: 0,
        kind: BuildingKind::Inn,
        walls: WallKind::Stone,
        roof: RoofKind::SlateTiles,
        rects: Vec::new(),
        door: (2, 2),
        faces: (0, 1),
    }];
    let looks = Looks::of(&dungeon, (1, 1), (3, 3));
    let signs: Vec<_> = looks.signs().collect();
    assert_eq!(signs, [((2, 2), BuildingKind::Inn)]);
    assert_eq!(looks.at(100, 100), CellLook::Empty);
}

#[test]
fn a_secret_door_is_drawn_as_a_door_and_listed_for_its_s() {
    let mut dungeon = dungeon();
    dungeon.level.set(2, 2, Cell::SecretDoor);
    dungeon.level.set(2, 1, Cell::Floor);
    dungeon.level.set(2, 3, Cell::Floor);
    let secret = CellLook::SecretDoor { east_west: true };
    assert_eq!(look_of(&dungeon, 2, 2), secret);
    let door = CellLook::Door { east_west: true };
    for (u, v) in [(0.0, 0.0), (0.25, 0.1), (0.45, 0.0), (0.0, 0.4)] {
        assert_eq!(colour_at(secret, u, v, 0.05), colour_at(door, u, v, 0.05));
    }
    let looks = Looks::of(&dungeon, (1, 1), (3, 3));
    assert_eq!(looks.secret_doors().collect::<Vec<_>>(), [(2, 2)]);

    dungeon.level.set(2, 2, Cell::Door);
    let looks = Looks::of(&dungeon, (1, 1), (3, 3));
    assert_eq!(looks.secret_doors().count(), 0);
}
