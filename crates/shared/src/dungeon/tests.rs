use super::*;

/// A hole is seen and lit across, but nothing else.
#[test]
fn holes_are_open_but_not_walkable() {
    let hole = Cell::Hole;
    assert!(hole.is_open());
    assert!(!hole.is_walkable() && !hole.is_outside() && !hole.is_liquid());
    assert!(Cell::CoveredGround.is_walkable() && !Cell::CoveredGround.is_outside());
    assert!(!hole.is_wall() && !hole.is_opening());
    assert!(hole.is_jumpable());
}

/// The room as a picture, `#` wall, `.` floor.
fn picture(level: &Level, mark: impl Fn(i64, i64) -> char) -> Vec<String> {
    let (min_x, min_y, width, height) = level.bounds();
    (min_y..min_y + height)
        .map(|y| (min_x..min_x + width).map(|x| mark(x, y)).collect())
        .collect()
}

#[test]
fn test_room_layout() {
    let level = Level::test_room();
    let rows = picture(&level, |x, y| if level.is_open(x, y) { '.' } else { '#' });
    assert_eq!(
        rows,
        [
            "#################",
            "#...............#",
            "#...............#",
            "#.....#.........#",
            "#.....#.........#",
            "#.....#.........#",
            "#...............#",
            "#...............#",
            "#################",
        ]
    );
    // Outside the level is wall.
    assert!(!level.is_open(100, 0));
}

/// Golden test: what the player sees from the middle of the test room.
/// The short wall two steps west hides most of the room's west end.
#[test]
fn sight_from_the_middle_of_the_test_room() {
    let level = Level::test_room();
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 0, 0, None);
    let rows = picture(&level, |x, y| {
        match (sight.is_seen(x, y), level.is_open(x, y)) {
            (true, true) => '.',
            (true, false) => '#',
            (false, _) => ' ',
        }
    });
    assert_eq!(
        rows,
        [
            "  ###############",
            "   .............#",
            "     ...........#",
            "      #.........#",
            "      #.........#",
            "      #.........#",
            "     ...........#",
            "   .............#",
            "  ###############",
        ]
    );
}

/// A wall's end hides a far cell's centre, but one of its corners shows.
#[test]
fn glimpses_show_parts_of_cells() {
    let mut level = Level::test_room();
    level.set(4, -1, Cell::Wall);
    level.set(4, 0, Cell::Wall);
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 2, 0, None);
    assert!(!sight.is_seen(6, 0));
    sight.add_glimpses(&level, 2, 0, None);
    assert!(sight.is_seen(6, 0));
}

/// A floor with walls placed by a picture: `#` wall, `+` door, `W` window.
fn drawn(rows: &[&str]) -> Level {
    let mut level = Level::new(0, 0, rows[0].len() as i64, rows.len() as i64);
    for (y, row) in rows.iter().enumerate() {
        for (x, mark) in row.chars().enumerate() {
            let cell = match mark {
                '#' => Cell::Wall,
                '+' => Cell::Door,
                'W' => Cell::Window,
                _ => Cell::Floor,
            };
            level.set(x as i64, y as i64, cell);
        }
    }
    level
}

#[test]
fn walls_take_their_shape_from_their_neighbours() {
    let level = drawn(&[
        ".....", //
        ".#...", //
        ".....", //
        ".###.", //
        "...#.", //
        ".###.", //
        "...#.", //
        ".#+#.", //
    ]);
    let shape = |x, y| level.wall_shape(x, y).unwrap();
    let reach = |x, y| shape(x, y).reach();
    // A column.
    assert_eq!(reach(1, 1), [2, 2, 2, 2]);
    // The end of a wall stops 60% of the way across.
    assert_eq!(reach(1, 3), [2, 20, 2, 4]);
    // Straight on, and a corner.
    assert_eq!(reach(2, 3), [2, 20, 2, 20]);
    assert_eq!(reach(3, 3), [2, 2, 20, 20]);
    // A T.
    assert_eq!(reach(3, 5), [20, 2, 20, 20]);
    // Walls join a door in line with them.
    assert!(shape(1, 7).east && shape(3, 7).west);
    // Not one they stand beside.
    assert!(level.opening_runs_east_west(2, 7));
    assert!(!shape(2, 5).south);
    assert_eq!(level.wall_shape(2, 7), None);
}

#[test]
fn walls_block_only_where_they_stand() {
    let mut level = Level::test_room();
    // Beside a column, along a line that crosses its cell off centre.
    level.set(2, 0, Cell::Wall);
    assert!(clear_line(&level, (0, -10), (160, 30)));
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 0, 0, None);
    // Straight through it is hidden, the cells either side are not.
    assert!(!sight.is_seen(4, 0));
    assert!(sight.is_seen(4, 1));
    // Seen from beside it, past its cell's corner.
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 1, 1, None);
    assert!(sight.is_seen(3, 0));
    assert!(sight.is_seen(2, 0));
}

#[test]
fn doors_and_windows_let_sight_through_but_walls_do_not() {
    let level = drawn(&[
        "...", //
        "#W#", //
        "...", //
        "#+#", //
        "...", //
        "###", //
        "...", //
    ]);
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 1, 2, None);
    assert!(sight.is_seen(1, 0));
    assert!(sight.is_seen(1, 4));
    assert!(!sight.is_seen(1, 6));
    // Windows can't be walked through; both can be seen across.
    assert!(!level.is_walkable(1, 1) && level.is_open(1, 1));
    assert!(level.is_walkable(1, 3) && level.is_open(1, 3));
    // Walls touching what is seen are seen.
    assert!(sight.is_seen(1, 5));
}

/// Sight is the same both ways, also past thin walls.
#[test]
fn seeing_is_mutual() {
    let level = drawn(&[
        "........", //
        ".#..##..", //
        "....#...", //
        "..#.....", //
        ".....W#.", //
        ".##+##..", //
        "........", //
    ]);
    let (_, _, width, height) = level.bounds();
    let view = |x, y| {
        let mut sight = Sight::new(&level);
        sight.add_view(&level, x, y, None);
        sight
    };
    for y in 0..height {
        for x in 0..width {
            if !level.is_open(x, y) {
                continue;
            }
            let sight = view(x, y);
            for oy in 0..height {
                for ox in 0..width {
                    if level.is_open(ox, oy) {
                        let back = view(ox, oy).is_seen(x, y);
                        assert_eq!(sight.is_seen(ox, oy), back, "{x},{y} {ox},{oy}");
                    }
                }
            }
        }
    }
}

#[test]
fn radius_limits_sight() {
    let level = Level::test_room();
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 0, 0, Some(2));
    assert!(sight.is_seen(2, 0));
    assert!(sight.is_seen(1, 1));
    assert!(!sight.is_seen(2, 1));
    assert!(!sight.is_seen(3, 0));
}

#[test]
fn views_combine() {
    let level = Level::test_room();
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 0, 0, None);
    assert!(!sight.is_seen(-4, 0));
    sight.add_view(&level, -6, 0, None);
    assert!(sight.is_seen(-4, 0));
    assert!(sight.is_seen(5, 0));
}

#[test]
fn nothing_is_seen_from_inside_a_wall() {
    let level = Level::test_room();
    let mut sight = Sight::new(&level);
    sight.add_view(&level, -2, 0, None);
    assert!(!sight.is_seen(-1, 0));
}
