use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::AccumulatedMouseScroll;

use super::*;
use crate::common::{ChosenServer, Session, TouchGesture};
use crate::debug_menu::DebugMenu;
use crate::minimap::FullMap;
use crate::widgets::ButtonPressed;

fn test_app(menu: SystemMenu) -> App {
    let mut app = App::new();
    app.add_message::<ButtonPressed>()
        .add_message::<AppExit>()
        .add_message::<crate::common::LoginChoice>()
        .add_message::<crate::common::OpenAccounts>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<AccumulatedMouseScroll>()
        .init_resource::<TouchGesture>()
        .init_resource::<DebugMenu>()
        .init_resource::<FullMap>()
        .insert_resource(Session::Playing)
        .init_resource::<ChosenServer>()
        .init_resource::<crate::common::PlayFor>()
        .init_resource::<crate::common::Preferences>()
        .insert_resource(menu);
    app
}

fn at_start() -> App {
    let mut app = test_app(SystemMenu::Closed);
    app.insert_resource(Session::AtStart);
    app
}

fn session(app: &App) -> Session {
    *app.world().resource::<Session>()
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
fn escape_is_left_for_the_debug_menu_the_full_map_and_skills() {
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
    assert_eq!(press(&mut app, Action::Credits), SystemMenu::Credits);
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

#[test]
fn the_startup_menu_stays_until_play() {
    let mut app = at_start();
    assert_eq!(press_escape(&mut app), SystemMenu::Menu);
    assert_eq!(press(&mut app, Action::Back), SystemMenu::Menu);
    assert_eq!(press(&mut app, Action::Settings), SystemMenu::Settings);
    assert_eq!(press_escape(&mut app), SystemMenu::Menu);
    assert_eq!(press_escape(&mut app), SystemMenu::Menu);
    assert_eq!(session(&app), Session::AtStart);
    assert_eq!(press(&mut app, Action::Play), SystemMenu::Closed);
    assert_eq!(session(&app), Session::Playing);
}

#[test]
fn logging_out_goes_back_to_the_startup_menu() {
    let mut app = test_app(SystemMenu::Menu);
    // Play does nothing while playing.
    assert_eq!(press(&mut app, Action::Play), SystemMenu::Menu);
    assert_eq!(session(&app), Session::Playing);
    assert_eq!(press(&mut app, Action::LogOut), SystemMenu::Menu);
    assert_eq!(session(&app), Session::AtStart);
    assert_eq!(press_escape(&mut app), SystemMenu::Menu);
}

#[test]
fn characters_connects_to_choose_and_play_to_play_the_latest() {
    use crate::common::PlayFor;
    let mut app = at_start();
    press_escape(&mut app);
    assert_eq!(press(&mut app, Action::Characters), SystemMenu::Closed);
    assert_eq!(session(&app), Session::Playing);
    assert_eq!(*app.world().resource::<PlayFor>(), PlayFor::Choose);

    let mut app = at_start();
    app.insert_resource(PlayFor::Choose);
    press_escape(&mut app);
    assert_eq!(press(&mut app, Action::Play), SystemMenu::Closed);
    assert_eq!(*app.world().resource::<PlayFor>(), PlayFor::Latest);
}

#[test]
fn dont_warn_again_keeps_the_slow_drawing_warning_away() {
    let mut app = at_start();
    app.insert_resource(SystemMenu::SlowDrawing);
    let button = app.world_mut().spawn(Action::QuietSlowDrawing).id();
    app.world_mut().write_message(ButtonPressed(button));
    app.world_mut().run_system_once(take_input).unwrap();
    assert_eq!(*app.world().resource::<SystemMenu>(), SystemMenu::Menu);
    let preferences = app.world().resource::<crate::common::Preferences>();
    assert!(preferences.quiet_slow_drawing);
}
