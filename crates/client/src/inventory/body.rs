//! The clothes on the Character panel: a row for each place on the character's body and a column
//! for each layer, then the Held column (`hands`). Items are dragged on to wear them
//! and off to take them off, with the same holding as everything else.

use bevy::prelude::*;
use shared::creature::{BodyPart, CreatureKind, Layer, Location};
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::wear::{Fit, Remover};

use super::hands::{held_now, in_hands};
use super::hold::Target;
use super::{Carried, Holding};
use crate::common::GodMode;
use crate::dom::{Dom, SlotShown, SlotThing, Template, document_in};
use crate::html;
use crate::widgets::{ALLOWED, REFUSED, UiId};

/// The clothes: the body's kind, a heading for each layer, a row for each place
/// on the body (copied from the template) and the Held column. Each slot is a
/// place for one that draws itself (`dom::SlotShown`), where the body has one.
static CLOTHES: Template = html!(
    "inventory.clothes",
    r#"
<form id="clothes">
  <span id="body" class="columns" valign="top">
    <div class="joined">
      <small id="kind"/>
      <span class="joined">
        <small width="80"/>
        <small width="32">In</small>
        <small width="32">Mid</small>
        <small width="32">Out</small>
        <small width="32">Over</small>
      </span>
      <div id="parts" class="joined"/>
    </div>
    <div id="hands" class="joined">
      <small width="32">&#160;</small>
      <small width="32">Held</small>
      <object-slot id="hand0" accepts="item" width="32" height="32"/>
      <object-slot id="hand1" accepts="item" width="32" height="32"/>
    </div>
  </span>
  <template id="part">
    <span id="row" class="joined">
      <small id="name" width="80"/>
      <object-slot id="skin" accepts="item" width="32" height="32"/>
      <object-slot id="middle" accepts="item" width="32" height="32"/>
      <object-slot id="outer" accepts="item" width="32" height="32"/>
      <object-slot id="over" accepts="item" width="32" height="32"/>
    </span>
  </template>
</form>
"#
);

/// On the clothes and everything in them: dropping here means wearing.
#[derive(Component, Clone, Copy)]
pub(super) struct BodyNode;

/// What holds the rows and the Held column, and shows whether a drop would work.
#[derive(Component)]
pub(super) struct BodyRows;

/// A slot of the clothes: a layer of a place on the body, or a hand.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ClothesSlot {
    Part(Location, Layer),
    Hand(usize),
}

/// The clothes' part of the Character panel, in `place`, built by the end of
/// the frame.
pub(super) fn spawn_rows(commands: &mut Commands, place: Entity) {
    commands.spawn(document_in(&CLOTHES, place));
}

/// The places on this body that have a row: all but Held, which has its column.
fn parts(kind: CreatureKind) -> impl Iterator<Item = &'static BodyPart> {
    let body = kind.def().body.iter();
    body.filter(|part| part.location != Location::Held)
}

/// A slot's layer, by its name in the template.
fn layer_named(name: &str) -> Option<Layer> {
    match name {
        "skin" => Some(Layer::Skin),
        "middle" => Some(Layer::Middle),
        "outer" => Some(Layer::Outer),
        "over" => Some(Layer::Over),
        _ => None,
    }
}

/// Gives each new slot of the clothes its slot to draw, where the body has one,
/// and marks the whole of them as somewhere to wear things.
pub(super) fn bind_clothes(
    new: Query<(Entity, &UiId), Added<UiId>>,
    carried: Res<Carried>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        let ("clothes", name) = id.parts() else {
            continue;
        };
        let slot = match (name, id.index()) {
            ("body", _) => {
                let marks = (BodyRows, BodyNode, BackgroundColor(Color::NONE));
                commands.entity(entity).insert(marks);
                continue;
            }
            ("hand0", _) => Some(ClothesSlot::Hand(0)),
            ("hand1", _) => Some(ClothesSlot::Hand(1)),
            (layer, Some(row)) => layer_named(layer).and_then(|layer| {
                let part = parts(carried.0.creature).nth(row)?;
                // Only rings are on fingers alone: what else covers them shows
                // on Hands (owner, 2026-11-07).
                let fingers_only = part.location == Location::Fingers && layer != Layer::Skin;
                let has = part.layers.contains(&layer) && !fingers_only;
                has.then_some(ClothesSlot::Part(part.location, layer))
            }),
            _ => None,
        };
        if let Some(slot) = slot {
            let shown = SlotShown::default();
            commands.entity(entity).with_child((BodyNode, slot, shown));
        }
    }
}

