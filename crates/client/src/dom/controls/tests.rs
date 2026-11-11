use bevy::prelude::*;

use super::{UiEvent, UiValue};
use crate::dom::build::tests::{app, find};
use crate::dom::input::Click;
use crate::dom::parts::{Control, ControlOf, Element, LabelFor};
use crate::dom::{Dom, Template, document};
use crate::html;
use crate::widgets::{
    ButtonPressed, ButtonState, Dropdown, DropdownChosen, SliderMoved, SwitchFlipped, TabChosen,
    UiId,
};

static CONTROLS: Template = html!(
    "controls",
    r#"<form id="a">
        <button id="go" class="main">Go</button>
        <label for="tick">Tick</label><input id="tick" type="checkbox" checked/>
        <label id="wraps">On <input id="on" type="checkbox" switch/></label>
        <input id="amount" type="range" min="0" max="10" step="1" value="4"/>
        <input id="name" type="text" value="Words"/>
        <select id="pick">
            <option value="x">First</option>
            <option selected>Second</option>
        </select>
        <toggle-row id="row"><option>One</option><option>Two</option></toggle-row>
        <key-box id="key" value="Shift+KeyG"/>
    </form>"#
);

/// An app with the controls built, and the messages widgets send.
fn built() -> App {
    let mut app = app();
    app.world_mut().spawn(document(&CONTROLS));
    app.update();
    app
}

/// What the controls said in the frame after `say` was sent.
fn said<M: Message>(app: &mut App, say: M) -> Vec<(String, UiValue)> {
    app.world_mut().write_message(say);
    app.update();
    let mut events = app.world_mut().resource_mut::<Messages<UiEvent>>();
    let said = events.drain();
    said.map(|event| (event.id.to_string(), event.value))
        .collect()
}

/// The widget showing the control with this id, and its element.
fn control(app: &mut App, id: &str) -> (Entity, Entity) {
    let widget = find(app, id);
    let holder = app.world().get::<ControlOf>(widget).expect("a control").0;
    (widget, holder)
}

fn attr(app: &App, holder: Entity, name: &str) -> Option<String> {
    let element = app.world().get::<Element>(holder).unwrap();
    element.attrs.text(name).map(str::to_owned)
}

#[test]
fn a_controls_id_is_on_its_widget() {
    let mut app = built();
    let (widget, holder) = control(&mut app, "a/go");
    assert_eq!(app.world().get::<Control>(holder).unwrap().0, widget);
    assert!(app.world().get::<UiId>(holder).is_none());
    let world = app.world_mut();
    let mut ids = world.query::<&UiId>();
    assert!(crate::widgets::duplicate_ids(ids.iter(world)).is_empty());
}

#[test]
fn a_pressed_button_says_so_by_its_id() {
    let mut app = built();
    let (widget, _) = control(&mut app, "a/go");
    let events = said(&mut app, ButtonPressed(widget));
    assert_eq!(events, [("a/go".to_owned(), UiValue::Pressed)]);
}

#[test]
fn a_checkbox_ticks_itself_and_a_switch_says_how_it_is() {
    let mut app = built();
    let (tick, holder) = control(&mut app, "a/tick");
    assert_eq!(
        said(&mut app, ButtonPressed(tick))[0].1,
        UiValue::Flag(false)
    );
    assert_eq!(attr(&app, holder, "checked"), None);
    assert_eq!(
        said(&mut app, ButtonPressed(tick))[0].1,
        UiValue::Flag(true)
    );
    assert_eq!(attr(&app, holder, "checked"), Some(String::new()));

    let (switch, holder) = control(&mut app, "a/on");
    let events = said(&mut app, SwitchFlipped(switch, true));
    assert_eq!(events, [("a/on".to_owned(), UiValue::Flag(true))]);
    assert!(attr(&app, holder, "checked").is_some());
}

#[test]
fn a_slider_gives_a_value_on_its_steps() {
    let mut app = built();
    let (slider, holder) = control(&mut app, "a/amount");
    let events = said(&mut app, SliderMoved(slider, 0.68));
    assert_eq!(events[0].1, UiValue::Number(7.0));
    assert_eq!(attr(&app, holder, "value").as_deref(), Some("7"));
}

#[test]
fn a_choice_gives_its_place_and_its_value() {
    let mut app = built();
    let (pick, holder) = control(&mut app, "a/pick");
    assert_eq!(app.world().get::<Dropdown>(pick).unwrap().selected, 1);
    let events = said(&mut app, DropdownChosen(pick, 0));
    let first = UiValue::Choice {
        index: 0,
        value: "x".into(),
    };
    assert_eq!(events[0].1, first);
    assert_eq!(attr(&app, holder, "value").as_deref(), Some("x"));

    let (row, _) = control(&mut app, "a/row");
    let events = said(&mut app, TabChosen(row, 1));
    let two = UiValue::Choice {
        index: 1,
        value: "Two".into(),
    };
    assert_eq!(events, [("a/row".to_owned(), two)]);
}

#[test]
fn attributes_set_by_id_reach_the_widget() {
    let mut app = built();
    let change = |mut dom: Dom| {
        dom.set_flag("a/go", "disabled", true);
        dom.set("a/pick", "value", "x");
        dom.set_text("a/go", "Went");
    };
    use bevy::ecs::system::RunSystemOnce;
    app.world_mut().run_system_once(change).unwrap();
    app.update();
    let (go, _) = control(&mut app, "a/go");
    assert_eq!(
        app.world().get::<ButtonState>(go),
        Some(&ButtonState::Disabled)
    );
    let (pick, _) = control(&mut app, "a/pick");
    assert_eq!(app.world().get::<Dropdown>(pick).unwrap().selected, 0);
    let label = app.world().get::<Children>(go).unwrap()[0];
    assert_eq!(app.world().get::<Text>(label).unwrap().0, "Went");
}

#[test]
fn a_click_on_a_label_presses_what_it_is_for() {
    let mut app = built();
    let (tick, _) = control(&mut app, "a/tick");
    let world = app.world_mut();
    // The label that names the checkbox with `for`.
    let mut labels = world.query_filtered::<Entity, With<LabelFor>>();
    let label = labels.single(world).unwrap();
    let click = Click {
        entity: label,
        at: Vec2::ZERO,
        count: 1,
    };
    world.trigger(click);
    app.update();
    let mut events = app.world_mut().resource_mut::<Messages<UiEvent>>();
    let said: Vec<UiEvent> = events.drain().collect();
    assert_eq!(said.len(), 1);
    assert_eq!(said[0].value, UiValue::Flag(false));
    let _ = tick;
}

#[test]
fn controls_are_checked_strictly() {
    let error = |source: &'static str| {
        let template = Template {
            name: "bad",
            source,
        };
        template.checked().unwrap_err().what
    };
    assert_eq!(
        error(r#"<form id="a"><input id="x"/></form>"#),
        "`<input>` needs a `type`"
    );
    assert_eq!(error("<button>Go</button>"), "`<button>` needs an `id`");
    assert_eq!(
        error(r#"<form id="a"><input id="x" type="radio"/></form>"#),
        "`type` is `text`, `checkbox`, `range`, `item`, `ability`, not `radio`"
    );
    assert_eq!(
        error(r#"<form id="a"><button id="x">Go <img src="icon:x"/></button></form>"#),
        "`<button>` can't hold `<img>`"
    );
}
