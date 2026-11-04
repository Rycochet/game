//! Opening grids in panels.

use bevy::prelude::*;
use shared::item::{Holder, Inventory};

use super::super::body::spawn_rows;
use super::super::filter::spawn_filter;
use super::super::ground::{GroundHolder, spawn_section};
use super::super::sort::spawn_sort;
use super::{
    AimLayer, BinNode, BinSlot, CELL_BORDER, COLUMN_GAP, DropNode, FIRST_AT, FONT, GridNode,
    ItemLayer, NEXT_AT, OpenGrids, WeightText,
};
use crate::common::SheetText;
use crate::widgets::{
    CELL, ContainerLook, ContainerSize, Floating, GRID_LINE, INK, RaisePanel, covering,
    spawn_container, spawn_item_grid, spawn_moveable_panel,
};

impl OpenGrids {
    /// Opens this grid's panel, or brings it to the front if it is open.
    pub(in crate::inventory) fn open(
        &mut self,
        holder: Holder,
        carried: &Inventory,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
    ) {
        if let Some(&(_, panel, _)) = self.open.iter().find(|(open, ..)| *open == holder) {
            raise.write(RaisePanel(panel));
            return;
        }
        let Some((width, height)) = carried.grid_size(holder) else {
            return;
        };
        let title = match holder {
            Holder::Inventory => "Character",
            Holder::Inside(id) => carried.find(id).map_or("", |item| item.kind.def().name),
        };
        let at = FIRST_AT + NEXT_AT * (self.opened % 8) as f32;
        self.opened += 1;
        let (panel, content) = spawn_moveable_panel(commands, title, at);
        let mut ground_holder = None;
        commands.entity(content).with_children(|content| {
            if holder != Holder::Inventory {
                spawn_grid(content, holder, width, height);
                return;
            }
            // What they are like, what they wear and what they carry.
            content
                .spawn(Node {
                    column_gap: px(COLUMN_GAP),
                    ..default()
                })
                .with_children(|columns| {
                    columns.spawn((SheetText, Text::default(), FONT.font(), TextColor(INK)));
                    spawn_rows(columns);
                    spawn_grid(columns, holder, width, height);
                    ground_holder = Some(spawn_section(columns));
                });
        });
        if let Some(ground_holder) = ground_holder {
            commands.entity(ground_holder).insert(GroundHolder);
        }
        self.open.push((holder, panel, (width, height)));
    }

    /// Whether the inventory's panel is open.
    pub(super) fn inventory_open(&self) -> bool {
        self.open
            .iter()
            .any(|(open, ..)| *open == Holder::Inventory)
    }

    pub(super) fn toggle_inventory(
        &mut self,
        carried: &Inventory,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
    ) {
        let open = self
            .open
            .iter()
            .position(|(open, ..)| *open == Holder::Inventory);
        match open {
            Some(index) => {
                let (_, panel, _) = self.open.remove(index);
                commands.entity(panel).despawn();
            }
            None => self.open(Holder::Inventory, carried, commands, raise),
        }
    }
}

/// A grid in its container: sort and filter above it, and under it the words on its
/// weight and, for the inventory, the Drop and bin slots.
pub(super) fn spawn_grid(parent: &mut ChildSpawnerCommands, holder: Holder, width: u8, height: u8) {
    let look = ContainerLook {
        title: None,
        controls: true,
        size: ContainerSize::Fixed,
        weight: true,
        slots: holder == Holder::Inventory,
    };
    let parts = spawn_container(parent, look);
    let mut commands = parent.commands();
    if let Some(controls) = parts.controls {
        commands.entity(controls).with_children(|row| {
            spawn_sort(row, holder);
            spawn_filter(row, Some(holder));
        });
    }
    commands.entity(parts.grid).with_children(|parent| {
        spawn_item_grid(parent, width, height, GridNode { holder }, (), |grid| {
            grid.spawn((ItemLayer { holder }, covering(), Floating));
            grid.spawn((AimLayer { holder }, covering(), Floating));
        });
    });
    if let Some(weight) = parts.weight {
        commands.entity(weight).insert(WeightText { holder });
    }
    if let Some(slots) = parts.slots {
        commands.entity(slots).with_children(|slots| {
            let slot = || Node {
                width: px(CELL),
                height: px(CELL),
                border: UiRect::all(px(CELL_BORDER)),
                ..default()
            };
            slots.spawn((DropNode, slot(), BorderColor::from(GRID_LINE)));
            slots.spawn((BinNode, BinSlot, slot(), BorderColor::from(GRID_LINE)));
        });
    }
}
