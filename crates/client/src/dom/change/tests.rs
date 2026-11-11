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
