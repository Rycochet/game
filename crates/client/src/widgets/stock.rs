//! A grid of what isn't carried: a shop's stock, or what is on the ground. Places
//! aren't kept anywhere but here: what comes from the server is only a list.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::id::Id;
use shared::item::{Inventory, Item, ItemCategory, Placed, Rotation, Shape};

use super::ALLOWED;
use super::item::{
    CELL, Floating, ItemLook, ItemName, at_cell, covering, logical_rect, spawn_item_cells,
    spawn_item_grid,
};
use crate::common::{HeldGift, StockCell, TakesGifts};

// Split into files (`cargo xtask lint` keeps files small).
mod place;

#[cfg(test)]
use place::longest_side;
use place::{first_place, landing, rows_for};

/// Where something held over a grid would land, with whose grid it is and its shape.
pub type StockAimAt = (Id, (u8, u8), Shape);

/// What is in a grid, and where.
pub struct StockGrid {
    /// Who it belongs to.
    pub with: Id,
    columns: u8,
    stock: Vec<Placed>,
    /// Things given, on their way, and where they were let go.
    gifts: Vec<Gift>,
    /// It needs drawing again.
    pub changed: bool,
    /// Only this shows clearly: the rest is faded.
    pub filter: Option<ItemCategory>,
}

/// Something given, and where in the grid it was let go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Gift {
    item: Id,
    rotation: Rotation,
    wanted: (i32, i32),
}

/// A grid as drawn.
#[derive(Component, Clone, Copy)]
pub struct StockGridNode(pub Id);

/// What shows in a grid where something held would go.
#[derive(Component, Clone, Copy)]
pub struct StockAim(pub Id);

impl StockGrid {
    /// Each thing in the first place it fits, oldest first.
    pub fn new(with: Id, columns: u8, items: &[Item]) -> Self {
        Self {
            with,
            columns,
            stock: restocked(columns, &[], items, &[]),
            gifts: Vec::new(),
            changed: true,
            filter: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stock.is_empty()
    }

    /// What is in it.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.stock.iter().map(|placed| &placed.item)
    }

    /// The server says what there is: what was there stays where it was, what was
    /// given goes where it was let go if it fits, and the rest in the first place.
    pub fn heard(&mut self, items: &[Item]) {
        self.stock = restocked(self.columns, &self.stock, items, &self.gifts);
        self.gifts.clear();
        self.changed = true;
    }

    /// Something was let go on it: if it is here already it moves, or else it is noted
    /// for when it comes back from the server.
    pub fn gave(&mut self, gift: &HeldGift, node: &ComputedNode, transform: &UiGlobalTransform) {
        let wanted = cell_in_grid(gift.corner, node, transform);
        let rest = others(&self.stock, gift.item);
        let columns = self.columns;
        let moved = self
            .stock
            .iter_mut()
            .find(|placed| placed.item.id == gift.item);
        if let Some(placed) = moved {
            let shape = placed.item.shape().turned(gift.rotation);
            (placed.x, placed.y) = landing(columns, &rest, shape, wanted);
            placed.rotation = gift.rotation;
            self.changed = true;
        } else {
            self.gifts.push(Gift {
                item: gift.item,
                rotation: gift.rotation,
                wanted,
            });
        }
    }

    /// Something was taken from it.
    pub fn took(&mut self, item: Id) {
        self.stock.retain(|placed| placed.item.id != item);
        self.changed = true;
    }

    /// Draws it in this, once anything drawn there has gone. What is held isn't drawn.
    pub fn draw(&self, commands: &mut Commands, holder: Entity, without: Option<Id>) {
        let with = self.with;
        let gifts = TakesGifts(with);
        commands.entity(holder).with_children(|holder| {
            let grid = (StockGridNode(with), gifts);
            let rows = rows_for(&self.stock);
            spawn_item_grid(holder, self.columns, rows, grid, gifts, |grid| {
                grid.spawn((covering(), Floating)).with_children(|layer| {
                    let shown = self.stock.iter();
                    for placed in shown.filter(|placed| Some(placed.item.id) != without) {
                        spawn_stock_item(layer, placed, with, self.filter);
                    }
                });
                grid.spawn((StockAim(with), covering(), Floating));
            });
        });
    }

