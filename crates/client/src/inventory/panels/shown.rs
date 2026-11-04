//! How big a grid is drawn: its whole size, or for a bag that grows
//! (`ContainerDef::grows`), only the rows in use and room below for the largest
//! thing it could take.

use shared::item::{Holder, Inventory, Placed};

/// The grid's columns and the rows drawn, or `None` if it isn't carried.
pub(super) fn shown_size(carried: &Inventory, holder: Holder) -> Option<(u8, u8)> {
    let (width, height) = carried.grid_size(holder)?;
    let Holder::Inside(id) = holder else {
        return Some((width, height));
    };
    let bag = carried.find(id)?;
    if !bag
        .kind
        .def()
        .container
        .is_some_and(|container| container.grows)
    {
        return Some((width, height));
    }
    // Anything put in must be smaller than its outside, so this is room enough.
    let outside = bag.shape();
    let room = outside.width.max(outside.height);
    Some((width, rows_for(&bag.contents, room, height)))
}

/// What is used, and `room` more, but never more rows than it has.
fn rows_for(contents: &[Placed], room: u8, most: u8) -> u8 {
    let cells = contents.iter().flat_map(|placed| placed.cells());
    let lowest = cells.map(|(_, y)| y + 1).max().unwrap_or(0);
    lowest.saturating_add(room).min(most)
}

/// Whether the grid grows, and so scrolls.
pub(super) fn grows(carried: &Inventory, holder: Holder) -> bool {
    let Holder::Inside(id) = holder else {
        return false;
    };
    let container = carried.find(id).and_then(|bag| bag.kind.def().container);
    container.is_some_and(|container| container.grows)
}

#[cfg(test)]
mod tests;
