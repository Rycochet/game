//! "Delete <name>?": asked before a character is deleted, as it can't be undone.

use bevy::prelude::*;

use super::chosen::Chosen;
use super::screen::FORM;
use crate::common::CharacterSelect;
use crate::widgets::{
    ButtonState, ButtonStyle, INK, Pressable, TextSize, UiId, spawn_button, spawn_dialog,
};

/// Above the screen.
const LAYER: i32 = 1;
const WIDTH: f32 = 360.0;

#[derive(Component)]
pub(super) struct RequesterLayer;
#[derive(Component)]
pub(super) struct Question;

/// The requester's answers. A press outside it is No too.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum Answer {
    Delete,
    Keep,
}

pub(super) fn spawn_requester(mut commands: Commands) {
    let dialog = spawn_dialog(&mut commands, LAYER, "Delete character");
    commands
        .entity(dialog.layer)
        .insert((RequesterLayer, Answer::Keep));
    commands.entity(dialog.content).with_child((
        Question,
        Text::new(""),
        TextSize::Medium.font(),
        TextColor(INK),
        Node {
            width: px(WIDTH),
            ..default()
        },
    ));
    commands.entity(dialog.buttons).with_children(|row| {
        for (label, answer, style, name) in [
            (
                "Delete",
                Answer::Delete,
                ButtonStyle::Danger,
                "really_delete",
            ),
            ("Cancel", Answer::Keep, ButtonStyle::Plain, "keep"),
        ] {
            let node = Node {
                padding: UiRect::axes(px(16), px(8)),
                ..default()
            };
            spawn_button(row, ButtonState::Normal, label, TextSize::Large, node).insert((
                answer,
                style,
                Pressable,
                UiId::new(FORM, name),
            ));
        }
    });
}

/// Up while Delete is being asked about, naming the character.
pub(super) fn show_requester(
    chosen: Res<Chosen>,
    select: Res<CharacterSelect>,
    mut layer: Single<&mut Visibility, With<RequesterLayer>>,
    mut question: Single<&mut Text, With<Question>>,
) {
    if !chosen.is_changed() {
        return;
    }
    let name = chosen
        .id
        .and_then(|id| select.list.iter().find(|c| c.summary.id == id))
        .map(|c| c.summary.name.as_str());
    let asking = chosen.deleting && name.is_some();
    if let Some(name) = name {
        question.0 = format!("Delete {name}? This can't be undone.");
    }
    layer.set_if_neq(if asking {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}
