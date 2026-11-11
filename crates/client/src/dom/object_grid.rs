//! `object-grid`: a grid of things that draws itself. Whoever owns it says
//! what is in it and what to tint (`GridShown`); this draws the cells, the
//! things in them and where something held would go. What picking a thing up
//! or letting one go there does is still its owner's, which finds a pressed
//! cell by its `ThingCell`.
//!
//! Any node with a `GridShown` is drawn as one: the element in markup, or a
//! node a screen not yet moved to markup spawns.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Item, Rotation};

use crate::common::TakesGifts;
use crate::widgets::{
    ALLOWED, CELL, Floating, GRID_COLOUR, GRID_LINE, ItemLook, REFUSED, at_cell, covering,
    spawn_item_cells,
};

/// When grids and slots are drawn (`PostUpdate`): an owner's system giving
/// new cells its own markers goes `.after` this.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DrawObjects;

/// One thing in a grid, and where: its top left cell, as turned.
#[derive(Clone, Debug, PartialEq)]
pub struct GridThing {
    pub item: Item,
    pub x: u8,
    pub y: u8,
    pub rotation: Rotation,
    /// Faded, as not what is being looked for.
    pub dimmed: bool,
}

/// Cells to tint where something held would go: green if it may, red if not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridAim {
    pub cells: Vec<(i32, i32)>,
    pub allowed: bool,
}

/// What a grid shows, set by its owner. It is drawn again when this changes.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct GridShown {
    pub columns: u8,
    pub rows: u8,
    pub things: Vec<GridThing>,
    pub aim: Option<GridAim>,
    /// All of it is somewhere to drop things to give them to this one.
    pub takes_gifts: Option<Id>,
}

/// On a grid, once drawn: the node of its cells, whose place on the screen
/// says which cell the pointer is over. It is made again when the grid's
/// size changes.
#[derive(Component, Clone, Copy, Debug)]
pub struct GridBox(pub Entity);

/// On each cell of a thing drawn in a grid or a slot (and its label): which
/// grid or slot, which thing, how it is turned, and which cell of its shape
/// this is.
#[derive(Component, Clone, Copy, Debug)]
pub struct ThingCell {
    pub grid: Entity,
    pub item: Id,
    pub rotation: Rotation,
    pub grip: (u8, u8),
}

/// What a grid was last drawn as, and the layers its things and aim are in.
#[derive(Component)]
pub(super) struct GridDrawn {
    shown: GridShown,
    cells: Entity,
    things: Entity,
    aim: Entity,
}

/// Draws each grid again as far as what it shows has changed: the whole box
/// for another size, or only its things, or only its aim.
pub(super) fn draw_object_grids(
    mut commands: Commands,
    changed: Query<(Entity, &GridShown, Option<&GridDrawn>), Changed<GridShown>>,
) {
    for (grid, shown, drawn) in &changed {
        let same_box = drawn.is_some_and(|drawn| {
            let was = &drawn.shown;
            (was.columns, was.rows, was.takes_gifts)
                == (shown.columns, shown.rows, shown.takes_gifts)
        });
        let (cells, things, aim) = match (drawn, same_box) {
            (Some(drawn), true) => (drawn.cells, drawn.things, drawn.aim),
            _ => {
                if let Some(drawn) = drawn {
                    commands.entity(drawn.cells).try_despawn();
                }
                let made = spawn_box(&mut commands, grid, shown);
                commands.entity(grid).insert(GridBox(made.0));
                made
            }
        };
        let was = drawn.filter(|_| same_box).map(|drawn| &drawn.shown);
        if was.is_none_or(|was| was.things != shown.things) {
            draw_things(&mut commands, grid, things, shown);
        }
        if was.is_none_or(|was| was.aim != shown.aim) {
            draw_aim(&mut commands, aim, shown);
        }
        commands.entity(grid).insert(GridDrawn {
            shown: shown.clone(),
            cells,
            things,
            aim,
        });
    }
}

/// The grid's cells, with the layers its things and aim are drawn in.
fn spawn_box(commands: &mut Commands, grid: Entity, shown: &GridShown) -> (Entity, Entity, Entity) {
    let node = Node {
        width: px(shown.columns as f32 * CELL),
        height: px(shown.rows as f32 * CELL),
        flex_shrink: 0.0,
        ..default()
    };
    let gifts = shown.takes_gifts.map(TakesGifts);
    let mut made = commands.spawn((node, BackgroundColor(GRID_COLOUR), ChildOf(grid)));
    if let Some(gifts) = gifts {
        made.insert(gifts);
    }
    let made = made.id();
    for y in 0..shown.rows {
        for x in 0..shown.columns {
            let line = (at_cell(x as f32, y as f32), BorderColor::from(GRID_LINE));
            let mut line = commands.spawn((line, ChildOf(made)));
            if let Some(gifts) = gifts {
                line.insert(gifts);
            }
        }
    }
    // Floating: the cells in them are what the pointer is on, or the grid.
    let things = commands.spawn((covering(), Floating, ChildOf(made))).id();
    let aim = commands.spawn((covering(), Floating, ChildOf(made))).id();
    (made, things, aim)
}

fn draw_things(commands: &mut Commands, grid: Entity, layer: Entity, shown: &GridShown) {
    commands.entity(layer).despawn_related::<Children>();
    commands.entity(layer).with_children(|layer| {
        for thing in &shown.things {
            let (item, rotation) = (thing.item.id, thing.rotation);
            let look = ItemLook::of(&thing.item)
                .turned(rotation)
                .dimmed(thing.dimmed);
            let shape = thing.item.shape().turned(rotation);
            let corner = (thing.x as f32, thing.y as f32);
            spawn_item_cells(layer, look, shape, corner, true, |x, y| ThingCell {
                grid,
                item,
                rotation,
                grip: (x, y),
            });
        }
    });
}

fn draw_aim(commands: &mut Commands, layer: Entity, shown: &GridShown) {
    commands.entity(layer).despawn_related::<Children>();
    let Some(aim) = &shown.aim else {
        return;
    };
    let tint = if aim.allowed { ALLOWED } else { REFUSED };
    let (columns, rows) = (shown.columns as i32, shown.rows as i32);
    commands.entity(layer).with_children(|layer| {
        for &(x, y) in &aim.cells {
            if (0..columns).contains(&x) && (0..rows).contains(&y) {
                // Floating: the grid under them is still what the pointer is on.
                let cell = at_cell(x as f32, y as f32);
                layer.spawn((cell, BackgroundColor(tint), Floating));
            }
        }
    });
}

#[cfg(test)]
mod tests;
