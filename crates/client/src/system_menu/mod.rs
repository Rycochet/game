//! The system menu: Esc or the gear button top right opens a small panel with
//! Settings, Logout and Quit, and while it is open single player is paused. See
//! `docs/context/client/system_menu.md`.

use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::protocol::ClientMessage;

use crate::common::{GameView, ServerClock, TouchGesture};
use crate::connection::ToServer;
use crate::debug_menu::DebugMenu;
use crate::minimap::FullMap;
use crate::widgets::{
    ButtonPressed, ButtonState, CLOSE_PIXELS, INK, Pressable, WidgetSystems, panel_title,
    spawn_button, spawn_close_button, spawn_icon_button, spawn_sized_layer, spawn_tabs,
};

const GEAR_BUTTON: f32 = 34.0;
const CORNER_GAP: f32 = 10.0;

const MENU_WIDTH: f32 = 260.0;
const MARGIN: f32 = 20.0;
/// Always there above the first button, for "Paused" to appear in, so nothing
/// moves when it does.
const PAUSED_ROW: f32 = 48.0;
const BUTTON_ROW: f32 = 38.0;
const GAP: f32 = 6.0;
const FONT: f32 = 18.0;
const SETTINGS_TITLE_ROW: f32 = 36.0;
/// Round the close button, still on it, for fingers.
const CLOSE_MARGIN: f32 = 4.0;
/// Settings takes as much of the window as the first layer does, though it is
/// above the menu (owner's choice: it needs the room).
const SETTINGS_SHARE: f32 = 90.0;

const MENU_LAYER: i32 = 0;
const ABOVE_MENU_LAYER: i32 = 1;

pub struct SystemMenuPlugin;

impl Plugin for SystemMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SystemMenu>()
            .add_systems(Startup, spawn_menu)
            .add_systems(
                PreUpdate,
                take_input
                    .in_set(WidgetSystems::Front)
                    .run_if(not(in_state(GameView::Connecting))),
            )
            .add_systems(Update, (show_menu, ask_for_pause));
    }
}

/// What is open.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemMenu {
    #[default]
    Closed,
    Menu,
    /// The menu, and above it the requester asking whether to quit.
    Confirming,
    /// The menu, and above it the settings.
    Settings,
}

impl SystemMenu {
    pub fn is_open(self) -> bool {
        self != Self::Closed
    }

    /// What is open after this.
    fn after(self, action: Action) -> Self {
        match (self, action) {
            (Self::Closed, Action::Open) => Self::Menu,
            (Self::Menu, Action::Back) => Self::Closed,
            (Self::Menu, Action::Settings) => Self::Settings,
            (Self::Menu, Action::Quit) => Self::Confirming,
            (Self::Confirming | Self::Settings, Action::Back) => Self::Menu,
            (same, _) => same,
        }
    }
}

/// What a press on this does. Each part can only be pressed while it is on top.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Open,
    /// Closes what is on top: its "No" or close button, or outside its panel.
    Back,
    Settings,
    Quit,
    ReallyQuit,
}

/// The gear button.
#[derive(Component)]
struct GearButton;

/// Which of the layers: shown while `SystemMenu` is one of these.
#[derive(Component, Clone, Copy)]
struct Layer(&'static [SystemMenu]);

#[derive(Component)]
struct PausedText;

/// A gear, drawn here so there is no file to ship for it.
fn gear_icon() -> Image {
    const SIZE: usize = 64;
    const TEETH: f32 = 8.0;
    const OUTSIDE: f32 = 29.0;
    const BETWEEN_TEETH: f32 = 22.0;
    const HOLE: f32 = 9.0;
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = SIZE as f32 / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (across, down) = (x as f32 + 0.5 - middle, y as f32 + 0.5 - middle);
            let distance = across.hypot(down);
            let turn = down.atan2(across) / std::f32::consts::TAU * TEETH;
            let reach = if turn.rem_euclid(1.0) < 0.5 {
                OUTSIDE
            } else {
                BETWEEN_TEETH
            };
            if (HOLE..=reach).contains(&distance) {
                bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&[60, 40, 20, 255]);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn spawn_menu(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let icon = images.add(gear_icon());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(CORNER_GAP),
                right: px(CORNER_GAP),
                ..default()
            },
            // Above the maps, as the warning button is.
            GlobalZIndex(15),
        ))
        .with_children(|holder| {
            let node = Node {
                width: px(GEAR_BUTTON),
                height: px(GEAR_BUTTON),
                ..default()
            };
            spawn_icon_button(holder, ButtonState::Normal, icon, node).insert((
                GearButton,
                Action::Open,
                Pressable,
            ));
        });
    spawn_main_panel(&mut commands);
    spawn_requester(&mut commands);
    spawn_settings(&mut commands);
}

fn wide_button() -> Node {
    Node {
        width: percent(100),
        height: px(BUTTON_ROW),
        ..default()
    }
}

/// A layer that presses outside its panel close.
fn spawn_layer(
    commands: &mut Commands,
    depth: i32,
    shown: &'static [SystemMenu],
    width: Val,
    height: Val,
    content: Node,
) -> Entity {
    let (layer, panel) = spawn_sized_layer(commands, depth, width, height, content);
    commands.entity(layer).insert((Layer(shown), Action::Back));
    panel
}

