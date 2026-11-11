use bevy::prelude::*;
use bevy::text::FontSize;

use crate::dom::parts::{InTable, Words};
use crate::dom::{Canvas, DomPlugin, Element, Inert, Tag, Template, document};
use crate::html;
use crate::widgets::{
    ButtonPressed, ChooseTab, ColourChosen, DropdownChosen, HoverTip, KeyChosen, SliderMoved,
    SwitchFlipped, TabChosen, TextEdited, TextFocus, UiId,
};

/// An app with just enough for documents to be built in.
pub(in crate::dom) fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), DomPlugin))
        .init_asset::<Image>()
        // What the input systems read.
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<bevy::input::mouse::AccumulatedMouseScroll>()
        .init_resource::<crate::common::UiPointer>()
        // What the widgets showing controls say, and are told.
        .add_message::<ButtonPressed>()
        .add_message::<SwitchFlipped>()
        .add_message::<SliderMoved>()
        .add_message::<TextEdited>()
        .add_message::<DropdownChosen>()
        .add_message::<TabChosen>()
        .add_message::<ChooseTab>()
        .add_message::<ColourChosen>()
        .add_message::<KeyChosen>()
        .init_resource::<TextFocus>();
    app
}

/// Builds `template` and gives the app back, a frame on.
pub(in crate::dom) fn built(template: &'static Template) -> App {
    let mut app = app();
    app.world_mut().spawn(document(template));
    app.update();
    app
}

/// The element with this id, as written.
pub(in crate::dom) fn find(app: &mut App, id: &str) -> Entity {
    let world = app.world_mut();
    let mut ids = world.query::<(Entity, &UiId)>();
    let found = ids.iter(world).find(|(_, known)| known.to_string() == id);
    found.unwrap_or_else(|| panic!("no element is `{id}`")).0
}

fn tags(app: &mut App) -> Vec<Tag> {
    let world = app.world_mut();
    let mut elements = world.query::<&Element>();
    elements.iter(world).map(|element| element.tag).collect()
}

static PLAIN: Template = html!(
    "plain",
    r#"<form id="a">
        <p id="words" title="A tip" inert>Some <small>small</small> words</p>
        <div id="gone" hidden><canvas id="box" width="8"/></div>
    </form>"#
);

#[test]
fn elements_are_built_with_their_ids_and_looks() {
    let mut app = built(&PLAIN);
    let words = find(&mut app, "a/words");
    let world = app.world();
    assert_eq!(world.get::<HoverTip>(words).unwrap().0, "A tip");
    assert!(world.get::<Inert>(words).is_some());
    assert_eq!(world.get::<Node>(words).unwrap().flex_wrap, FlexWrap::Wrap);
    // "Some", the small element, "words".
    assert_eq!(world.get::<Children>(words).unwrap().len(), 3);
    let gone = find(&mut app, "a/gone");
    assert_eq!(
        app.world().get::<Node>(gone).unwrap().display,
        Display::None
    );
    let canvas = find(&mut app, "a/box");
    assert!(app.world().get::<Canvas>(canvas).is_some());
    assert_eq!(app.world().get::<Node>(canvas).unwrap().width, px(8));
}

#[test]
fn words_take_the_size_of_what_they_are_in() {
    let mut app = built(&PLAIN);
    let world = app.world_mut();
    let mut words = world.query_filtered::<(&Text, &TextFont), With<Words>>();
    let mut size = |wanted: &str| {
        let found = words.iter(world).find(|(text, _)| text.0 == wanted);
        found.unwrap().1.font_size
    };
    assert_eq!(size("Some"), FontSize::Px(16.0));
    assert_eq!(size("small"), FontSize::Px(13.0));
}

static TABLE: Template = html!(
    "table",
    r#"<table class="striped">
        <tr><td>a</td><td>b</td><td>c</td></tr>
        <tr><td colspan="2">d</td><td>e</td></tr>
    </table>"#
);

#[test]
fn a_tables_rows_and_cells_are_placed_in_its_grid() {
    let mut app = built(&TABLE);
    let world = app.world_mut();
    let mut spots = world.query::<(&InTable, &Node, Has<BackgroundColor>)>();
    let (mut rows, mut cells) = (0, 0);
    for (spot, node, tinted) in spots.iter(world) {
        match spot {
            InTable::Table { columns } => assert_eq!(*columns, 3),
            InTable::Row { row, .. } => {
                rows += 1;
                assert_eq!(tinted, *row == 1);
                assert_eq!(node.grid_column, GridPlacement::start_span(1, 3));
            }
            InTable::Cell {
                row, column, span, ..
            } => {
                cells += 1;
                assert_eq!(node.grid_row, GridPlacement::start(*row as i16 + 1));
                let across = GridPlacement::start_span(*column as i16 + 1, *span);
                assert_eq!(node.grid_column, across);
            }
        }
    }
    assert_eq!((rows, cells), (2, 5));
}

static BROKEN: Template = html!("broken", "<form id='a'><blink/></form>");

#[test]
fn bad_markup_builds_nothing() {
    let mut app = built(&BROKEN);
    assert!(tags(&mut app).is_empty());
}
