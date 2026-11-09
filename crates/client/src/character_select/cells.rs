//! The grid's cells: each character standing to the same scale (`portrait_look`),
//! with their name and when they were last played under them, the latest first,
//! then a "+" to make another if the server allows.

use bevy::prelude::*;
use shared::protocol::{CharacterId, ListedCharacter};

use super::chosen::Chosen;
use super::screen::{FORM, Grid};
use crate::common::{CharacterSelect, local_date_time};
use crate::item_art::{ItemPicture, portrait_look};
use crate::widgets::{ButtonState, INK, Pressable, TextSize, UiId, spawn_button};

/// A portrait's width; its height is half as much again.
const PICTURE_WIDTH: f32 = 128.0;
const PICTURE_HEIGHT: f32 = PICTURE_WIDTH * 1.5;
/// A cell's padding round its picture and words.
const PADDING: f32 = 8.0;
/// The "+" cell's sign.
const PLUS_FONT: f32 = 64.0;
/// Words on a chosen (dark) cell: a button's lit edge.
const CHOSEN_TEXT: Color = Color::srgb(0.95, 0.87, 0.7);

/// Words in a cell, light while it is chosen.
#[derive(Component)]
pub(super) struct CellText;

/// What a cell stands for.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum Cell {
    Character(CharacterId),
    New,
}

/// The cells again whenever the list changes.
pub(super) fn fill_cells(
    mut commands: Commands,
    select: Res<CharacterSelect>,
    grid: Single<Entity, With<Grid>>,
) {
    if !select.is_changed() {
        return;
    }
    let sorted = select.sorted();
    commands
        .entity(*grid)
        .despawn_related::<Children>()
        .with_children(|grid| {
            for (index, listed) in sorted.into_iter().enumerate() {
                spawn_character(grid, listed, index);
            }
            if select.can_create {
                spawn_new(grid);
            }
        });
}

/// A cell's button, with its contents in a column.
fn spawn_cell<'a>(grid: &'a mut ChildSpawnerCommands, cell: Cell, id: UiId) -> EntityCommands<'a> {
    let node = Node {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Start,
        padding: UiRect::all(px(PADDING)),
        row_gap: px(4),
        width: px(PICTURE_WIDTH + 2.0 * PADDING),
        ..default()
    };
    let mut button = spawn_button(grid, ButtonState::Normal, "", TextSize::Small, node);
    // Its own contents instead of a label.
    button
        .despawn_related::<Children>()
        .insert((cell, Pressable, id));
    button
}

fn spawn_character(grid: &mut ChildSpawnerCommands, listed: &ListedCharacter, index: usize) {
    let summary = &listed.summary;
    let picture = portrait_look(listed.kind, &listed.appearance, &listed.worn);
    let id = UiId::new(FORM, "character").at(index);
    let mut cell = spawn_cell(grid, Cell::Character(summary.id), id);
    cell.with_children(|cell| {
        let node = Node {
            width: px(PICTURE_WIDTH),
            height: px(PICTURE_HEIGHT),
            ..default()
        };
        match picture {
            Some(look) => {
                let pixels = PICTURE_HEIGHT as u32;
                cell.spawn((ItemPicture { look, pixels }, ImageNode::default(), node));
            }
            None => {
                cell.spawn(node);
            }
        }
        cell.spawn((
            CellText,
            Text::new(summary.name.clone()),
            TextSize::Large.font(),
            TextColor(INK),
            TextLayout::justify(Justify::Center),
        ));
        let played = match summary.last_played {
            0 => "Never played".to_owned(),
            at => local_date_time(at),
        };
        cell.spawn((
            CellText,
            Text::new(played),
            TextSize::Small.font(),
            TextColor(INK),
            TextLayout::justify(Justify::Center),
        ));
    });
}

/// The last cell: a "+" as big as a portrait, to make another.
fn spawn_new(grid: &mut ChildSpawnerCommands) {
    let mut cell = spawn_cell(grid, Cell::New, UiId::new(FORM, "new"));
    // In the middle of the whole cell, as tall as the others in its row.
    cell.with_children(|cell| {
        cell.spawn(Node {
            width: px(PICTURE_WIDTH),
            min_height: px(PICTURE_HEIGHT),
            flex_grow: 1.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_child((
            Text::new("+"),
            TextFont::from_font_size(PLUS_FONT),
            TextColor(INK),
        ));
    });
}

/// The chosen character's cell looks chosen, its words light on it.
pub(super) fn show_chosen(
    chosen: Res<Chosen>,
    mut cells: Query<(&Cell, &mut ButtonState, &Children)>,
    mut words: Query<&mut TextColor, With<CellText>>,
) {
    for (cell, mut state, children) in &mut cells {
        let picked = matches!(cell, Cell::Character(id) if chosen.id == Some(*id));
        let (now, colour) = match picked {
            true => (ButtonState::Selected, CHOSEN_TEXT),
            false => (ButtonState::Normal, INK),
        };
        state.set_if_neq(now);
        for child in children {
            if let Ok(mut text) = words.get_mut(*child) {
                text.set_if_neq(TextColor(colour));
            }
        }
    }
}
