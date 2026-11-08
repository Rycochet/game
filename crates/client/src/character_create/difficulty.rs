//! The screen's Difficulty dropdown: the difficulties the server allows a new
//! character, starting on the player's own (or the nearest allowed), disabled
//! when there is only one. Choosing Ironman asks first, as death is then
//! permanent; this is the only time it can be chosen.

use bevy::prelude::*;
use shared::difficulty::Difficulty;

use super::draft::Draft;
use super::screen::FORM;
use crate::common::{IRONMAN_QUESTION, PlayDifficulty, difficulty_words};
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonStyle, Dropdown, DropdownChosen, HoverTip, INK, Pressable,
    TextSize, UiId, field_label, spawn_button, spawn_dialog, spawn_dropdown,
};

/// Above the screen.
const REQUESTER_LAYER: i32 = 1;
const WORDS_WIDTH: f32 = 420.0;
const FONT: TextSize = TextSize::Medium;

/// Holds the dropdown, made again when what it offers changes.
#[derive(Component)]
pub(super) struct DifficultyBox;

/// The dropdown, with the difficulty each option is.
#[derive(Component)]
pub(super) struct DifficultyList(Vec<Difficulty>);

/// The requester asking whether to choose Ironman.
#[derive(Component)]
pub(super) struct IronmanRequester;

/// Its buttons: `true` for yes.
#[derive(Component)]
pub(super) struct IronmanAnswer(bool);

/// The label and the empty box the dropdown goes in (`fill_difficulty`).
pub(super) fn spawn_difficulty(column: &mut ChildSpawnerCommands) {
    column.spawn(field_label("Difficulty"));
    column.spawn((DifficultyBox, Node::default()));
}

pub(super) fn spawn_ironman_requester(mut commands: Commands) {
    let dialog = spawn_dialog(&mut commands, REQUESTER_LAYER, "Ironman");
    commands
        .entity(dialog.layer)
        .insert((IronmanRequester, Visibility::Hidden));
    commands.entity(dialog.content).with_child((
        Text::new(IRONMAN_QUESTION),
        FONT.font(),
        TextColor(INK),
        Node {
            max_width: px(WORDS_WIDTH),
            ..default()
        },
    ));
    commands.entity(dialog.buttons).with_children(|row| {
        for (label, yes, style, name) in [
            ("Yes, Ironman", true, ButtonStyle::Main, "ironman.yes"),
            ("No", false, ButtonStyle::Plain, "ironman.no"),
        ] {
            let node = Node {
                width: px(160.0),
                height: px(36.0),
                ..default()
            };
            spawn_button(row, ButtonState::Normal, label, FONT, node).insert((
                IronmanAnswer(yes),
                style,
                Pressable,
                UiId::new(FORM, name),
            ));
        }
    });
}

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
