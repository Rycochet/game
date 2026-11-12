use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use crate::dom::build::tests::{built, find};
use crate::dom::parts::Words;
use crate::dom::{Dom, Length, Template, document};
use crate::html;
use crate::widgets::{Progress, UiId};

static LIST: Template = html!(
    "list",
    r#"<form id="a">
        <template id="row"><p id="name">A name</p></template>
        <div id="rows"/>
        <p id="status">Waiting</p>
        <progress id="bar" value="1" max="4">Some</progress>
    </form>"#
);

/// Asks for changes as a system would, and runs a frame.
fn change(app: &mut App, ask: fn(&mut Dom)) {
    let asked = app
        .world_mut()
        .run_system_once(move |mut dom: Dom| ask(&mut dom));
    asked.unwrap();
    app.update();
}

fn words_in(app: &App, entity: Entity) -> Vec<String> {
    let children = app.world().get::<Children>(entity);
    let children = children
        .map(|children| children.to_vec())
        .unwrap_or_default();
    let text = |child: &Entity| {
        let world = app.world();
        world.get::<Words>(*child)?;
        Some(world.get::<Text>(*child)?.0.clone())
    };
    children.iter().filter_map(text).collect()
}

#[test]
fn words_are_changed_by_id() {
    let mut app = built(&LIST);
    change(&mut app, |dom| dom.set_text("a/status", "Ready"));
    let status = find(&mut app, "a/status");
    assert_eq!(words_in(&app, status), ["Ready"]);
}

#[test]
fn attributes_are_set_and_cleared_by_id() {
    let mut app = built(&LIST);
    change(&mut app, |dom| {
        dom.set_flag("a/status", "hidden", true);
        dom.set("a/rows", "width", "50%");
        dom.set("a/bar", "value", "3");
    });
    let status = find(&mut app, "a/status");
    let rows = find(&mut app, "a/rows");
    let node = |app: &App, entity| app.world().get::<Node>(entity).unwrap().clone();
    assert_eq!(node(&app, status).display, Display::None);
    assert_eq!(node(&app, rows).width, percent(50));
    let world = app.world_mut();
    let mut bars = world.query::<&Progress>();
    assert_eq!(bars.single(world).unwrap().0, 0.75);

    change(&mut app, |dom| dom.set_flag("a/status", "hidden", false));
    assert_eq!(node(&app, status).display, Display::Flex);
}

#[test]
fn a_change_that_is_not_allowed_changes_nothing() {
    let mut app = built(&LIST);
    change(&mut app, |dom| {
        dom.set("a/rows", "width", "wide");
        dom.set("a/rows", "onclick", "go()");
        dom.set("a/nothing", "width", "4");
    });
    let rows = find(&mut app, "a/rows");
    let element = app.world().get::<crate::dom::Element>(rows).unwrap();
    assert_eq!(element.attrs.width, None::<Length>);
}

#[test]
fn copies_of_a_template_get_numbered_ids() {
    let mut app = built(&LIST);
    change(&mut app, |dom| {
        dom.copy_template("a/row", "a/rows", 0);
        dom.copy_template("a/row", "a/rows", 1);
        // Seen at once, as changes are made in order.
        dom.set_text("a/name.1", "Second");
    });
    let rows = find(&mut app, "a/rows");
    assert_eq!(app.world().get::<Children>(rows).unwrap().len(), 2);
    let (first, second) = (find(&mut app, "a/name.0"), find(&mut app, "a/name.1"));
    assert_eq!(words_in(&app, first), ["A name"]);
    assert_eq!(words_in(&app, second), ["Second"]);

    change(&mut app, |dom| dom.empty("a/rows"));
    assert!(
        app.world()
            .get::<Children>(rows)
            .is_none_or(|rows| rows.is_empty())
    );
}

#[test]
fn a_document_can_be_filled_in_the_frame_it_is_spawned() {
    let mut app = crate::dom::build::tests::app();
    let spawn = |mut commands: Commands, mut dom: Dom| {
        commands.spawn(document(&LIST));
        dom.set_text("a/status", "Filled");
    };
    app.world_mut().run_system_once(spawn).unwrap();
    app.update();
    let status = find(&mut app, "a/status");
    assert_eq!(words_in(&app, status), ["Filled"]);
    let world = app.world_mut();
    let mut ids = world.query::<&UiId>();
    assert_eq!(
        crate::widgets::duplicate_ids(ids.iter(world)),
        Vec::<String>::new()
    );
}

