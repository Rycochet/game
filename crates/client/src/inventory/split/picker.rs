//! The split box for a mixed stack (owner's design): a row for each kind of thing in
//! it, with a small picture keeping its glow, its name and how many, and a row for
//! all of it. Pressing a row picks those up together.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Item, Placed, Shape};

use crate::common::{NO_QUALITY_NAME, quality_colour};
use crate::item_art::{ItemPicture, look_of};
use crate::widgets::{
    Aura, AuraMask, CELL, Floating, PICTURE_PIXELS_PER_CELL, Pressable, TextSize, aura_glow,
    item_colour, item_edge, item_edge_width,
};

const GAP: f32 = 8.0;
const FONT: TextSize = TextSize::Medium;

/// On a row of the picker: the items it picks up, from the bottom up, or `None`
/// for all of the stack.
#[derive(Component, Clone, Debug)]
pub(in crate::inventory) struct PickRow(pub(in crate::inventory) Option<Vec<Id>>);

/// The rows for a mixed stack: all of it, then a row for each kind, its top's first.
pub(super) fn spawn_rows(inside: &mut ChildSpawnerCommands, placed: &Placed) {
    let stack = &placed.item;
    spawn_row(inside, PickRow(None), stack.top(), "All", stack.count());
    for group in stack.groups() {
        // From the bottom up, as a stack keeps them.
        let ids = group.iter().rev().map(|one| one.id).collect();
        spawn_row(
            inside,
            PickRow(Some(ids)),
            group[0],
            &group[0].name(),
            group.len() as u32,
        );
    }
}

fn spawn_row(
    inside: &mut ChildSpawnerCommands,
    row: PickRow,
    item: &Item,
    words: &str,
    count: u32,
) {
    let colour = item.quality.map_or(NO_QUALITY_NAME, quality_colour);
    let node = Node {
        width: percent(100),
        align_items: AlignItems::Center,
        column_gap: px(GAP),
        ..default()
    };
    inside.spawn((row, Pressable, node)).with_children(|line| {
        spawn_small(line, item);
        line.spawn((Text::new(words), FONT.font(), TextColor(colour)));
        let count = Text::new(format!("× {count}"));
        line.spawn((count, FONT.font(), TextColor(NO_QUALITY_NAME)));
    });
}

/// The item in one cell, whatever it takes up in a grid, with its glow.
fn spawn_small(line: &mut ChildSpawnerCommands, item: &Item) {
    let picture = look_of(item);
    let colour = match picture {
        Some(_) => Color::NONE,
        None => item_colour(item.kind),
    };
    let node = Node {
        width: px(CELL),
        height: px(CELL),
        flex_shrink: 0.0,
        border: UiRect::all(px(item_edge_width(item.quality))),
        ..default()
    };
    let edge = BorderColor::from(item_edge(item.quality));
    let mut square = line.spawn((node, BackgroundColor(colour), edge));
    let fill = || Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    if let Some(aura) = Aura::of(item) {
        let mask = match &picture {
            Some(look) => AuraMask::Picture(look.clone(), PICTURE_PIXELS_PER_CELL),
            None => AuraMask::Cells(Shape::rectangle(1, 1)),
        };
        square.with_child(aura_glow(aura, mask, fill()));
    }
    if let Some(look) = picture {
        let pixels = PICTURE_PIXELS_PER_CELL;
        square.with_child((
            Floating,
            ItemPicture { look, pixels },
            ImageNode::default(),
            fill(),
        ));
    }
}