/// What a slot holds: the worn things there, the last on top.
fn thing_in(here: &[&Item], carried: &Inventory, god: bool) -> Option<SlotThing> {
    let item = here.last()?;
    Some(SlotThing {
        item: (*item).clone(),
        count: here.len(),
        // Something under another can be picked up, only more slowly.
        faded: carried.covered_by(item.id).is_some(),
        // A curse once it is known can't be.
        pressable: god || carried.may_remove(item.id, Remover::Wearer),
    })
}

/// Writes the rows again when the body changes, and says what each slot holds
/// when what is worn, or what is held, changes.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_clothes(
    carried: Res<Carried>,
    holding: Res<Holding>,
    god: Res<GodMode>,
    built: Query<(), With<BodyRows>>,
    mut slots: Query<(&ClothesSlot, &mut SlotShown)>,
    new_slots: Query<(), Added<ClothesSlot>>,
    mut dom: Dom,
    (mut rows_for, mut drawn_without): (Local<Option<CreatureKind>>, Local<Option<Id>>),
) {
    if built.is_empty() {
        *rows_for = None;
        return;
    }
    let inventory = &carried.0;
    let kind = inventory.creature;
    if *rows_for != Some(kind) {
        *rows_for = Some(kind);
        let def = kind.def();
        dom.set_text("clothes/kind", format!("{}, {:?}", def.name, def.size));
        dom.empty("clothes/parts");
        for (row, part) in parts(kind).enumerate() {
            dom.copy_template("clothes/part", "clothes/parts", row);
            let name = format!("{:?}", part.location);
            dom.set_text(&format!("clothes/name.{row}"), name);
        }
        // Only a body with somewhere to hold things has the column.
        let holds = def.body.iter().any(|part| part.location == Location::Held);
        dom.set_flag("clothes/hands", "hidden", !holds);
    }
    let held = holding.held.map(|held| held.item.id);
    let changed = carried.is_changed() || god.is_changed() || *drawn_without != held;
    if !changed && new_slots.is_empty() {
        return;
    }
    *drawn_without = held;
    let in_hand = held_now(inventory, held);
    let hands = in_hands(&in_hand);
    for (slot, mut shown) in &mut slots {
        let thing = match *slot {
            ClothesSlot::Hand(hand) => thing_in(hands[hand], inventory, god.0),
            ClothesSlot::Part(location, layer) => {
                let worn = inventory.worn.iter();
                let here: Vec<&Item> = worn
                    .filter(|item| Some(item.id) != held)
                    .filter(|item| {
                        item.kind.def().wear.is_some_and(|wear| wear.layer == layer)
                            && item.worn_locations().contains(&location)
                    })
                    .collect();
                thing_in(&here, inventory, god.0)
            }
        };
        // (Only when it differs: the slot draws itself again when changed.)
        if shown.thing != thing {
            shown.thing = thing;
        }
    }
}

/// Green or red behind the rows, while something held is over them.
pub(super) fn show_body_aim(
    holding: Res<Holding>,
    mut rows: Query<&mut BackgroundColor, With<BodyRows>>,
) {
    let tint = match holding.aim {
        Some(Target::Body { allowed: true }) => ALLOWED,
        Some(Target::Body { allowed: false }) => REFUSED,
        _ => Color::NONE,
    };
    for mut background in &mut rows {
        background.set_if_neq(BackgroundColor(tint));
    }
}

/// What a worn item's name adds: what is over it, slowing changing it, and how it
/// fits.
pub(super) fn worn_notes(carried: &Inventory, id: Id) -> String {
    let mut notes = String::new();
    if let Some(over) = carried.covered_by(id).and_then(|over| carried.find(over)) {
        notes.push_str(&format!(" (under {})", over.kind.def().name));
    }
    if let Some(Fit::Loose(by)) = carried.worn_fit(id) {
        notes.push_str(&format!(" (loose by {by})"));
    }
    notes
}

#[cfg(test)]
mod tests;