static PICKER: Template = html!(
    "picker",
    r#"<form id="b"><select id="pick"><option>One</option></select></form>"#
);

#[test]
fn a_selects_options_are_changed_and_its_value_chooses_among_them() {
    let mut app = built(&PICKER);
    change(&mut app, |dom| {
        let options = [("e", "Easy"), ("h", "Hard")];
        dom.set_options(
            "b/pick",
            options.map(|(value, words)| (value.to_owned(), words.to_owned())),
        );
        dom.set("b/pick", "value", "h");
    });
    let pick = find(&mut app, "b/pick");
    let dropdown = app.world().get::<crate::widgets::Dropdown>(pick).unwrap();
    assert_eq!(dropdown.options, ["Easy", "Hard"]);
    assert_eq!(dropdown.selected, 1);

    // None is refused, leaving what it had.
    change(&mut app, |dom| dom.set_options("b/pick", []));
    let dropdown = app.world().get::<crate::widgets::Dropdown>(pick).unwrap();
    assert_eq!(dropdown.options, ["Easy", "Hard"]);
}

static KEYS: Template = html!(
    "keys",
    r#"<form id="c">
        <template id="row"><tr><td id="name" fills/><td>key</td></tr></template>
        <table id="keys" class="striped"/>
        <div id="other"/>
    </form>"#
);

#[test]
fn rows_from_a_template_are_added_to_a_table() {
    use crate::dom::parts::InTable;
    let mut app = built(&KEYS);
    change(&mut app, |dom| {
        dom.copy_template("c/row", "c/keys", 0);
        dom.copy_template("c/row", "c/keys", 1);
        dom.set_text("c/name.1", "Second");
        // Rows go nowhere else.
        dom.copy_template("c/row", "c/other", 2);
    });
    let keys = find(&mut app, "c/keys");
    let table = InTable::Table {
        columns: 2,
        wide: Some(0),
    };
    assert_eq!(app.world().get::<InTable>(keys), Some(&table));
    let world = app.world_mut();
    let mut spots = world.query::<(&InTable, &Node, Has<BackgroundColor>)>();
    let mut rows = Vec::new();
    for (spot, node, tinted) in spots.iter(world) {
        if let InTable::Row { row, .. } = spot {
            assert_eq!(node.grid_column, GridPlacement::start_span(1, 2));
            assert_eq!(tinted, *row == 1);
            rows.push(*row);
        }
    }
    rows.sort();
    assert_eq!(rows, [0, 1]);
    let other = find(&mut app, "c/other");
    assert!(app.world().get::<Children>(other).is_none());
}

static NOTE: Template = html!("note", r#"<form id="n"><p id="says">Plain</p></form>"#);

#[test]
fn items_written_into_words_show_as_items_and_go_again() {
    use bevy::ecs::system::RunSystemOnce;
    use shared::id::Id;
    use shared::item_text::item_token;

    use crate::widgets::ItemText;

    let mut app = built(&NOTE);
    let says = find(&mut app, "n/says");
    let words = |app: &App| -> Entity { app.world().get::<Children>(says).unwrap()[0] };
    assert!(app.world().get::<ItemText>(words(&app)).is_none());

    let token = format!("Take {}", item_token(Id::for_test(3)));
    let written = token.clone();
    let write = move |mut dom: Dom| dom.set_text("n/says", written.clone());
    app.world_mut().run_system_once(write).unwrap();
    app.update();
    let shown = app
        .world()
        .get::<ItemText>(words(&app))
        .expect("items shown");
    assert_eq!(shown.0, token);
    assert_eq!(app.world().get::<Text>(words(&app)).unwrap().0, "");

    let plain = |mut dom: Dom| dom.set_text("n/says", "Plain again");
    app.world_mut().run_system_once(plain).unwrap();
    app.update();
    assert!(app.world().get::<ItemText>(words(&app)).is_none());
    assert_eq!(
        app.world().get::<Text>(words(&app)).unwrap().0,
        "Plain again"
    );
}
