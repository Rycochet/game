use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use crate::dom::build::tests::{app, find};
use crate::dom::parts::Darkened;
use crate::dom::{Dom, OwnPointer, Template, document};
use crate::html;
use crate::widgets::layer_z;

static DARK: Template = html!(
    "dark",
    r#"<panel id="map" form="d" dark depth="2" hidden><p>Over the dark</p></panel>"#
);

fn darkening(app: &mut App) -> Entity {
    let panel = find(app, "d/map");
    app.world()
        .get::<Darkened>(panel)
        .expect("a dark panel is darkened")
        .0
}

#[test]
fn a_dark_panel_darkens_the_window_while_it_shows() {
    let mut app = app();
    let root = app.world_mut().spawn(document(&DARK)).id();
    app.update();
    let behind = darkening(&mut app);
    let world = app.world();
    assert!(world.get::<OwnPointer>(behind).is_some());
    assert_eq!(
        world.get::<GlobalZIndex>(behind),
        Some(&GlobalZIndex(layer_z(2)))
    );
    assert_eq!(world.get::<Visibility>(behind), Some(&Visibility::Hidden));
    assert!(world.get::<ChildOf>(behind).is_none(), "across the window");

    app.world_mut()
        .run_system_once(|mut dom: Dom| dom.set_flag("d/map", "hidden", false))
        .unwrap();
    app.update();
    let shown = app.world().get::<Visibility>(behind);
    assert_eq!(shown, Some(&Visibility::Inherited));

    app.world_mut().entity_mut(root).despawn();
    app.update();
    assert!(
        app.world().get_entity(behind).is_err(),
        "gone with its document"
    );
}
