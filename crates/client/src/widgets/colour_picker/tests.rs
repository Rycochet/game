use bevy::ecs::system::RunSystemOnce;

use super::*;

fn picker() -> ColourPicker {
    ColourPicker {
        swatches: vec![
            Swatch::Colour(Color::BLACK),
            Swatch::Colour(Color::WHITE),
            Swatch::Picture(Handle::default()),
        ],
        names: vec!["Black".into(), "White".into(), "Speckled".into()],
        selected: 0,
    }
}

/// An app with one picker in it: the app and the picker.
fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_message::<ButtonPressed>()
        .add_message::<ColourChosen>()
        .add_systems(Update, (press_swatches, show_swatches).chain());
    app.world_mut()
        .run_system_once(|mut commands: Commands| {
            commands.spawn(Node::default()).with_children(|parent| {
                spawn_colour_picker(parent, picker(), TextSize::Medium, Node::default());
            });
        })
        .unwrap();
    let picker = app
        .world_mut()
        .query_filtered::<Entity, With<ColourPicker>>()
        .single(app.world())
        .unwrap();
    (app, picker)
}

fn swatch(app: &mut App, index: usize) -> Entity {
    app.world_mut()
        .query::<(Entity, &SwatchOf)>()
        .iter(app.world())
        .find(|(_, swatch)| swatch.index == index)
        .map(|(entity, _)| entity)
        .unwrap()
}

fn state(app: &App, swatch: Entity) -> ButtonState {
    *app.world().get::<ButtonState>(swatch).unwrap()
}

#[test]
fn swatches_are_square_and_a_line_as_tall_as_a_dropdown() {
    let (mut app, _) = app();
    let first = swatch(&mut app, 0);
    let node = app.world().get::<Node>(first).unwrap();
    assert_eq!(node.width, node.height);
    assert_eq!(node.width, px(line_height(TextSize::Medium)));
}

#[test]
fn the_chosen_one_stands_out_and_a_press_chooses_another() {
    let (mut app, picker) = app();
    let (first, third) = (swatch(&mut app, 0), swatch(&mut app, 2));
    assert_eq!(state(&app, first), ButtonState::Active);
    assert_eq!(state(&app, third), ButtonState::Normal);

    app.world_mut().write_message(ButtonPressed(third));
    app.update();
    let chosen: Vec<ColourChosen> = app
        .world()
        .resource::<Messages<ColourChosen>>()
        .iter_current_update_messages()
        .copied()
        .collect();
    assert_eq!(chosen, [ColourChosen(picker, 2)]);
    assert_eq!(app.world().get::<ColourPicker>(picker).unwrap().selected, 2);
    assert_eq!(state(&app, first), ButtonState::Normal);
    assert_eq!(state(&app, third), ButtonState::Active);
}

#[test]
fn only_the_chosen_one_has_the_white_edge() {
    assert_eq!(chosen_edge(true), CHOSEN_EDGE);
    assert_eq!(chosen_edge(false), 0.0);
}
