//! Change Creature's panel: how the change is made (for good, only the body for a
//! while, or as an illusion, not made yet), then a button for each kind of creature,
//! only working where there is ground to drop what the new body can't wear on.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::protocol::{ClientMessage, KindChange, LevelKind};

use crate::common::{CurrentDungeon, GameView};
use crate::connection::ToServer;
use crate::dom::{Dom, Template};
use crate::html;
use crate::widgets::{ButtonPressed, ButtonState, UiId};

/// How Change Creature changes the character: `Permanent` until another is chosen.
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct ChosenChange(KindChange);

/// Asks the server to make the character this kind of creature.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct CreatureButton(CreatureKind);

/// Chooses how the change is made; `None` for an illusion, which can't be chosen yet.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct ChangeButton(Option<KindChange>);

/// The panel's contents, as markup. Each kind of creature's button is a copy
/// of the template, numbered by the kind's place in `CreatureKind::ALL`.
pub(super) static CHANGE_CREATURE: Template = html!(
    "debug_menu.change_creature",
    r#"
<form id="debug.change_creature">
  <span nowrap>
    <button id="permanent">Permanent</button>
    <button id="temporary">Temporary</button>
    <button id="illusion" disabled>Illusion</button>
  </span>
  <p>Not on the world map or in the Hall</p>
  <div id="kinds" class="tight"/>
  <template id="kind"><button id="kind"/></template>
</form>
"#
);

/// What a built part of the panel is known by, from its id; the place for
/// the kinds is filled with a button for each.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId, dom: &mut Dom) {
    let kind = id.index().and_then(|at| CreatureKind::ALL.get(at));
    match (id.parts().1, kind) {
        ("permanent", _) => bound.insert(ChangeButton(Some(KindChange::Permanent))),
        ("temporary", _) => bound.insert(ChangeButton(Some(KindChange::Temporary))),
        ("illusion", _) => bound.insert(ChangeButton(None)),
        ("kind", Some(&kind)) => bound.insert(CreatureButton(kind)),
        ("kinds", _) => {
            for (at, kind) in CreatureKind::ALL.into_iter().enumerate() {
                dom.copy_template(
                    "debug.change_creature/kind",
                    "debug.change_creature/kinds",
                    at,
                );
                dom.set_text(&format!("debug.change_creature/kind.{at}"), kind.def().name);
            }
            bound
        }
        _ => bound,
    };
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
