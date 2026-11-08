//! The Down and Up buttons at the bottom middle of the screen, shown whenever
//! pressing them would work (with or without touch), and their keys (Shift +
//! . and Shift + ,: `>` and `<` on UK and US keyboards). On stairs they read
//! Down or Up; otherwise Explore (on the world map) and Leave (to it). While
//! the player is fighting they are greyed and the keys do nothing: no changing
//! floors in a fight.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, LevelKind};
use shared::site::{StairsDirection, can_leave};

use crate::common::{Action, CurrentDungeon, GameView, MapPosition, OwnVitals, Preferences};
use crate::connection::ToServer;
use crate::player::Player;
use crate::widgets::{
    ButtonPressed, ButtonState, HoverTip, Pressable, TextSize, UiId, spawn_button,
};

/// The buttons' widget form (named when they were only for the wilderness).
const FORM: &str = "wilderness";
/// The buttons' tip while fighting.
const FIGHTING: &str = "Not while fighting";

/// The button to go down stairs, or explore the wilderness from the world map.
#[derive(Component)]
pub(super) struct DownButton;

/// The button to go up stairs, or leave the wilderness or a site for the
/// world map.
#[derive(Component)]
pub(super) struct UpButton;

/// The buttons, side by side at the bottom of the screen in the middle (for
/// now: the owner has better plans), hidden until they can be used.
pub(super) fn spawn_buttons(mut commands: Commands) {
    let row = Node {
        position_type: PositionType::Absolute,
        bottom: px(16.0),
        width: percent(100.0),
        justify_content: JustifyContent::Center,
        ..default()
    };
    commands.spawn(row).with_children(|row| {
        let hidden = || Node {
            display: Display::None,
            ..default()
        };
        let normal = ButtonState::Normal;
        spawn_button(row, normal, "Explore", TextSize::Large, hidden()).insert((
            DownButton,
            Pressable,
            HoverTip(String::new()),
            UiId::new(FORM, "down"),
        ));
        spawn_button(row, normal, "Leave", TextSize::Large, hidden()).insert((
            UpButton,
            Pressable,
            HoverTip(String::new()),
            UiId::new(FORM, "up"),
        ));
    });
}

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

/// A button's parts this changes.
type ButtonParts = (
    Entity,
    &'static mut Node,
    &'static mut ButtonState,
    &'static mut HoverTip,
    &'static Children,
);

/// One button as it should look: shown or not, its words, and greyed (with
/// a tip saying why) while fighting.
fn show(
    (node, state, tip, children): (&mut Node, &mut ButtonState, &mut HoverTip, &Children),
    texts: &mut Query<&mut Text>,
    (shown, label, fighting): (bool, &str, bool),
) {
    let display = if shown { Display::Flex } else { Display::None };
    if node.display != display {
        node.display = display;
    }
    let wanted = if fighting {
        ButtonState::Disabled
    } else {
        ButtonState::Normal
    };
    if *state != wanted {
        *state = wanted;
    }
    let words = if fighting { FIGHTING } else { "" };
    if tip.0 != words {
        tip.0 = words.to_string();
    }
    for &child in children {
        if let Ok(mut text) = texts.get_mut(child)
            && text.0 != label
        {
            text.0 = label.to_string();
        }
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
    mut down: Single<ButtonParts, (With<DownButton>, Without<UpButton>)>,
    mut up: Single<ButtonParts, With<UpButton>>,
    mut texts: Query<&mut Text>,
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
    let down_label = if on_down { "Down" } else { "Explore" };
    let up_label = if on_up { "Up" } else { "Leave" };
    let (d, u) = (&mut *down, &mut *up);
    let down_entity = d.0;
    let up_entity = u.0;
    let down_look = (down_shown, down_label, fighting);
    show((&mut d.1, &mut d.2, &mut d.3, d.4), &mut texts, down_look);
    show(
        (&mut u.1, &mut u.2, &mut u.3, u.4),
        &mut texts,
        (up_shown, up_label, fighting),
    );

    let presses: Vec<Entity> = pressed.read().map(|press| press.0).collect();
    if fighting {
        return;
    }
    let controls = &preferences.controls;
    // The keys whatever the buttons show: the server decides (a Dev Area is
    // left from anywhere).
    if ((on_world_map || in_site) && controls.just_pressed(&keys, Action::Down))
        || (down_shown && presses.contains(&down_entity))
    {
        to_server.write(ToServer(ClientMessage::Down));
    }
    if (in_site && controls.just_pressed(&keys, Action::Up))
        || (up_shown && presses.contains(&up_entity))
    {
        to_server.write(ToServer(ClientMessage::Up));
    }
}
