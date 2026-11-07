//! The wilderness on the client: the keys (and, with touch on, buttons) to
//! explore it from the world map and to leave it, and keeping the place on the
//! world map (`WorldReturn`, for the weather and the way out) with the player.
//! The server sends it as a level like any site's, a window round the player
//! that `WildernessWindow` moves (`CurrentDungeon::move_window`).

use bevy::prelude::*;
use shared::protocol::ClientMessage;
use shared::wilderness::SCALE;

use crate::common::{
    Action, CurrentDungeon, CurrentWorld, GameView, MapPosition, Preferences, WorldReturn,
};
use crate::connection::ToServer;
use crate::player::Player;
use crate::widgets::{
    ButtonPressed, ButtonState, Pressable, TextSize, TouchOnly, UiId, spawn_button,
};

/// The touch buttons' widget form.
const FORM: &str = "wilderness";

/// The button to explore the wilderness from the world map.
#[derive(Component)]
pub(super) struct ExploreButton;

/// The button to leave it.
#[derive(Component)]
pub(super) struct LeaveButton;

/// The touch buttons, at the bottom of the screen in the middle (for now:
/// the owner has better plans).
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

/// Explore (on the world map) or Leave Wilderness (in it), by key or button.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn explore_or_leave(
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    touch_only: Res<TouchOnly>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut pressed: MessageReader<ButtonPressed>,
    mut explore: Single<(Entity, &mut Node), (With<ExploreButton>, Without<LeaveButton>)>,
    mut leave: Single<(Entity, &mut Node), With<LeaveButton>>,
    mut to_server: MessageWriter<ToServer>,
) {
    let on_world_map = *view.get() == GameView::WorldMap;
    let in_wilderness = *view.get() == GameView::Dungeon && dungeon.is_wilderness();
    let touch = preferences.touch || touch_only.0;
    let shown = |yes: bool| if yes { Display::Flex } else { Display::None };
    let show = |node: &mut Node, yes: bool| {
        if node.display != shown(yes) {
            node.display = shown(yes);
        }
    };
    show(&mut explore.1, touch && on_world_map);
    show(&mut leave.1, touch && in_wilderness);
    let presses: Vec<Entity> = pressed.read().map(|press| press.0).collect();
    let controls = &preferences.controls;
    if on_world_map
        && (controls.just_pressed(&keys, Action::Explore) || presses.contains(&explore.0))
    {
        to_server.write(ToServer(ClientMessage::Explore));
    }
    // In any site too: the server takes it as going back from a Dev Area, and
    // ignores it elsewhere.
    let in_site = *view.get() == GameView::Dungeon;
    if (in_site && controls.just_pressed(&keys, Action::LeaveWilderness))
        || (in_wilderness && presses.contains(&leave.0))
    {
        to_server.write(ToServer(ClientMessage::LeaveWilderness));
    }
}

/// In the wilderness, the place on the world map is the point the player is
/// in: the weather and the land's colours outside go by it, and leaving starts
/// from it.
pub(super) fn follow_point(
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    player: Single<&MapPosition, (With<Player>, Changed<MapPosition>)>,
    mut world_return: ResMut<WorldReturn>,
) {
    let Some(world) = world else {
        return;
    };
    if !dungeon.is_wilderness() {
        return;
    }
    let width = world.surface.width() * SCALE;
    let point = MapPosition {
        x: player.x.rem_euclid(width).div_euclid(SCALE),
        y: player.y.div_euclid(SCALE),
    };
    if world_return.0 != point {
        world_return.0 = point;
    }
}
