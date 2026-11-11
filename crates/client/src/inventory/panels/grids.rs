//! Opening grids in panels.

use bevy::prelude::*;
use shared::item::{Holder, Inventory};

use super::frame::spawn_grid;
use super::pages::spawn_pages;
use super::shown::{grows, shown_size};
use super::{FIRST_AT, NEXT_AT, OpenGrids};
use crate::widgets::{RaisePanel, spawn_moveable_panel};

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
        let Some((width, height)) = shown_size(carried, holder) else {
            return;
        };
        let grows = grows(carried, holder);
        let title = match holder {
            Holder::Inventory => "Character",
            Holder::Inside(id) => carried.find(id).map_or("", |item| item.kind.def().name),
        };
        let at = FIRST_AT + NEXT_AT * (self.opened % 8) as f32;
        self.opened += 1;
        let (panel, content) = spawn_moveable_panel(commands, title, at);
        let id = super::super::ui_ids::grid_id(Some(holder), "panel");
        commands.entity(panel).insert(id);
        let page = self.page;
        commands.entity(content).with_children(|content| {
            if holder != Holder::Inventory {
                spawn_grid(content, holder, grows);
                return;
            }
            // Tabs for what they are like, what they wear and carry, and what they
            // can do.
            content
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|column| {
                    self.pages = Some(spawn_pages(column, page));
                });
        });
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
                self.pages = None;
            }
            None => self.open(Holder::Inventory, carried, commands, raise),
        }
    }
}
