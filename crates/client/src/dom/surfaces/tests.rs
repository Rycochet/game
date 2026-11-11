use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use crate::dom::build::tests::{app, find};
use crate::dom::parts::{AlwaysBars, ControlOf, Element};
use crate::dom::{Dom, Template, UiEvent, UiValue, document};
use crate::html;
use crate::widgets::ButtonPressed;

static SURFACES: Template = html!(
    "surfaces",
    r#"<form id="a">
        <panel id="moves" title="Moves" moveable left="40" top="60"><p id="in">Inside</p></panel>
        <panel title="Stays"><p>Fixed</p></panel>
        <dialog id="ask" title="A question" hidden>
            <p>Sure?</p>
            <footer><button id="yes">Yes</button></footer>
        </dialog>
        <scroll id="list" max-height="50" scrollbars="always"><p id="row">A row</p></scroll>
    </form>"#
);

fn built() -> (App, Entity) {
    let mut app = app();
    let root = app.world_mut().spawn(document(&SURFACES)).id();
    app.update();
    (app, root)
}

fn events(app: &mut App) -> Vec<(String, UiValue)> {
    let mut events = app.world_mut().resource_mut::<Messages<UiEvent>>();
    let said = events.drain();
    said.map(|event| (event.id.to_string(), event.value))
        .collect()
}

/// Whether `entity` is somewhere inside `holder`.
fn is_in(app: &App, entity: Entity, holder: Entity) -> bool {
    let mut at = entity;
    while let Some(parent) = app.world().get::<ChildOf>(at) {
        at = parent.parent();
        if at == holder {
            return true;
        }
    }
    false
}

#[test]
fn a_moving_panel_stands_apart_from_its_document() {
    let (mut app, root) = built();
    let panel = find(&mut app, "a/moves");
    assert!(app.world().get::<ChildOf>(panel).is_none());
    let inside = find(&mut app, "a/in");
    assert!(is_in(&app, inside, panel));
    assert!(!is_in(&app, inside, root));
}

#[test]
fn surfaces_go_with_their_document() {
    let (mut app, root) = built();
    let (panel, dialog) = (find(&mut app, "a/moves"), find(&mut app, "a/ask"));
    app.world_mut().entity_mut(root).despawn();
    app.update();
    assert!(app.world().get_entity(panel).is_err());
    assert!(app.world().get_entity(dialog).is_err());
    // Nobody closed them: nothing is said.
    assert!(events(&mut app).is_empty());
}

#[test]
fn a_closed_panel_says_so() {
    let (mut app, _) = built();
    let panel = find(&mut app, "a/moves");
    app.world_mut().entity_mut(panel).despawn();
    app.update();
    assert_eq!(events(&mut app), [("a/moves".to_owned(), UiValue::Closed)]);
}

#[test]
fn a_dialog_is_shown_and_hidden_by_its_attribute() {
    let (mut app, _) = built();
    let layer = find(&mut app, "a/ask");
    assert_eq!(
        app.world().get::<Visibility>(layer),
        Some(&Visibility::Hidden)
    );
    let show = |mut dom: Dom| dom.set_flag("a/ask", "hidden", false);
    app.world_mut().run_system_once(show).unwrap();
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(layer),
        Some(&Visibility::Inherited)
    );

    // A press outside it is told by its id.
    app.world_mut().write_message(ButtonPressed(layer));
    app.update();
    assert_eq!(events(&mut app), [("a/ask".to_owned(), UiValue::Dismissed)]);
}

#[test]
fn a_dialogs_footer_is_its_row_of_buttons() {
    let (mut app, _) = built();
    let layer = find(&mut app, "a/ask");
    let yes = find(&mut app, "a/yes");
    assert!(is_in(&app, yes, layer));
    let holder = app.world().get::<ControlOf>(yes).unwrap().0;
    let footer = app.world().get::<ChildOf>(holder).unwrap().parent();
    let tag = app.world().get::<Element>(footer).unwrap().tag;
    assert_eq!(tag, crate::dom::Tag::Footer);
}

#[test]
fn a_scroll_area_holds_what_is_written_in_it() {
    let (mut app, _) = built();
    let area = find(&mut app, "a/list");
    assert!(app.world().get::<AlwaysBars>(area).is_some());
    let row = find(&mut app, "a/row");
    assert!(is_in(&app, row, area));
    // Filled by its id, as anything else is.
    let add = |mut dom: Dom| dom.set_text("a/row", "Changed");
    app.world_mut().run_system_once(add).unwrap();
    app.update();
    let words = app.world().get::<Children>(row).unwrap()[0];
    assert_eq!(app.world().get::<Text>(words).unwrap().0, "Changed");
}

#[test]
fn a_panel_that_moves_needs_an_id() {
    let template = Template {
        name: "bad",
        source: "<panel moveable/>",
    };
    let error = template.checked().unwrap_err();
    assert_eq!(error.what, "`<panel>` needs an `id`");
}
