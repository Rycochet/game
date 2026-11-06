//! Change Creature's panel: how the change is made (for good, only the body for a
//! while, or as an illusion, not made yet), then a button for each kind of creature,
//! only working where there is ground to drop what the new body can't wear on.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::protocol::{ClientMessage, KindChange, LevelKind};

use super::{FONT, row, words};
use crate::common::{CurrentDungeon, GameView};
use crate::connection::ToServer;
use crate::widgets::{BUTTON_PADDING, ButtonPressed, ButtonState, Pressable, UiId, spawn_button};

/// How Change Creature changes the character: `Permanent` until another is chosen.
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct ChosenChange(KindChange);

/// Asks the server to make the character this kind of creature.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct CreatureButton(CreatureKind);

/// Chooses how the change is made; `None` for an illusion, which can't be chosen yet.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct ChangeButton(Option<KindChange>);

pub(super) fn spawn_creature_buttons(content: &mut ChildSpawnerCommands) {
    let node = || Node {
        padding: BUTTON_PADDING,
        ..default()
    };
    content.spawn(row()).with_children(|line| {
        for (label, how, name) in [
            ("Permanent", Some(KindChange::Permanent), "permanent"),
            ("Temporary", Some(KindChange::Temporary), "temporary"),
            ("Illusion", None, "illusion"),
        ] {
            spawn_button(line, ButtonState::Normal, label, FONT, node()).insert((
                ChangeButton(how),
                Pressable,
                UiId::new("debug.change_creature", name),
            ));
        }
    });
    content.spawn(words("Not on the world map or in the Hall"));
    for kind in CreatureKind::ALL {
        spawn_button(content, ButtonState::Normal, kind.def().name, FONT, node())
            .insert((CreatureButton(kind), Pressable));
    }
}

/// What Change Creature's buttons do.
pub(in crate::debug_menu) fn press_creature_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    kinds: Query<(&CreatureButton, &ButtonState)>,
    changes: Query<(&ChangeButton, &ButtonState)>,
    mut chosen: ResMut<ChosenChange>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok((&ChangeButton(Some(how)), _)) = changes.get(press.0) {
            chosen.0 = how;
        }
        if let Ok((&CreatureButton(kind), ButtonState::Normal)) = kinds.get(press.0) {
            let how = chosen.0;
            to_server.write(ToServer(ClientMessage::DevSetCreature { kind, how }));
        }
    }
}

/// The kinds work only where there is ground for what the new body can't wear: in a
/// site or the wilderness, not on the world map or in the Hall, as the server refuses
/// it there. The chosen way of changing shows as chosen; Illusion is never usable yet.
pub(in crate::debug_menu) fn update_creature_buttons(
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    chosen: Res<ChosenChange>,
    mut kinds: Query<&mut ButtonState, (With<CreatureButton>, Without<ChangeButton>)>,
    mut changes: Query<(&ChangeButton, &mut ButtonState), Without<CreatureButton>>,
) {
    let in_hall = dungeon
        .info
        .as_ref()
        .is_some_and(|info| matches!(info.kind, LevelKind::Hall { .. }));
    let has_ground = *view.get() == GameView::Dungeon && !in_hall;
    for mut state in &mut kinds {
        state.set_if_neq(if has_ground {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
    for (&ChangeButton(how), mut state) in &mut changes {
        state.set_if_neq(match how {
            None => ButtonState::Disabled,
            Some(how) if how == chosen.0 => ButtonState::Selected,
            Some(_) => ButtonState::Normal,
        });
    }
}
