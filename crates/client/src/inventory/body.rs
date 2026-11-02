//! The clothes on the Character panel: a row for each place on the character's body and a column
//! for each layer. Items are dragged on to wear them and off to take them off,
//! with the same holding as everything else.

use bevy::prelude::*;
use shared::creature::Layer;
use shared::id::Id;
use shared::item::{Inventory, Item, Rotation};
use shared::wear::Fit;

use super::hold::Target;
use super::panels::ItemCell;
use super::{Carried, Holding};
use crate::widgets::{
    ALLOWED, CELL, GRID_COLOUR, GRID_LINE, INK, ITEM_EDGE, ITEM_LABEL_FONT, Pressable, REFUSED,
    item_colour, item_edge, item_edge_width, item_label, set_edge,
};

const NAME_WIDTH: f32 = 80.0;
const FONT: f32 = 14.0;
const BORDER: f32 = 1.0;
const COVERED_ALPHA: f32 = 0.45;

/// On the panel's slots and everything in them: dropping here means wearing.
#[derive(Component, Clone, Copy)]
pub(super) struct BodyNode;

/// What holds the rows, and shows whether a drop would work.
#[derive(Component)]
pub(super) struct BodyRows;

/// The clothes' part of the Character panel.
pub(super) fn spawn_rows(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        BodyRows,
        BodyNode,
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
}

fn words(text: impl Into<String>, width: f32) -> impl Bundle {
    (
        BodyNode,
        Text::new(text),
        TextFont::from_font_size(FONT),
        TextColor(INK),
        Node {
            width: px(width),
            ..default()
        },
    )
}

fn slot_node() -> Node {
    Node {
        width: px(CELL),
        height: px(CELL),
        border: UiRect::all(px(BORDER)),
        ..default()
    }
}

/// How a slot is drawn: the worn things here, last on top.
fn spawn_slot(row: &mut ChildSpawnerCommands, carried: &Inventory, here: &[&Item]) {
    let Some(item) = here.last() else {
        row.spawn((
            BodyNode,
            slot_node(),
            BackgroundColor(GRID_COLOUR),
            BorderColor::from(GRID_LINE),
        ));
        return;
    };
    let covered = carried.covered_by(item.id).is_some();
    let tint = item_colour(item.kind).with_alpha(if covered { COVERED_ALPHA } else { 1.0 });
    // Held by a cell it has, and it is never turned while worn.
    let grip = item.shape().cells().next().unwrap_or((0, 0));
    let cell = ItemCell {
        id: item.id,
        rotation: Rotation::None,
        grip,
    };
    let text = match here.len() {
        1 => item_label(item.kind),
        many => format!("{}{many}", item_label(item.kind)),
    };
    let mut slot = row.spawn((
        BodyNode,
        cell,
        Node {
            border: UiRect::all(px(item_edge_width(item.quality))),
            ..slot_node()
        },
        BackgroundColor(tint),
        BorderColor::from(item_edge(item.quality)),
    ));
    // Something under another can't be picked up.
    if !covered {
        slot.insert(Pressable);
    }
    if item.set {
        let width = item_edge_width(item.quality);
        slot.with_child(set_edge(item.quality, width, width));
    }
    slot.with_child((
        BodyNode,
        cell,
        Text::new(text),
        TextFont::from_font_size(ITEM_LABEL_FONT),
        TextColor(ITEM_EDGE),
    ));
}

/// Draws the rows again when what is worn, or the body, or what is held changes.
pub(super) fn show_clothes(
    carried: Res<Carried>,
    holding: Res<Holding>,
    new_rows: Query<(), Added<BodyRows>>,
    rows: Query<Entity, With<BodyRows>>,
    mut commands: Commands,
    mut drawn_without: Local<Option<Id>>,
) {
    let held = holding.held.map(|held| held.item.id);
    if !carried.is_changed() && new_rows.is_empty() && *drawn_without == held {
        return;
    }
    *drawn_without = held;
    for entity in &rows {
        commands.entity(entity).despawn_related::<Children>();
        let inventory = &carried.0;
        let def = inventory.creature.def();
        commands.entity(entity).with_children(|rows| {
            rows.spawn((
                BodyNode,
                Text::new(format!("{}, {:?}", def.name, def.size)),
                TextFont::from_font_size(FONT),
                TextColor(INK),
            ));
            rows.spawn((BodyNode, Node::default()))
                .with_children(|row| {
                    row.spawn(words("", NAME_WIDTH));
                    for layer in Layer::ALL {
                        row.spawn(words(layer_letter(layer), CELL));
                    }
                });
            for part in def.body {
                rows.spawn((BodyNode, Node::default()))
                    .with_children(|row| {
                        row.spawn(words(format!("{:?}", part.location), NAME_WIDTH));
                        for layer in Layer::ALL {
                            if !part.layers.contains(&layer) {
                                row.spawn((BodyNode, slot_node()));
                                continue;
                            }
                            let here: Vec<&Item> = inventory
                                .worn
                                .iter()
                                .filter(|item| Some(item.id) != held)
                                .filter(|item| {
                                    item.kind.def().wear.is_some_and(|wear| {
                                        wear.layer == layer
                                            && wear.locations.contains(&part.location)
                                    })
                                })
                                .collect();
                            spawn_slot(row, inventory, &here);
                        }
                    });
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

/// What a worn item's name adds: why it can't be moved, and how it fits.
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