fn spawn_main_panel(commands: &mut Commands) {
    let panel = spawn_layer(
        commands,
        MENU_LAYER,
        &[
            SystemMenu::Menu,
            SystemMenu::Confirming,
            SystemMenu::Settings,
        ],
        px(MENU_WIDTH),
        Val::Auto,
        Node {
            padding: UiRect {
                left: px(MARGIN),
                right: px(MARGIN),
                top: px(0),
                bottom: px(MARGIN),
            },
            flex_direction: FlexDirection::Column,
            row_gap: px(GAP),
            ..default()
        },
    );
    commands.entity(panel).with_children(|panel| {
        panel
            .spawn(Node {
                height: px(PAUSED_ROW),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_child((PausedText, panel_title("Paused"), Visibility::Hidden));
        for (state, label, action) in [
            (ButtonState::Normal, "Settings", Some(Action::Settings)),
            // Back to choosing a character, once there is that to go back to.
            (ButtonState::Disabled, "Logout", None),
            (ButtonState::Normal, "Quit", Some(Action::Quit)),
        ] {
            let mut button = spawn_button(panel, state, label, FONT, wide_button());
            button.insert(Pressable);
            if let Some(action) = action {
                button.insert(action);
            }
        }
    });
}

fn spawn_requester(commands: &mut Commands) {
    let panel = spawn_layer(
        commands,
        ABOVE_MENU_LAYER,
        &[SystemMenu::Confirming],
        Val::Auto,
        Val::Auto,
        Node {
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(MARGIN),
            ..default()
        },
    );
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new("Are you sure?"),
            TextFont::from_font_size(FONT),
            TextColor(INK),
        ));
        panel
            .spawn(Node {
                column_gap: px(GAP),
                ..default()
            })
            .with_children(|row| {
                for (label, action) in [("Yes, quit", Action::ReallyQuit), ("No", Action::Back)] {
                    let node = Node {
                        width: px(MENU_WIDTH / 2.0 - GAP),
                        height: px(BUTTON_ROW),
                        ..default()
                    };
                    spawn_button(row, ButtonState::Normal, label, FONT, node)
                        .insert((action, Pressable));
                }
            });
    });
}

fn spawn_settings(commands: &mut Commands) {
    let panel = spawn_layer(
        commands,
        ABOVE_MENU_LAYER,
        &[SystemMenu::Settings],
        percent(SETTINGS_SHARE),
        percent(SETTINGS_SHARE),
        Node {
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(MARGIN),
            ..default()
        },
    );
    commands.entity(panel).with_children(|panel| {
        panel
            .spawn(Node {
                height: px(SETTINGS_TITLE_ROW),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|title| {
                title.spawn(panel_title("Settings"));
                title
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            right: px(-CLOSE_MARGIN),
                            top: px((SETTINGS_TITLE_ROW - CLOSE_PIXELS) / 2.0 - CLOSE_MARGIN),
                            padding: UiRect::all(px(CLOSE_MARGIN)),
                            ..default()
                        },
                        Action::Back,
                        Pressable,
                    ))
                    .with_children(|holder| {
                        spawn_close_button(holder, default());
                    });
            });
        // Nothing on its one page yet, so nothing listens for `TabChosen`.
        spawn_tabs(panel, &["Gameplay"], 0, FONT);
    });
}

/// After the text boxes and dropdowns have used what they want, and before the
/// debug menu, the full map and the game. While open, the menu has all input.
#[allow(clippy::too_many_arguments)]
fn take_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
    mut pressed: MessageReader<ButtonPressed>,
    actions: Query<&Action>,
    debug_menu: Res<DebugMenu>,
    full_map: Res<FullMap>,
    mut menu: ResMut<SystemMenu>,
    mut exit: MessageWriter<AppExit>,
) {
    let mut action = pressed
        .read()
        .filter_map(|press| actions.get(press.0).ok())
        .last()
        .copied();
    let escape = keys.just_pressed(KeyCode::Escape);
    if menu.is_open() {
        if escape {
            action = Some(Action::Back);
        }
        keys.reset_all();
        mouse.reset(MouseButton::Right);
        scroll.delta = Vec2::ZERO;
        *gesture = TouchGesture::default();
    } else if escape && !debug_menu.open && !full_map.open {
        // (Those two close on it instead, seeing it after this.)
        keys.clear_just_pressed(KeyCode::Escape);
        action = Some(Action::Open);
    }
    let Some(action) = action else {
        return;
    };
    if action == Action::ReallyQuit && *menu == SystemMenu::Confirming {
        exit.write(AppExit::Success);
        show_ended_page();
    } else {
        menu.set_if_neq(menu.after(action));
    }
}

/// Native: nothing, the window just goes. In a browser the page stays, so it
/// shows its own screen in `index.html`, with a button to load the game again.
fn show_ended_page() {
    #[cfg(target_arch = "wasm32")]
    {
        let screen = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("ended"));
        if let Some(screen) = screen {
            let _ = screen.remove_attribute("hidden");
        }
    }
}

