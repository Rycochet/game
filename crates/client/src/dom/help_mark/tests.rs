use bevy::ecs::system::RunSystemOnce;

use super::*;
use crate::dom::Template;
use crate::dom::build::tests::{built, find};
use crate::dom::input::Hit;
use crate::html;
use crate::widgets::{Pinned, PinnedTip};

static ROWS: Template = html!(
    "help",
    r#"<form id="help">
  <span id="tipped" class="labelled" title="What this does.">
    <label for="first">First</label>
    <input id="first" type="checkbox" switch/>
  </span>
  <span id="plain" class="labelled">
    <label for="second">Second</label>
    <input id="second" type="checkbox" switch/>
  </span>
</form>"#
);

/// The "?" in a row's label, if it has one.
fn mark_of(app: &mut App, row: &str) -> Option<Entity> {
    let row = find(app, row);
    let world = app.world_mut();
    let mut marks = world.query::<(Entity, &HelpMark)>();
    let found = marks.iter(world).find(|(_, mark)| mark.0 == row);
    found.map(|(mark, _)| mark)
}

#[test]
fn only_rows_with_a_tip_have_a_mark_after_their_label() {
    let mut app = built(&ROWS);
    app.update();
    let mark = mark_of(&mut app, "help/tipped").expect("a mark");
    let label = app
        .world()
        .get::<ChildOf>(mark)
        .expect("in something")
        .parent();
    let element = app.world().get::<Element>(label).expect("an element");
    assert_eq!(element.tag, Tag::Label);
    assert!(mark_of(&mut app, "help/plain").is_none());
}

#[test]
fn a_tip_set_from_code_gains_a_mark_and_losing_it_loses_it() {
    let mut app = built(&ROWS);
    let plain = find(&mut app, "help/plain");
    app.world_mut()
        .entity_mut(plain)
        .insert(HoverTip("Now it says something.".into()));
    app.update();
    assert!(mark_of(&mut app, "help/plain").is_some());
    app.world_mut()
        .entity_mut(plain)
        .insert(HoverTip(String::new()));
    app.update();
    app.update();
    assert!(mark_of(&mut app, "help/plain").is_none());
}

#[test]
fn a_press_on_the_mark_pins_its_tip_and_any_other_press_lets_it_go() {
    let mut app = built(&ROWS);
    app.update();
    let mark = mark_of(&mut app, "help/tipped").expect("a mark");
    let row = find(&mut app, "help/tipped");
    let press = |app: &mut App, on: Option<Entity>| {
        app.world_mut().resource_mut::<Hit>().target = on;
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.release(MouseButton::Left);
        mouse.clear();
        mouse.press(MouseButton::Left);
        app.world_mut().run_system_once(pin_tips).expect("runs");
        app.world().resource::<PinnedTip>().0
    };
    let pinned = Pinned {
        tip_of: row,
        beside: mark,
    };
    assert_eq!(press(&mut app, Some(mark)), Some(pinned));
    assert_eq!(press(&mut app, Some(row)), None);
    assert_eq!(press(&mut app, Some(mark)), Some(pinned));
    // Pressing it again lets it go.
    assert_eq!(press(&mut app, Some(mark)), None);
}

#[test]
fn a_row_built_by_code_gets_its_mark_right_after_its_label() {
    let mut app = built(&ROWS);
    let world = app.world_mut();
    let row = world.spawn(Node::default()).id();
    let label = world.spawn((Text::new("Words"), ChildOf(row))).id();
    let control = world.spawn((Node::default(), ChildOf(row))).id();
    world
        .entity_mut(row)
        .insert((HoverTip("Why.".into()), HelpAfter(label)));
    app.update();
    let world = app.world_mut();
    let mut marks = world.query::<(Entity, &HelpMark)>();
    let (mark, _) = marks
        .iter(world)
        .find(|(_, mark)| mark.0 == row)
        .expect("a mark");
    let held: Vec<Entity> = world.get::<Children>(row).expect("children").to_vec();
    assert_eq!(held, vec![label, mark, control]);
}
