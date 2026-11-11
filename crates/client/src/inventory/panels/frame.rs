//! A grid in its frame, as markup: sort and filter above it, and under it the
//! words on its weight and, for the inventory, the Drop and bin slots. The
//! places in it are filled here once it is built (`bind_frames`).

use bevy::prelude::*;
use shared::item::Holder;

use super::super::filter::spawn_filter;
use super::super::sort::spawn_sort;
use super::{BinNode, BinSlot, DropNode, FONT, GridOwner, WeightText};
use crate::dom::{DocumentOf, GridShown, SlotShown, Template, document_in};
use crate::html;
use crate::widgets::{INK, UiId};

/// What is carried, with the slots for dropping and throwing away.
static INVENTORY: Template = html!(
    "inventory.grid",
    r#"
<form id="inventory">
  <container id="frame">
    <header id="controls"/>
    <object-grid id="grid" type="item" places="kept"/>
    <footer>
      <canvas id="weight" fills/>
      <span>
        <object-slot id="drop" accepts="item"/>
        <object-slot id="bin" accepts="item"/>
      </span>
    </footer>
  </container>
</form>
"#
);

/// A bag: one of several documents alike, by the bag (`bag#<id>/...`).
static BAG: Template = html!(
    "inventory.bag",
    r#"
<form id="bag">
  <container id="frame">
    <header id="controls"/>
    <object-grid id="grid" type="item" places="kept"/>
    <footer><canvas id="weight" fills/></footer>
  </container>
</form>
"#
);

/// A bag whose grid grows with what is in it: it scrolls past a height.
static GROWING_BAG: Template = html!(
    "inventory.growing_bag",
    r#"
<form id="bag">
  <container id="frame">
    <header id="controls"/>
    <scroll id="scroll" max-height="60%">
      <object-grid id="grid" type="item" places="kept" grows/>
    </scroll>
    <footer><canvas id="weight" fills/></footer>
  </container>
</form>
"#
);

/// A grid in its frame, built by the end of the frame. One that `grows`
/// scrolls.
pub(super) fn spawn_grid(parent: &mut ChildSpawnerCommands, holder: Holder, grows: bool) {
    let place = parent.spawn(Node::default()).id();
    let mut commands = parent.commands();
    match holder {
        Holder::Inventory => {
            commands.spawn(document_in(&INVENTORY, place));
        }
        Holder::Inside(bag) => {
            let template = if grows { &GROWING_BAG } else { &BAG };
            commands.spawn((document_in(template, place), DocumentOf(bag)));
        }
    }
}

/// Fills the places of each frame once it is built: its controls, its grid
/// (which draws itself: `show_grids` says what is in it), its weight and its
/// slots.
pub(in crate::inventory) fn bind_frames(
    new: Query<(Entity, &UiId), Added<UiId>>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        let (form, name) = id.parts();
        let holder = match (form, id.which()) {
            ("inventory", _) => Holder::Inventory,
            ("bag", Some(bag)) => Holder::Inside(bag),
            _ => continue,
        };
        let mut place = commands.entity(entity);
        match name {
            "controls" => {
                place.with_children(|row| {
                    spawn_sort(row, holder);
                    spawn_filter(row, Some(holder));
                });
            }
            "grid" => {
                place.insert((GridOwner { holder }, GridShown::default()));
            }
            "weight" => {
                let words = (Text::default(), FONT.font(), TextColor(INK));
                place.with_child((WeightText { holder }, words));
            }
            // Slots drawing themselves: `show_drop` and `show_bins` say what
            // each shows.
            "drop" => {
                place.with_child((DropNode, SlotShown::default()));
            }
            "bin" => {
                place.with_child((BinNode, BinSlot, SlotShown::default()));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
