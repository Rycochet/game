use shared::dungeon::{Cell, Level};

use super::outline::Outline;
use super::*;

/// How far from the middle of a cell the liquid reaches along the edge of a
/// quarter: its east-west edge, or its north-south one.
fn reach_on_side(outline: Option<Outline>, east_side: bool) -> f32 {
    match (outline, east_side) {
        (Some(Outline::Full), _) => HALF_CELL,
        (Some(Outline::Notch), _) => CHANNEL_HALF,
        (Some(Outline::AlongEast), true) | (Some(Outline::AlongSouth), false) => CHANNEL_HALF,
        _ => 0.0,
    }
}

/// Whatever the four cells round a grid corner are, the quarters touching
/// it agree where the liquid crosses from one cell to the next.
#[test]
fn quarters_meet_their_neighbours() {
    const KINDS: [Cell; 3] = [Cell::Floor, Cell::Wall, Cell::Water];
    for &nw in &KINDS {
        for &ne in &KINDS {
            for &sw in &KINDS {
                for &se in &KINDS {
                    let mut level = Level::new(0, 0, 2, 2);
                    level.set(0, 0, nw);
                    level.set(1, 0, ne);
                    level.set(0, 1, sw);
                    level.set(1, 1, se);
                    let cases = (nw, ne, sw, se);
                    let at = |cell, quarter| outline_at(&level, cell, quarter);
                    let [a, b, c, d] = [
                        at((0, 0), (1.0, 1.0)),
                        at((1, 0), (-1.0, 1.0)),
                        at((0, 1), (1.0, -1.0)),
                        at((1, 1), (-1.0, -1.0)),
                    ];
                    let side = reach_on_side;
                    assert_eq!(side(a, true), side(b, true), "north edge: {cases:?}");
                    assert_eq!(side(c, true), side(d, true), "south edge: {cases:?}");
                    assert_eq!(side(a, false), side(c, false), "west edge: {cases:?}");
                    assert_eq!(side(b, false), side(d, false), "east edge: {cases:?}");
                }
            }
        }
    }
}

fn pool(cells: &[(i64, i64)]) -> Level {
    let mut level = Level::new(-4, -4, 9, 9);
    for y in -4..=4 {
        for x in -4..=4 {
            level.set(x, y, Cell::Floor);
        }
    }
    for &(x, y) in cells {
        level.set(x, y, Cell::Water);
    }
    level
}

#[test]
fn a_channel_is_narrow_and_curves_round_its_corner() {
    const NE: (f32, f32) = (1.0, -1.0);
    const SW: (f32, f32) = (-1.0, 1.0);
    // Running east-west.
    let level = pool(&[(-1, 0), (0, 0), (1, 0)]);
    assert!(is_narrow(&level, 0, 0));
    assert_eq!(outline_at(&level, (0, 0), NE), Some(Outline::AlongEast));
    // Coming from the north and turning east, with the outside of the
    // bend reaching past the middle of the cell.
    let level = pool(&[(0, -1), (0, 0), (1, 0)]);
    assert!(!is_narrow(&level, 0, 0));
    assert_eq!(outline_at(&level, (0, 0), NE), Some(Outline::Notch));
    assert_eq!(outline_at(&level, (0, 0), SW), Some(Outline::Round));
    // Floor isn't liquid.
    assert_eq!(outline_at(&level, (2, 0), NE), None);
}

#[test]
fn narrow_channels_are_flat_until_they_meet_something_deeper() {
    let at_the_seam = (HALF_CELL, 0.0);
    let depth_at_east_seam = |level: &Level, cell| {
        let quarter = (1.0, -1.0);
        let outline = outline_at(level, cell, quarter).unwrap();
        bed_at(level, cell, quarter, outline).depth(at_the_seam)
    };
    // Along a channel it stays half deep.
    let mut level = pool(&[(-1, 0), (0, 0), (1, 0)]);
    assert_eq!(depth_at_east_seam(&level, (0, 0)), NARROW_DEPTH);
    // Running into a wall it reaches full depth there...
    level.set(1, 0, Cell::Wall);
    assert_eq!(depth_at_east_seam(&level, (0, 0)), FULL_DEPTH);
    // ...and into something wider too: (1, 0) has liquid north and east.
    let level = pool(&[(-1, 0), (0, 0), (1, 0), (1, -1), (2, 0)]);
    assert_eq!(depth_at_east_seam(&level, (0, 0)), FULL_DEPTH);
}

#[test]
fn liquid_beside_a_wall_runs_to_its_foot() {
    // A wall along the north of a row of water.
    let mut level = pool(&[(-1, 1), (0, 1), (1, 1)]);
    for x in -2..=2 {
        level.set(x, 0, Cell::Wall);
    }
    let south_east = (1.0, 1.0);
    assert_eq!(
        outline_at(&level, (0, 0), south_east),
        Some(Outline::Full),
        "all along it"
    );
    // Beside the water's end it is only as wide as the channel, and past
    // it there is none.
    assert_eq!(
        outline_at(&level, (1, 0), south_east),
        Some(Outline::AlongSouth)
    );
    assert_eq!(outline_at(&level, (2, 0), south_east), None);
    // And its north side stays dry.
    assert_eq!(outline_at(&level, (0, 0), (1.0, -1.0)), None);
}

#[test]
fn deeper_is_darker_and_the_bed_darker_than_the_sides() {
    let close = |a: f32, b: f32| (a - b).abs() < 0.001;
    assert!(close(depth_shade(NARROW_DEPTH), 0.6));
    assert!(close(depth_shade(FULL_DEPTH), 0.4));
    assert!(close(bed_shade(NARROW_DEPTH), 0.4));
    assert!(close(bed_shade(FULL_DEPTH), 0.2));
}

#[test]
fn a_channel_runs_on_behind_its_centre_line() {
    let quarter = (-1.0, 1.0);
    assert_eq!(Outline::AlongEast.behind(quarter), Some((1.0, 1.0)));
    assert_eq!(Outline::AlongSouth.behind(quarter), Some((-1.0, -1.0)));
    assert_eq!(Outline::Full.behind(quarter), None);
}

#[test]
fn a_wall_goes_down_only_where_liquid_touches_it() {
    let span = |outline: Outline| outline.span_along(true, 1.0, (0.0, HALF_CELL));
    assert_eq!(span(Outline::Full), Some((0.0, HALF_CELL)));
    assert_eq!(span(Outline::AlongSouth), Some((0.0, CHANNEL_HALF)));
    assert_eq!(span(Outline::Round), None);
}