/// Shows what is open.
#[allow(clippy::type_complexity)]
fn show_menu(
    menu: Res<SystemMenu>,
    view: Res<State<GameView>>,
    clock: Res<ServerClock>,
    mut layers: Query<(&Layer, &mut Visibility)>,
    mut gear: Single<&mut Visibility, (With<GearButton>, Without<Layer>, Without<PausedText>)>,
    mut paused: Single<&mut Visibility, (With<PausedText>, Without<Layer>)>,
) {
    let shown_if = |shown: bool| {
        if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for (layer, mut visibility) in &mut layers {
        visibility.set_if_neq(shown_if(layer.0.contains(&menu)));
    }
    gear.set_if_neq(shown_if(*view.get() != GameView::Connecting));
    // Only hidden, so its room stays and nothing moves.
    paused.set_if_neq(shown_if(clock.paused));
}

/// While the menu is open, asks a server that can pause to do so, and to carry on
/// once it has closed. The server's answer is what stops the client (`connection`).
fn ask_for_pause(
    menu: Res<SystemMenu>,
    clock: Res<ServerClock>,
    mut asked: Local<bool>,
    mut to_server: MessageWriter<ToServer>,
) {
    let wanted = menu.is_open() && clock.can_pause;
    if wanted != *asked {
        *asked = wanted;
        to_server.write(ToServer(ClientMessage::SetPaused(wanted)));
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    fn test_app(menu: SystemMenu) -> App {
        let mut app = App::new();
        app.add_message::<ButtonPressed>()
            .add_message::<AppExit>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseScroll>()
            .init_resource::<TouchGesture>()
            .init_resource::<DebugMenu>()
            .init_resource::<FullMap>()
            .insert_resource(menu);
        app
    }

    fn press_escape(app: &mut App) -> SystemMenu {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.world_mut().run_system_once(take_input).unwrap();
        *app.world().resource::<SystemMenu>()
    }

    fn press(app: &mut App, action: Action) -> SystemMenu {
        let entity = app.world_mut().spawn(action).id();
        app.world_mut().write_message(ButtonPressed(entity));
        app.world_mut().run_system_once(take_input).unwrap();
        *app.world().resource::<SystemMenu>()
    }

    fn exits(app: &App) -> usize {
        app.world()
            .resource::<Messages<AppExit>>()
            .iter_current_update_messages()
            .count()
    }

    #[test]
    fn escape_opens_it_and_closes_what_is_on_top() {
        let mut app = test_app(SystemMenu::Closed);
        assert_eq!(press_escape(&mut app), SystemMenu::Menu);
        // Taken: nothing after this sees it.
        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::Escape)
        );

        for on_top in [SystemMenu::Confirming, SystemMenu::Settings] {
            let mut app = test_app(on_top);
            assert_eq!(press_escape(&mut app), SystemMenu::Menu);
            assert_eq!(press_escape(&mut app), SystemMenu::Closed);
        }
    }

    #[test]
    fn escape_is_left_for_the_debug_menu_and_the_full_map() {
        let mut app = test_app(SystemMenu::Closed);
        app.world_mut().resource_mut::<DebugMenu>().open = true;
        assert_eq!(press_escape(&mut app), SystemMenu::Closed);
        assert!(
            app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::Escape)
        );

        let mut app = test_app(SystemMenu::Closed);
        app.world_mut().resource_mut::<FullMap>().open = true;
        assert_eq!(press_escape(&mut app), SystemMenu::Closed);
    }

    #[test]
    fn while_open_it_has_every_key() {
        let mut app = test_app(SystemMenu::Menu);
        for key in [KeyCode::Backquote, KeyCode::KeyM] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
        }
        app.world_mut().run_system_once(take_input).unwrap();
        let keys = app.world().resource::<ButtonInput<KeyCode>>();
        assert!(!keys.just_pressed(KeyCode::Backquote));
        assert!(!keys.just_pressed(KeyCode::KeyM));
    }

    #[test]
    fn the_buttons_go_where_they_say() {
        let mut app = test_app(SystemMenu::Closed);
        assert_eq!(press(&mut app, Action::Open), SystemMenu::Menu);
        assert_eq!(press(&mut app, Action::Settings), SystemMenu::Settings);
        assert_eq!(press(&mut app, Action::Back), SystemMenu::Menu);
        assert_eq!(press(&mut app, Action::Quit), SystemMenu::Confirming);
        assert_eq!(press(&mut app, Action::Back), SystemMenu::Menu);
        assert_eq!(press(&mut app, Action::Back), SystemMenu::Closed);
    }

    #[test]
    fn only_yes_quits() {
        let mut app = test_app(SystemMenu::Menu);
        press(&mut app, Action::Quit);
        assert_eq!(exits(&app), 0);
        press(&mut app, Action::Back);
        assert_eq!(exits(&app), 0);
        press(&mut app, Action::Quit);
        press(&mut app, Action::ReallyQuit);
        assert_eq!(exits(&app), 1);
    }
}
