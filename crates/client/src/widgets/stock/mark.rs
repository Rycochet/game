//! Making the cells `dom`'s grids draw a stock grid's.

use bevy::prelude::*;

use super::{StockGridNode, StockOf};
use crate::common::{StockCell, TakesGifts};
use crate::dom::{DrawObjects, GridBox, GridShown, ThingCell};
use crate::widgets::ItemName;

/// Marks them once the grids have drawn themselves.
pub(in crate::widgets) fn add(app: &mut App) {
    app.add_systems(PostUpdate, mark_stock.after(DrawObjects));
}

/// Marks each stock grid's cells as drawn, and each cell of a thing in one:
/// named when pointed at, somewhere to give things, and what a press takes.
fn mark_stock(
    boxes: Query<(&StockOf, &GridBox), Changed<GridBox>>,
    cells: Query<(Entity, &ThingCell), Added<ThingCell>>,
    grids: Query<(&StockOf, &GridShown)>,
    mut commands: Commands,
) {
    for (StockOf(with), GridBox(cells)) in &boxes {
        commands.entity(*cells).try_insert(StockGridNode(*with));
    }
    for (entity, cell) in &cells {
        let Ok((StockOf(with), shown)) = grids.get(cell.grid) else {
            continue;
        };
        let things = &mut shown.things.iter();
        let Some(thing) = things.find(|thing| thing.item.id == cell.item) else {
            continue;
        };
        // A stack is named by its top, which is what shows.
        let top = thing.item.top();
        let name = ItemName {
            words: top.name(),
            quality: top.quality,
            set: top.set,
        };
        let stock = StockCell {
            with: *with,
            item: thing.item.clone(),
            rotation: cell.rotation,
            grip: cell.grip,
        };
        let marks = (name, TakesGifts(*with), stock);
        commands.entity(entity).try_insert(marks);
    }
}
