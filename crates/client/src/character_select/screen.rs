//! The screen's parts: a full-screen layer with the grid of characters on the left
//! and the chosen one on the right, spawned once and shown while choosing.

use bevy::prelude::*;

use crate::common::CharacterSelect;
use crate::widgets::{
    ButtonState, ButtonStyle, INK, Pressable, TextSize, UiId, panel_title, spawn_button,
    spawn_filling_scroll_area, spawn_fullscreen_layer,
};

/// The form all its widgets' ids are in.
pub(super) const FORM: &str = "character_select";
/// The right-hand column, describing the chosen character.
const INFO_WIDTH: f32 = 380.0;

#[derive(Component)]
pub(super) struct SelectLayer;
/// Where the cells go, wrapping like words.
#[derive(Component)]
pub(super) struct Grid;
/// The chosen character's name, over what they are like.
#[derive(Component)]
pub(super) struct InfoName;
#[derive(Component)]
pub(super) struct InfoWords;
/// When the chosen character was made, at the bottom.
#[derive(Component)]
pub(super) struct InfoCreated;

/// What the buttons along the bottom right do.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum Button {
    Play,
    Delete,
    Cancel,
}

pub(super) fn spawn_screen(mut commands: Commands) {
    let content = Node {
        flex_direction: FlexDirection::Row,
        column_gap: px(24),
        padding: UiRect::all(px(24)),
        ..default()
    };
    let (layer, panel) = spawn_fullscreen_layer(&mut commands, 0, content);
    commands.entity(layer).insert(SelectLayer);
    commands.entity(panel).with_children(|panel| {
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: px(12),
                ..default()
            })
            .with_children(|left| {
                left.spawn((panel_title("Characters"), Node::default()));
                let area = spawn_filling_scroll_area(left);
                left.commands().entity(area).with_child((
                    Grid,
                    Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(12),
                        row_gap: px(12),
                        ..default()
                    },
                ));
            });
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                width: px(INFO_WIDTH),
                flex_shrink: 0.0,
                row_gap: px(12),
                ..default()
            })
            .with_children(spawn_info);
    });
}

/// The right: the chosen one's name, what they are like, when they were made, and
/// the buttons.
fn spawn_info(column: &mut ChildSpawnerCommands) {
    column.spawn((InfoName, panel_title(""), Node::default()));
    let words = || (Text::new(""), TextSize::Medium.font(), TextColor(INK));
    column.spawn((words(), InfoWords));
    // What is left pushes the rest to the bottom.
    column.spawn(Node {
        flex_grow: 1.0,
        ..default()
    });
    column.spawn((words(), InfoCreated));
    column
        .spawn(Node {
            justify_content: JustifyContent::End,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            for (label, button, style, name) in [
                ("Play", Button::Play, ButtonStyle::Main, "play"),
                ("Delete", Button::Delete, ButtonStyle::Danger, "delete"),
                ("Cancel", Button::Cancel, ButtonStyle::Plain, "cancel"),
            ] {
                let node = Node {
                    padding: UiRect::axes(px(16), px(8)),
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, label, TextSize::Large, node).insert((
                    button,
                    style,
                    Pressable,
                    UiId::new(FORM, name),
                ));
            }
        });
}

/// Shows the screen while choosing.
pub(super) fn show_screen(
    select: Res<CharacterSelect>,
    mut layer: Single<&mut Visibility, With<SelectLayer>>,
) {
    if !select.is_changed() {
        return;
    }
    layer.set_if_neq(if select.open {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}
