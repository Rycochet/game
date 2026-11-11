//! The clothes on the Character panel: a row for each place on the character's body and a column
//! for each layer, then the Held column (`hands`). Items are dragged on to wear them
//! and off to take them off, with the same holding as everything else.

use bevy::prelude::*;
use shared::creature::{Layer, Location};
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::wear::{Fit, Remover};

use super::hands::spawn_held_column;
use super::hold::Target;
use super::{Carried, Holding};
use crate::common::GodMode;
use crate::dom::{SlotShown, SlotThing};
use crate::widgets::{ALLOWED, CELL, INK, REFUSED, TextSize};

const NAME_WIDTH: f32 = 80.0;
const FONT: TextSize = TextSize::Small;
const BORDER: f32 = 1.0;
/// Between the clothes and the Held column.
const COLUMN_GAP: f32 = 8.0;

/// On the panel's slots and everything in them: dropping here means wearing.
#[derive(Component, Clone, Copy)]
pub(super) struct BodyNode;

/// What holds the rows and the Held column, and shows whether a drop would work.
#[derive(Component)]
pub(super) struct BodyRows;

/// The clothes' part of the Character panel.
pub(super) fn spawn_rows(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        BodyRows,
        BodyNode,
        Node {
            column_gap: px(COLUMN_GAP),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
}

pub(super) fn words(text: impl Into<String>, width: f32) -> impl Bundle {
    (
        BodyNode,
        Text::new(text),
        FONT.font(),
        TextColor(INK),
        Node {
            width: px(width),
            ..default()
        },
    )
}

pub(super) fn slot_node() -> Node {
    Node {
        width: px(CELL),
        height: px(CELL),
        border: UiRect::all(px(BORDER)),
        ..default()
    }
}

/// A slot with the worn things here, the last on top: it draws itself
/// (`dom::SlotShown`).
pub(super) fn spawn_slot(
    row: &mut ChildSpawnerCommands,
    carried: &Inventory,
    here: &[&Item],
    god: bool,
) {
    let thing = here.last().map(|item| SlotThing {
        item: (*item).clone(),
        count: here.len(),
        // Something under another can be picked up, only more slowly.
        faded: carried.covered_by(item.id).is_some(),
        // A curse once it is known can't be.
        pressable: god || carried.may_remove(item.id, Remover::Wearer),
    });
    let shown = SlotShown { thing, ..default() };
    row.spawn((BodyNode, shown));
}

/// Draws the rows again when what is worn, or the body, or what is held changes.
pub(super) fn show_clothes(
    carried: Res<Carried>,
    holding: Res<Holding>,
    god: Res<GodMode>,
    new_rows: Query<(), Added<BodyRows>>,
    rows: Query<Entity, With<BodyRows>>,
    mut commands: Commands,
    mut drawn_without: Local<Option<Id>>,
) {
    let held = holding.held.map(|held| held.item.id);
    if !carried.is_changed() && !god.is_changed() && new_rows.is_empty() && *drawn_without == held {
        return;
    }
    *drawn_without = held;
    for entity in &rows {
        commands.entity(entity).despawn_related::<Children>();
        let inventory = &carried.0;
        commands.entity(entity).with_children(|both| {
            both.spawn((
                BodyNode,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|rows| spawn_table(rows, inventory, held, god.0));
            spawn_held_column(both, inventory, held, god.0);
        });
    }
}

/// The table of clothes: a row per place on the body but Held, a column per layer.
fn spawn_table(
    rows: &mut ChildSpawnerCommands,
    inventory: &Inventory,
    held: Option<Id>,
    god: bool,
) {
    let def = inventory.creature.def();
    rows.spawn((
        BodyNode,
        Text::new(format!("{}, {:?}", def.name, def.size)),
        FONT.font(),
        TextColor(INK),
    ));
    rows.spawn((BodyNode, Node::default()))
        .with_children(|row| {
            row.spawn(words("", NAME_WIDTH));
            for layer in Layer::ALL {
                row.spawn(words(layer_letter(layer), CELL));
            }
        });
    for part in def
        .body
        .iter()
        .filter(|part| part.location != Location::Held)
    {
        rows.spawn((BodyNode, Node::default()))
            .with_children(|row| {
                row.spawn(words(format!("{:?}", part.location), NAME_WIDTH));
                for layer in Layer::ALL {
                    // Only rings are on fingers alone: what else covers them shows
                    // on Hands (owner, 2026-11-07).
                    let fingers_only = part.location == Location::Fingers && layer != Layer::Skin;
                    if fingers_only || !part.layers.contains(&layer) {
                        row.spawn((BodyNode, slot_node()));
                        continue;
                    }
                    let here: Vec<&Item> = inventory
                        .worn
                        .iter()
                        .filter(|item| Some(item.id) != held)
                        .filter(|item| {
                            item.kind.def().wear.is_some_and(|wear| wear.layer == layer)
                                && item.worn_locations().contains(&part.location)
                        })
                        .collect();
                    spawn_slot(row, inventory, &here, god);
                }
            });
    }
}

/// The heading of a layer's column.
fn layer_letter(layer: Layer) -> &'static str {
    match layer {
        Layer::Skin => "In",
        Layer::Middle => "Mid",
        Layer::Outer => "Out",
        Layer::Over => "Over",
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
