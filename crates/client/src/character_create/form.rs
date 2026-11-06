//! What the screen's boxes, lists and Begin do.

use bevy::prelude::*;
use shared::sheet::Trait;

use super::draft::Draft;
use super::screen::{BeginButton, CreateLayer, KindList, LooksBox, NameBox, spawn_looks};
use crate::common::MakingCharacter;
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, ButtonState, DropdownChosen, TextBox, TextEdited, TextFocus};

/// The list choosing one of the kind's looks.
#[derive(Component)]
pub(super) struct LookList(pub Trait);

/// Shows the screen while the server waits for a character, starting afresh each
/// time, with the keyboard in the name box.
#[allow(clippy::type_complexity)]
pub(super) fn show_screen(
    making: Res<MakingCharacter>,
    mut draft: ResMut<Draft>,
    mut focus: ResMut<TextFocus>,
    mut layer: Single<&mut Visibility, With<CreateLayer>>,
    mut name: Single<(Entity, &mut TextBox), With<NameBox>>,
) {
    if !making.is_changed() {
        return;
    }
    if making.0 {
        *draft = Draft::default();
        name.1.text.clear();
        focus.0 = Some(name.0);
    }
    layer.set_if_neq(if making.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}

/// Takes what is typed and chosen into the draft.
pub(super) fn read_choices(
    mut edited: MessageReader<TextEdited>,
    mut chosen: MessageReader<DropdownChosen>,
    names: Query<&TextBox, With<NameBox>>,
    kinds: Query<(), With<KindList>>,
    looks: Query<&LookList>,
    mut draft: ResMut<Draft>,
) {
    for edit in edited.read() {
        if let Ok(name) = names.get(edit.0) {
            draft.name = name.text.clone();
        }
    }
    for &DropdownChosen(list, index) in chosen.read() {
        if kinds.contains(list) {
            if let Some(&kind) = Draft::playable().get(index) {
                draft.choose_kind(kind);
            }
        } else if let Ok(look) = looks.get(list) {
            draft.appearance.0.insert(look.0, index as u8);
        }
    }
}

/// The kind's own looks to choose from, again whenever the kind changes.
pub(super) fn fill_looks(
    mut commands: Commands,
    draft: Res<Draft>,
    looks_box: Single<Entity, With<LooksBox>>,
    mut shown_for: Local<Option<shared::creature::CreatureKind>>,
) {
    if *shown_for == Some(draft.kind) {
        return;
    }
    *shown_for = Some(draft.kind);
    commands
        .entity(*looks_box)
        .despawn_related::<Children>()
        .with_children(|looks| spawn_looks(looks, &draft));
}

/// Begin sends the character, once it has a name; the server's welcome follows.
pub(super) fn press_begin(
    mut pressed: MessageReader<ButtonPressed>,
    mut begin: Single<(Entity, &mut ButtonState), With<BeginButton>>,
    draft: Res<Draft>,
    mut making: ResMut<MakingCharacter>,
    mut to_server: MessageWriter<ToServer>,
) {
    let state = if draft.ready() {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    begin.1.set_if_neq(state);
    if pressed.read().any(|press| press.0 == begin.0) && draft.ready() {
        to_server.write(ToServer(draft.message()));
        making.0 = false;
    }
}
