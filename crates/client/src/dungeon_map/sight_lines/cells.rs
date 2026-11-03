//! The cells an outline reaches into or crosses, counted as seen.

use shared::dungeon::{Level, Sight};

use super::Vec2d;

/// Counts every open cell an outline reaches into as seen, row by row: the part
/// of each triangle of its fan in a row is a span of cells.
pub(super) fn touch_cells(level: &Level, from: Vec2d, outline: &[Vec2d], touched: &mut Sight) {
    for i in 0..outline.len() {
        let triangle = [from, outline[i], outline[(i + 1) % outline.len()]];
        let lowest = triangle.iter().map(|p| p.y).fold(f64::MAX, f64::min);
        let highest = triangle.iter().map(|p| p.y).fold(f64::MIN, f64::max);
        for row in (lowest + 0.5).floor() as i64..=(highest + 0.5).floor() as i64 {
            let Some((left, right)) = span_in_row(&triangle, row) else {
                continue;
            };
            for x in (left + 0.5).floor() as i64..=(right + 0.5).floor() as i64 {
                if level.is_open(x, row) && !touched.is_seen(x, row) {
                    touched.reveal(level, x, row);
                }
            }
        }
    }
}

/// Counts the cells in `from` as seen in `into` too.
pub fn add_cells(level: &Level, from: &Sight, into: &mut Sight) {
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if from.is_seen(x, y) {
                into.insert(x, y);
            }
        }
    }
}

/// Counts the cells an outline's edge goes through, and those beside them.
pub(super) fn cross_cells(outline: &[Vec2d], crossed: &mut Sight) {
    for i in 0..outline.len() {
        let (a, b) = (outline[i], outline[(i + 1) % outline.len()]);
        let edge = [a, b, b];
        for row in (a.y.min(b.y) + 0.5).floor() as i64..=(a.y.max(b.y) + 0.5).floor() as i64 {
            let Some((left, right)) = span_in_row(&edge, row) else {
                continue;
            };
            for x in (left + 0.5).floor() as i64 - 1..=(right + 0.5).floor() as i64 + 1 {
                for y in row - 1..=row + 1 {
                    crossed.insert(x, y);
                }
            }
        }
    }
}

/// The smallest and largest `x` of a triangle within a row of cells.
pub(super) fn span_in_row(triangle: &[Vec2d; 3], row: i64) -> Option<(f64, f64)> {
    let (top, bottom) = (row as f64 - 0.5, row as f64 + 0.5);
    let mut span: Option<(f64, f64)> = None;
    let mut add = |x: f64| {
        span = Some(span.map_or((x, x), |(a, b)| (a.min(x), b.max(x))));
    };
    for i in 0..3 {
        let (a, b) = (triangle[i], triangle[(i + 1) % 3]);
        if (top..=bottom).contains(&a.y) {
            add(a.x);
        }
        for y in [top, bottom] {
            if (a.y < y) != (b.y < y) {
                add(a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y));
            }
        }
    }
    span
}
