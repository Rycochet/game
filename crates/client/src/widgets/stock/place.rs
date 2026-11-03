//! Where things go in a stock grid: the first place they fit, or where they were
//! let go.

use shared::creature::Size;
use shared::item::{ItemKind, Placed, Shape};

/// The longest side of any kind of item at its largest: what the empty rows must
/// have room for, however it is turned.
pub(super) fn longest_side() -> u8 {
    let shapes = ItemKind::ALL
        .iter()
        .map(|kind| kind.def().shape.scaled(Size::Giant));
    shapes
        .flat_map(|shape| [shape.width, shape.height])
        .max()
        .unwrap_or(1)
}

/// How many rows a grid has: all it holds, and room for the largest thing below.
pub(super) fn rows_for(stock: &[Placed]) -> u8 {
    let cells = stock.iter().flat_map(|placed| placed.cells());
    let lowest = cells.map(|(_, y)| y + 1).max().unwrap_or(0);
    lowest + longest_side()
}

/// Whether a shape's cells are all free with its corner here.
fn fits(columns: u8, stock: &[Placed], shape: Shape, x: i32, y: i32) -> bool {
    let taken: Vec<(u8, u8)> = stock.iter().flat_map(|placed| placed.cells()).collect();
    x >= 0
        && y >= 0
        && shape.cells().all(|(cell_x, cell_y)| {
            let cell = (x + cell_x as i32, y + cell_y as i32);
            cell.0 < columns as i32 && !taken.contains(&(cell.0 as u8, cell.1 as u8))
        })
}

/// The first place a shape fits, along each row from the left. There is always one,
/// at the latest below everything.
pub(super) fn first_place(columns: u8, stock: &[Placed], shape: Shape) -> (u8, u8) {
    let rows = rows_for(stock) as i32;
    (0..rows)
        .flat_map(|y| (0..columns as i32).map(move |x| (x, y)))
        .find(|&(x, y)| fits(columns, stock, shape, x, y))
        .map_or((0, rows as u8), |(x, y)| (x as u8, y as u8))
}

/// Where something let go here ends up: there if it fits, or else the first place.
pub(super) fn landing(columns: u8, stock: &[Placed], shape: Shape, wanted: (i32, i32)) -> (u8, u8) {
    if fits(columns, stock, shape, wanted.0, wanted.1) {
        (wanted.0 as u8, wanted.1 as u8)
    } else {
        first_place(columns, stock, shape)
    }
}
