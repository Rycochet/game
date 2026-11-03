use super::*;

/// A level of floor with holes where the picture has `O`, offset so that
/// no cell in it is on the level's edge.
fn level_of(rows: &[&str]) -> Level {
    let mut level = Level::new(-1, -1, 12, 12);
    for y in -1..11 {
        for x in -1..11 {
            level.set(x, y, Cell::Floor);
        }
    }
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if c == 'O' {
                level.set(x as i64, y as i64, Cell::Hole);
            }
        }
    }
    level
}

/// Every point of a level's holes' edges that lies on the line between
/// two cells, in level units, rounded to compare.
fn crossings(level: &Level) -> Vec<(i64, i64)> {
    let mut found = Vec::new();
    for y in 0..10 {
        for x in 0..10 {
            if level.cell(x, y) != Cell::Hole {
                continue;
            }
            for (sx, sy) in QUARTERS {
                let Some(remains) = remains(level, (x, y), (sx, sy)) else {
                    continue;
                };
                for &(east, south) in &remains.edge {
                    if east == HALF_CELL || south == HALF_CELL {
                        let px = x as f32 * CELL_SIZE + sx as f32 * east;
                        let pz = y as f32 * CELL_SIZE + sy as f32 * south;
                        found.push(((px * 1000.0).round() as i64, (pz * 1000.0).round() as i64));
                    }
                }
            }
        }
    }
    found.sort();
    found
}

/// Where a hole's edge leaves a cell, the neighbouring hole cell's edge
/// starts at exactly that point, in any arrangement of cells.
#[test]
fn touching_holes_line_up() {
    let level = level_of(&[
        "..........",
        ".OOO..O...",
        ".O.O.OO...",
        ".OOO..O...",
        "...O......",
        "..OOOOO...",
        "..O.O.....",
        "...O.O....",
    ]);
    let found = crossings(&level);
    assert!(!found.is_empty());
    // Each point comes in a pair: one from each cell sharing the edge.
    for pair in found.chunks(2) {
        assert_eq!(pair.len(), 2, "{found:?}");
        assert_eq!(pair[0], pair[1], "{found:?}");
    }
}

/// All-hole quarters have no floor; a lone hole keeps floor in every
/// quarter, within the cell, round a dark middle.
#[test]
fn a_quarter_has_floor_only_beside_floor() {
    let lone = level_of(&["...", ".O.", "..."]);
    for quarter in QUARTERS {
        let remains = remains(&lone, (1, 1), quarter).unwrap();
        assert!(remains.floor.len() >= 4);
        for &(east, south) in &remains.floor {
            assert!((0.0..=HALF_CELL).contains(&east));
            assert!((0.0..=HALF_CELL).contains(&south));
            assert!(east > 0.0 || south > 0.0);
        }
    }
    let open = level_of(&["OOO", "OOO", "OOO"]);
    assert!(remains(&open, (1, 1), (1, 1)).is_none());
}

/// A hole on the level's edge, or in its corner, has no floor along the
/// edge: only where it meets floor inside the level.
#[test]
fn nothing_is_left_along_the_levels_edge() {
    let mut level = Level::new(0, 0, 3, 3);
    for y in 0..3 {
        for x in 0..3 {
            level.set(x, y, Cell::Hole);
        }
    }
    for quarter in QUARTERS {
        assert!(remains(&level, (0, 0), quarter).is_none(), "{quarter:?}");
        assert!(remains(&level, (1, 0), quarter).is_none(), "{quarter:?}");
    }
    level.set(1, 1, Cell::Floor);
    // Towards the floor, a bite of it; off the edge, still nothing.
    assert!(remains(&level, (0, 0), (1, 1)).is_some());
    assert!(remains(&level, (0, 0), (-1, -1)).is_none());
}

/// In every arrangement of the three cells round a quarter, the floor's
/// outline stays a simple shape seen whole from the grid's corner, with
/// no area lost (the angle round the corner only ever goes one way).
#[test]
fn every_arrangement_has_a_good_outline() {
    for mask in 0..8 {
        let mut level = level_of(&["...", ".O.", "..."]);
        for (bit, (x, y)) in [(2, 1), (1, 2), (2, 2)].into_iter().enumerate() {
            if mask & (1 << bit) != 0 {
                level.set(x, y, Cell::Hole);
            }
        }
        let Some(remains) = remains(&level, (1, 1), (1, 1)) else {
            assert_eq!(mask, 7);
            continue;
        };
        let angle = |&(east, south): &Point| (HALF_CELL - south).atan2(HALF_CELL - east);
        let angles: Vec<f32> = remains.floor[1..].iter().map(angle).collect();
        let up = angles.windows(2).all(|p| p[1] >= p[0] - 1e-4);
        let down = angles.windows(2).all(|p| p[1] <= p[0] + 1e-4);
        assert!(up || down, "mask {mask}: {:?}", remains.floor);
    }
}
