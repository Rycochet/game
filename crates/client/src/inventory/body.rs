//! The clothes on the Character panel: a row for each place on the character's body and a column
//! for each layer. Items are dragged on to wear them and off to take them off,
//! with the same holding as everything else.

use bevy::prelude::*;
use shared::creature::Layer;
use shared::id::Id;
use shared::item::{Inventory, Item, Rotation, Shape};
use shared::wear::{Fit, Remover};

use super::hold::Target;
use super::panels::ItemCell;
use super::{Carried, Holding};
use crate::common::GodMode;
use crate::item_art::{ItemPicture, look_of};
use crate::widgets::{
    ALLOWED, Aura, AuraMask, CELL, Floating, GRID_COLOUR, GRID_LINE, INK, ITEM_EDGE,
    ITEM_LABEL_FONT, PICTURE_PIXELS_PER_CELL, Pressable, REFUSED, TextSize, aura_glow, item_colour,
    item_edge, item_edge_width, item_label, set_edge,
};

const NAME_WIDTH: f32 = 80.0;
const FONT: TextSize = TextSize::Small;
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
        FONT.font(),
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
fn spawn_slot(row: &mut ChildSpawnerCommands, carried: &Inventory, here: &[&Item], god: bool) {
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
    let alpha = if covered { COVERED_ALPHA } else { 1.0 };
    let picture = look_of(item);
    let tint = if picture.is_some() {
        Color::NONE
    } else {
        item_colour(item.kind).with_alpha(alpha)
    };
    // Held by a cell it has, and it is never turned while worn.
    let grip = item.shape().cells().next().unwrap_or((0, 0));
    let cell = ItemCell {
        id: item.id,
        rotation: Rotation::None,
        grip,
    };
    // A picture says what it is, leaving only how many.
    let label = if picture.is_some() {
        String::new()
    } else {
        item_label(item.kind)
    };
    let text = match here.len() {
        1 => label,
        many => format!("{label}{many}"),
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
    // Something under another can't be picked up, nor a curse once it is known.
    if !covered && (god || carried.may_remove(item.id, Remover::Wearer)) {
        slot.insert(Pressable);
    }
    if let Some(aura) = Aura::of(item) {
        // One cell here, whatever it takes up in a grid.
        let mask = match &picture {
            Some(look) => AuraMask::Picture(look.clone(), PICTURE_PIXELS_PER_CELL),
            None => AuraMask::Cells(Shape::rectangle(1, 1)),
        };
        let node = Node {
            width: percent(100),
            height: percent(100),
            ..default()
        };
        slot.with_child(aura_glow(aura, mask, node));
    }
    if let Some(look) = picture {
        let node = Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        };
        slot.with_child((
            Floating,
            ItemPicture {
                look,
                pixels: PICTURE_PIXELS_PER_CELL,
            },
            ImageNode::default().with_color(Color::WHITE.with_alpha(alpha)),
            node,
        ));
    }
    if item.set {
        let width = item_edge_width(item.quality);
        slot.with_child(set_edge(item.quality, width, width));
    }
    slot.with_child((
        BodyNode,
        cell,
        Text::new(text),
        ITEM_LABEL_FONT.font(),
        TextColor(ITEM_EDGE),
    ));
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
        let def = inventory.creature.def();
        commands.entity(entity).with_children(|rows| {
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
                            spawn_slot(row, inventory, &here, god.0);
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
