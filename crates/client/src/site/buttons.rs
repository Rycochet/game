//! Down and Up: the movement pad's ladder buttons (`step_pad` spawns them)
//! and their keys (Shift + . and Shift + ,: `>` and `<` on UK and US
//! keyboards). The buttons are enabled whenever pressing them would work: on
//! stairs, or exploring from the world map and leaving to it. While the player
//! is fighting they are disabled and the keys do nothing: no changing floors
//! in a fight.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, LevelKind};
use shared::site::{StairsDirection, can_leave};

use crate::common::{
    Action, CurrentDungeon, GameView, MapPosition, OwnVitals, PadButton, Preferences,
};
use crate::connection::ToServer;
use crate::player::Player;
use crate::widgets::{ButtonPressed, ButtonState, HoverTip};

/// The buttons' tip while fighting.
const FIGHTING: &str = "Not while fighting";
/// Their tips when they would do nothing here.
const NO_DOWN: &str = "Down: on stairs going down, or Explore from the world map";
const NO_UP: &str = "Up: on stairs going up, or Leave where a way out is";

/// Whether the server would let the player leave from where they stand, as
/// far as the client can tell: in the wilderness from outside ground (the
/// server also refuses where the world map is water now), in a site where
/// `can_leave` says. A Dev Area's key works from anywhere, but its button
/// follows its building's rules.
fn can_leave_here(dungeon: &CurrentDungeon, at: &MapPosition) -> bool {
    let Some(info) = &dungeon.info else {
        return false;
    };
    match info.kind {
        LevelKind::Wilderness => dungeon.level.cell(at.x, at.y).is_outside(),
        LevelKind::Poi(kind) => can_leave(kind, info.index, &dungeon.level, at.x, at.y),
        LevelKind::Hall { .. } => false,
    }
}

/// One button as it should look: enabled or not, and its tip, which says
/// what it does now, or why it can't.
fn show(mut state: Mut<ButtonState>, mut tip: Mut<HoverTip>, (usable, words): (bool, &str)) {
    let wanted = if usable {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    state.set_if_neq(wanted);
    if tip.0 != words {
        tip.0 = words.to_string();
    }
}

/// Down (stairs down, or exploring from the world map) and Up (stairs up, or
/// leaving the wilderness or a site), by key or button; neither while
/// fighting.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn down_or_up(
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    (vitals, time): (Res<OwnVitals>, Res<Time>),
    player: Option<Single<&MapPosition, With<Player>>>,
    mut pressed: MessageReader<ButtonPressed>,
    mut buttons: Query<(Entity, &PadButton, &mut ButtonState, &mut HoverTip)>,
    mut to_server: MessageWriter<ToServer>,
) {
    let on_world_map = *view.get() == GameView::WorldMap;
    let in_site = *view.get() == GameView::Dungeon;
    let at = player.map(|at| **at).filter(|_| in_site);
    let stairs = at.and_then(|at| {
        let here = dungeon.stairs.iter().find(|s| (s.x, s.y) == (at.x, at.y));
        here.map(|stairs| stairs.direction)
    });
    let on_down = stairs == Some(StairsDirection::Down);
    let on_up = stairs == Some(StairsDirection::Up);
    let can_leave = at.is_some_and(|at| can_leave_here(&dungeon, &at));
    let fighting = vitals.fighting(time.elapsed_secs_f64());

    let (down_shown, up_shown) = (on_world_map || on_down, on_up || can_leave);
    let down_words = match (fighting, down_shown, on_down) {
        (true, ..) => FIGHTING,
        (_, false, _) => NO_DOWN,
        (_, true, true) => "Down",
        (_, true, false) => "Explore",
    };
    let up_words = match (fighting, up_shown, on_up) {
        (true, ..) => FIGHTING,
        (_, false, _) => NO_UP,
        (_, true, true) => "Up",
        (_, true, false) => "Leave",
    };
    let (mut down_entity, mut up_entity) = (None, None);
    for (entity, button, state, tip) in &mut buttons {
        match button {
            PadButton::Down => {
                down_entity = Some(entity);
                show(state, tip, (down_shown && !fighting, down_words));
            }
            PadButton::Up => {
                up_entity = Some(entity);
                show(state, tip, (up_shown && !fighting, up_words));
            }
            _ => {}
        }
    }

    let presses: Vec<Entity> = pressed.read().map(|press| press.0).collect();
    if fighting {
        return;
    }
    let controls = &preferences.controls;
    // The keys whatever the buttons show: the server decides (a Dev Area is
    // left from anywhere).
    if ((on_world_map || in_site) && controls.just_pressed(&keys, Action::Down))
        || (down_shown && down_entity.is_some_and(|e| presses.contains(&e)))
    {
        to_server.write(ToServer(ClientMessage::Down));
    }
    if (in_site && controls.just_pressed(&keys, Action::Up))
        || (up_shown && up_entity.is_some_and(|e| presses.contains(&e)))
    {
        to_server.write(ToServer(ClientMessage::Up));
    }
}