    /// Where this, held over the grid, would land.
    pub fn aim(
        &self,
        gift: &HeldGift,
        carried: &Inventory,
        node: &ComputedNode,
        transform: &UiGlobalTransform,
    ) -> Option<StockAimAt> {
        let theirs = self.stock.iter().find(|placed| placed.item.id == gift.item);
        let item = theirs
            .map(|placed| &placed.item)
            .or(carried.find(gift.item))?;
        let shape = item.shape().turned(gift.rotation);
        let wanted = cell_in_grid(gift.corner, node, transform);
        let rest = others(&self.stock, gift.item);
        let at = landing(self.columns, &rest, shape, wanted);
        Some((gift.to, at, shape))
    }
}

/// Green where something held would go if let go now, in the aims given.
pub fn draw_stock_aim<'a>(
    commands: &mut Commands,
    aims: impl Iterator<Item = (Entity, &'a StockAim)>,
    aim: Option<StockAimAt>,
) {
    for (entity, shown) in aims {
        commands.entity(entity).despawn_related::<Children>();
        let Some((_, (x, y), shape)) = aim.filter(|aim| aim.0 == shown.0) else {
            continue;
        };
        commands.entity(entity).with_children(|layer| {
            for (cell_x, cell_y) in shape.cells() {
                let (cell_x, cell_y) = ((x + cell_x) as f32, (y + cell_y) as f32);
                // Floating: the grid under them is still what the pointer is on.
                layer.spawn((at_cell(cell_x, cell_y), BackgroundColor(ALLOWED), Floating));
            }
        });
    }
}

fn spawn_stock_item(
    layer: &mut ChildSpawnerCommands,
    placed: &Placed,
    with: Id,
    filter: Option<ItemCategory>,
) {
    // A stack is named by its top, which is what shows.
    let top = placed.item.top();
    let name = ItemName {
        words: top.name(),
        quality: top.quality,
        set: top.set,
    };
    let dimmed = filter.is_some_and(|only| top.kind.category() != only);
    spawn_item_cells(
        layer,
        ItemLook::of(&placed.item)
            .turned(placed.rotation)
            .dimmed(dimmed),
        placed.shape(),
        (placed.x as f32, placed.y as f32),
        true,
        |x, y| {
            (
                name.clone(),
                TakesGifts(with),
                StockCell {
                    with,
                    item: placed.item.clone(),
                    rotation: placed.rotation,
                    grip: (x, y),
                },
            )
        },
    );
}

/// The cell of this grid an item's corner is nearest.
fn cell_in_grid(corner: Vec2, node: &ComputedNode, transform: &UiGlobalTransform) -> (i32, i32) {
    let (grid, _) = logical_rect(node, transform);
    let cell = ((corner - grid) / CELL).round();
    (cell.x as i32, cell.y as i32)
}

/// Everything in a grid but this.
fn others(stock: &[Placed], item: Id) -> Vec<Placed> {
    let others = stock.iter().filter(|placed| placed.item.id != item);
    others.cloned().collect()
}

/// Oldest first. Things without a time are older than time was kept.
fn oldest_first(items: &[Item]) -> Vec<&Item> {
    let mut items: Vec<&Item> = items.iter().collect();
    items.sort_by_key(|item| (item.made_at.map(|made| made.minute), item.id));
    items
}

/// The grid after hearing what there is.
fn restocked(columns: u8, old: &[Placed], items: &[Item], gifts: &[Gift]) -> Vec<Placed> {
    // What was there stays where it was, as it is now: more may be known of it.
    let mut stock: Vec<Placed> = old
        .iter()
        .filter_map(|placed| {
            let item = items.iter().find(|item| item.id == placed.item.id)?;
            Some(Placed {
                item: item.clone(),
                ..placed.clone()
            })
        })
        .collect();
    for item in oldest_first(items) {
        if stock.iter().any(|placed| placed.item.id == item.id) {
            continue;
        }
        let gift = gifts.iter().find(|gift| gift.item == item.id);
        let rotation = gift.map_or(Rotation::None, |gift| gift.rotation);
        let shape = item.shape().turned(rotation);
        let (x, y) = match gift {
            Some(gift) => landing(columns, &stock, shape, gift.wanted),
            None => first_place(columns, &stock, shape),
        };
        stock.push(Placed {
            item: item.clone(),
            x,
            y,
            rotation,
        });
    }
    stock
}

#[cfg(test)]
mod tests;
