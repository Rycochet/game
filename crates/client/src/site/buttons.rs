//! The Explore and Leave buttons at the bottom middle of the screen, shown
//! whenever pressing them would work (with or without touch), and their keys
//! (Shift + . and Shift + ,: `>` and `<` on UK and US keyboards).

use bevy::prelude::*;
use shared::protocol::{ClientMessage, LevelKind};
use shared::site::can_leave;

use crate::common::{Action, CurrentDungeon, GameView, MapPosition, Preferences};
use crate::connection::ToServer;
use crate::player::Player;
use crate::widgets::{ButtonPressed, ButtonState, Pressable, TextSize, UiId, spawn_button};

/// The buttons' widget form (named when they were only for the wilderness).
const FORM: &str = "wilderness";

/// The button to explore the wilderness from the world map.
#[derive(Component)]
pub(super) struct ExploreButton;

/// The button to leave the wilderness or a site for the world map.
#[derive(Component)]
pub(super) struct LeaveButton;

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
        spawn_button(
            row,
            ButtonState::Normal,
            "Explore",
            TextSize::Large,
            hidden(),
        )
        .insert((ExploreButton, Pressable, UiId::new(FORM, "explore")));
        spawn_button(row, ButtonState::Normal, "Leave", TextSize::Large, hidden()).insert((
            LeaveButton,
            Pressable,
            UiId::new(FORM, "leave"),
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

/// Explore (on the world map) or Leave (in the wilderness or a site), by key
/// or button.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn explore_or_leave(
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    player: Option<Single<&MapPosition, With<Player>>>,
    mut pressed: MessageReader<ButtonPressed>,
    mut explore: Single<(Entity, &mut Node), (With<ExploreButton>, Without<LeaveButton>)>,
    mut leave: Single<(Entity, &mut Node), With<LeaveButton>>,
    mut to_server: MessageWriter<ToServer>,
) {
    let on_world_map = *view.get() == GameView::WorldMap;
    let in_site = *view.get() == GameView::Dungeon;
    let can_leave = in_site && player.is_some_and(|at| can_leave_here(&dungeon, &at));
    let shown = |yes: bool| if yes { Display::Flex } else { Display::None };
    let show = |node: &mut Node, yes: bool| {
        if node.display != shown(yes) {
            node.display = shown(yes);
        }
    };
    show(&mut explore.1, on_world_map);
    show(&mut leave.1, can_leave);
    let presses: Vec<Entity> = pressed.read().map(|press| press.0).collect();
    let controls = &preferences.controls;
    if on_world_map
        && (controls.just_pressed(&keys, Action::Explore) || presses.contains(&explore.0))
    {
        to_server.write(ToServer(ClientMessage::Explore));
    }
    // The key in any site, whatever the button shows: the server decides
    // (a Dev Area is left from anywhere).
    if (in_site && controls.just_pressed(&keys, Action::Leave))
        || (can_leave && presses.contains(&leave.0))
    {
        to_server.write(ToServer(ClientMessage::Leave));
    }
}
