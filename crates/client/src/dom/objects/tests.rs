use shared::id::Id;
use shared::item::{Item, ItemKind};

use super::*;
use crate::dom::build::tests::{built, find};
use crate::dom::{Dom, Template};
use crate::html;

static THINGS: Template = html!(
    "things",
    r#"<form id="a"><object id="thing" type="item"/></form>"#
);

static WRONG: Template = html!(
    "wrong",
    r#"<form id="a"><object id="thing" type="text"/></form>"#
);
static ALSO_WRONG: Template = html!(
    "also_wrong",
    r#"<form id="a"><input id="thing" type="item"/></form>"#
);

fn children(app: &App, entity: Entity) -> usize {
    app.world()
        .get::<Children>(entity)
        .map_or(0, |held| held.len())
}

#[test]
fn an_object_shows_its_item_once_it_is_known() {
    use bevy::ecs::system::RunSystemOnce;
    let id = Id::for_test(7);
    let mut app = built(&THINGS);
    let thing = find(&mut app, "a/thing");
    // Naming nothing, it takes no room.
    app.update();
    assert_eq!(children(&app, thing), 0);

    let name = move |mut dom: Dom| dom.set("a/thing", "for", &id.to_string());
    app.world_mut().run_system_once(name).unwrap();
    app.update();
    // Asked after, and an empty cell meanwhile.
    assert_eq!(children(&app, thing), 1);
    let wanted = app.world_mut().resource_mut::<KnownItems>().take_wanted();
    assert!(wanted.contains(&id));

    let item = Item::new(id, ItemKind::Apple);
    let found = KnownItem::Found(Box::new(item.clone()), 0.0);
    app.world_mut().resource_mut::<KnownItems>().set(id, found);
    app.update();
    let shown = app.world().get::<ObjectShown>(thing).unwrap();
    assert_eq!(shown.0, Some(Drawn::Item(Box::new(item))));
}

#[test]
fn an_objects_type_is_its_own_and_only_its() {
    assert!(WRONG.checked().is_err());
    assert!(ALSO_WRONG.checked().is_err());
    THINGS.assert_good();
}
