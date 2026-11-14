use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::close_on_button;
use crate::dom::{DomPlugin, document};
use crate::minimap::FullMap;
use crate::minimap::markup::{FULL_MAP, bind_maps};
use crate::minimap::parts::MapPictures;
use crate::widgets::{ButtonPressed, ButtonState, Pressable, UiId};

/// The full map built from its markup by `dom`, with its places bound, open.
fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), DomPlugin))
        .init_asset::<Image>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<crate::widgets::PinnedTip>()
        .init_resource::<bevy::input::mouse::AccumulatedMouseScroll>()
        .init_resource::<crate::common::UiPointer>()
        .init_resource::<crate::widgets::TextFocus>()
        .add_message::<ButtonPressed>()
        .add_message::<crate::widgets::RaisePanel>()
        .add_message::<crate::widgets::SwitchFlipped>()
        .add_message::<crate::widgets::SliderMoved>()
        .add_message::<crate::widgets::TextEdited>()
        .add_message::<crate::widgets::DropdownChosen>()
        .add_message::<crate::widgets::TabChosen>()
        .add_message::<crate::widgets::ChooseTab>()
        .add_message::<crate::widgets::ColourChosen>()
        .add_message::<crate::widgets::KeyChosen>()
        .insert_resource(MapPictures {
            minimap: Handle::default(),
            full: Handle::default(),
            minimap_marks: Handle::default(),
            full_marks: Handle::default(),
        })
        .init_resource::<FullMap>()
        .add_systems(Update, (bind_maps, close_on_button).chain());
    app.world_mut().spawn(document(&FULL_MAP));
    app.update();
    app.update();
    app.world_mut().resource_mut::<FullMap>().set_open(true);
    app
}

fn find(app: &mut App, id: UiId) -> Entity {
    let world = app.world_mut();
    let mut ids = world.query::<(Entity, &UiId)>();
    let found = ids.iter(world).find(|(_, known)| **known == id);
    found.expect("the close button's place").0
}

/// `dom` takes a press on any element before `take_input` sees it, so the
/// X must close the map as a pressed button (the widgets' own search, or
/// `client.ui.press`), not by `take_input`'s.
#[test]
fn pressing_the_close_button_closes_the_map() {
    let mut app = app();
    let close = find(&mut app, UiId::new("full_map", "close"));
    assert!(app.world().get::<Pressable>(close).is_some());
    // The X drawn in it is a button's look, not a button of its own, so a
    // press on it reaches its place.
    let children = app.world().get::<Children>(close).expect("the X");
    let x = children[0];
    assert!(app.world().get::<ButtonState>(x).is_some());
    assert!(app.world().get::<Pressable>(x).is_none());

    app.world_mut().write_message(ButtonPressed(close));
    app.update();
    assert!(!app.world().resource::<FullMap>().open);
}

#[test]
fn other_buttons_leave_the_map_open() {
    let mut app = app();
    let other = app
        .world_mut()
        .spawn((Pressable, UiId::new("other", "close")))
        .id();
    app.world_mut().write_message(ButtonPressed(other));
    app.world_mut()
        .run_system_once(close_on_button)
        .expect("the system runs");
    assert!(app.world().resource::<FullMap>().open);
}
