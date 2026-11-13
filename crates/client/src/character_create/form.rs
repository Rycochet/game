//! What the screen's boxes, lists and Begin do.

use bevy::prelude::*;
use shared::sheet::Trait;

use super::draft::Draft;
use super::looks::{PatternPictures, spawn_looks};
use super::screen::{BeginButton, CancelButton, KindList, LooksBox, NameBox};
use crate::common::{
    CharacterChoice, CharacterRefused, IRONMAN_QUESTION, MakingCharacter, PlayDifficulty,
    Preferences,
};
use crate::connection::ToServer;
use crate::dom::Dom;
use crate::widgets::{
    ButtonPressed, ButtonState, ColourChosen, DropdownChosen, TextBox, TextEdited, TextFocus,
};

/// The list choosing one of the kind's looks.
#[derive(Component)]
pub(super) struct LookList(pub Trait);

/// Shows the screen while the server waits for a character, with the keyboard
/// in the name box: afresh each time, or, when the server turned the last one
/// down, with everything as it was and why under the name (owner, 2026-11-13).
#[allow(clippy::too_many_arguments)]
pub(super) fn show_screen(
    making: Res<MakingCharacter>,
    refused: Res<CharacterRefused>,
    play: Res<PlayDifficulty>,
    preferences: Res<Preferences>,
    mut draft: ResMut<Draft>,
    mut focus: ResMut<TextFocus>,
    mut name: Single<(Entity, &mut TextBox), With<NameBox>>,
    mut dom: Dom,
) {
    if !making.is_changed() {
        return;
    }
    show_refusal(refused.0.as_deref(), &mut name.1, &mut dom);
    if making.0 && refused.0.is_some() {
        // The boxes and lists still show the draft: only the name may have
        // been folded since.
        name.1.text.clone_from(&draft.name);
        focus.0 = Some(name.0);
    } else if making.0 {
        *draft = Draft {
            // The player's own, or the nearest the server allows.
            difficulty: play.allowed.closest(preferences.difficulty),
            ..Draft::default()
        };
        name.1.text.clear();
        focus.0 = Some(name.0);
        // The kinds that can be played, the first chosen.
        let kinds = Draft::playable().into_iter();
        let kinds = kinds.map(|kind| kind.def().name.to_owned());
        dom.set_options(
            "character_create/kind",
            kinds.map(|name| (name.clone(), name)),
        );
        dom.set("character_create/kind", "value", draft.kind.def().name);
        dom.set("character_create/gender", "value", draft.gender.name());
        dom.set_text("character_create/ironman_question", IRONMAN_QUESTION);
    }
    dom.set_flag("character_create/layer", "hidden", !making.0);
}

/// Why the server turned the name down, in red under it, the box edged in red;
/// or nothing.
fn show_refusal(reason: Option<&str>, name: &mut TextBox, dom: &mut Dom) {
    name.error = reason.is_some();
    dom.set_text("character_create/name_refused", reason.unwrap_or_default());
    dom.set_flag("character_create/name_refused", "hidden", reason.is_none());
}

/// Takes what is typed and chosen into the draft. Typing in the name puts
/// away why the last was turned down.
#[allow(clippy::too_many_arguments)]
pub(super) fn read_choices(
    mut edited: MessageReader<TextEdited>,
    mut chosen: MessageReader<DropdownChosen>,
    mut colours: MessageReader<ColourChosen>,
    mut names: Query<&mut TextBox, With<NameBox>>,
    kinds: Query<(), With<KindList>>,
    looks: Query<&LookList>,
    mut draft: ResMut<Draft>,
    mut dom: Dom,
) {
    for edit in edited.read() {
        if let Ok(mut name) = names.get_mut(edit.0) {
            if name.error {
                show_refusal(None, &mut name, &mut dom);
            }
            // Only what can be in a name: lookalikes made plain as they are
            // typed (the server folds and checks it again).
            let folded: String = shared::name::fold(&name.text)
                .chars()
                .take(shared::name::MAX_NAME)
                .collect();
            if folded != name.text {
                name.text.clone_from(&folded);
            }
            draft.name = folded;
        }
    }
    let picked = colours
        .read()
        .map(|&ColourChosen(picker, index)| (picker, index));
    let listed: Vec<(Entity, usize)> = chosen
        .read()
        .map(|&DropdownChosen(list, index)| (list, index))
        .chain(picked)
        .collect();
    for (list, index) in listed {
        if kinds.contains(list) {
            if let Some(&kind) = Draft::playable().get(index) {
                draft.choose_kind(kind);
            }
        } else if let Ok(look) = looks.get(list) {
            draft.appearance.0.insert(look.0, index as u8);
        }
    }
}

/// The kind's own looks to choose from, again whenever the kind or gender
/// changes.
pub(super) fn fill_looks(
    mut commands: Commands,
    draft: Res<Draft>,
    looks_box: Single<Entity, With<LooksBox>>,
    mut pictures: ResMut<PatternPictures>,
    mut images: ResMut<Assets<Image>>,
    mut shown_for: Local<Option<(shared::creature::CreatureKind, shared::gender::Gender)>>,
) {
    let now = (draft.kind, draft.gender);
    if *shown_for == Some(now) {
        return;
    }
    *shown_for = Some(now);
    commands
        .entity(*looks_box)
        .despawn_related::<Children>()
        .with_children(|looks| spawn_looks(looks, &draft, &mut pictures, &mut images));
}

/// Begin sends the character, once it has a name; the server's welcome follows.
/// Cancel goes back to choosing (or the startup menu), making nothing.
pub(super) fn press_begin(
    mut pressed: MessageReader<ButtonPressed>,
    mut begin: Single<(Entity, &mut ButtonState), With<BeginButton>>,
    cancel: Single<Entity, With<CancelButton>>,
    draft: Res<Draft>,
    mut making: ResMut<MakingCharacter>,
    mut to_server: MessageWriter<ToServer>,
    mut choices: MessageWriter<CharacterChoice>,
) {
    let state = if draft.ready() {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    begin.1.set_if_neq(state);
    for press in pressed.read() {
        if press.0 == begin.0 && draft.ready() {
            to_server.write(ToServer(draft.message()));
            making.0 = false;
        } else if press.0 == *cancel {
            choices.write(CharacterChoice::CancelNew);
        }
    }
}
