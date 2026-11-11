//! The screen's Difficulty dropdown: the difficulties the server allows a new
//! character, starting on the player's own (or the nearest allowed), disabled
//! when there is only one. Choosing Ironman asks first, as death is then
//! permanent; this is the only time it can be chosen.

use bevy::prelude::*;
use shared::difficulty::Difficulty;

use super::draft::Draft;
use super::screen::FORM;
use crate::common::{PlayDifficulty, difficulty_words};
use crate::widgets::{
    ButtonPressed, ButtonState, Dropdown, DropdownChosen, HoverTip, TextSize, UiId, spawn_dropdown,
};

const FONT: TextSize = TextSize::Medium;

/// Holds the dropdown, made again when what it offers changes.
#[derive(Component)]
pub(super) struct DifficultyBox;

/// The dropdown, with the difficulty each option is.
#[derive(Component)]
pub(super) struct DifficultyList(Vec<Difficulty>);

/// The requester asking whether to choose Ironman (in the screen's markup).
#[derive(Component)]
pub(super) struct IronmanRequester;

/// Its buttons: `true` for yes.
#[derive(Component)]
pub(super) struct IronmanAnswer(pub bool);

/// The dropdown's tip: what each difficulty means.
fn words(only: bool) -> String {
    let each: Vec<_> = Difficulty::ALL.into_iter().map(difficulty_words).collect();
    let first = if only {
        "This server allows only this difficulty."
    } else {
        "How hard the game is for this character. Easy, Normal and Hard can be changed \
         later, in Settings."
    };
    format!("{first}\n\n{}", each.join("\n"))
}

/// Makes the dropdown again when what it offers or shows changes, and as the
/// requester closes (putting back a choice not kept).
pub(super) fn fill_difficulty(
    mut commands: Commands,
    draft: Res<Draft>,
    play: Res<PlayDifficulty>,
    holder: Single<Entity, With<DifficultyBox>>,
    requester: Single<&Visibility, With<IronmanRequester>>,
    mut shown: Local<Option<(Vec<Difficulty>, Difficulty, bool)>>,
) {
    let options: Vec<_> = play.allowed.iter().collect();
    let asking = **requester != Visibility::Hidden;
    let now = (options.clone(), draft.difficulty, asking);
    if shown.as_ref() == Some(&now) {
        return;
    }
    *shown = Some(now);
    let only = options.len() < 2;
    let dropdown = Dropdown {
        options: options.iter().map(|d| d.name().to_owned()).collect(),
        selected: options
            .iter()
            .position(|&d| d == draft.difficulty)
            .unwrap_or(0),
        ..default()
    };
    commands
        .entity(*holder)
        .despawn_related::<Children>()
        .with_children(|holder| {
            let node = Node {
                width: percent(100),
                ..default()
            };
            let mut made = spawn_dropdown(holder, dropdown, FONT, node);
            made.insert((
                DifficultyList(options),
                HoverTip(words(only)),
                UiId::new(FORM, "difficulty"),
            ));
            if only {
                made.insert(ButtonState::Disabled);
            }
        });
}

/// A choice goes into the draft; Ironman only once the requester says yes.
pub(super) fn choose_difficulty(
    mut chosen: MessageReader<DropdownChosen>,
    mut pressed: MessageReader<ButtonPressed>,
    lists: Query<&DifficultyList>,
    answers: Query<&IronmanAnswer>,
    mut requester: Single<&mut Visibility, With<IronmanRequester>>,
    mut draft: ResMut<Draft>,
) {
    for &DropdownChosen(list, index) in chosen.read() {
        let Some(&difficulty) = lists.get(list).ok().and_then(|list| list.0.get(index)) else {
            continue;
        };
        if difficulty == Difficulty::Ironman && draft.difficulty != difficulty {
            requester.set_if_neq(Visibility::Inherited);
        } else {
            draft.difficulty = difficulty;
        }
    }
    for press in pressed.read() {
        if let Ok(&IronmanAnswer(yes)) = answers.get(press.0) {
            if yes {
                draft.difficulty = Difficulty::Ironman;
            }
            requester.set_if_neq(Visibility::Hidden);
        }
    }
}
