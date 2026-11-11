use shared::id::Id;
use shared::item::{Item, ItemKind};

use super::*;

fn app() -> App {
    let mut app = App::new();
    app.add_systems(Update, draw_object_slots);
    app
}

fn apple(n: u128) -> SlotThing {
    SlotThing {
        item: Item::new(Id::for_test(n), ItemKind::Apple),
        count: 1,
        faded: false,
        pressable: true,
    }
}

fn cells(app: &mut App) -> Vec<(Entity, ThingCell, bool)> {
    let mut cells = app
        .world_mut()
        .query::<(Entity, &ThingCell, Has<Pressable>)>();
    let found = cells.iter(app.world());
    found
        .map(|(entity, cell, pressable)| (entity, *cell, pressable))
        .collect()
}

#[test]
fn an_empty_slot_is_a_cell_and_a_filled_one_shows_its_thing() {
    let mut app = app();
    let slot = app.world_mut().spawn(SlotShown::default()).id();
    app.update();
    let node = app.world().get::<Node>(slot).unwrap();
    assert_eq!((node.width, node.height), (px(CELL), px(CELL)));
    assert_eq!(
        app.world().get::<BackgroundColor>(slot).unwrap().0,
        GRID_COLOUR
    );
    assert!(cells(&mut app).is_empty());

    app.world_mut().get_mut::<SlotShown>(slot).unwrap().thing = Some(apple(1));
    app.update();
    let drawn = cells(&mut app);
    // The thing and its letters, of this slot; the thing can be pressed.
    assert_eq!(drawn.len(), 2);
    assert!(drawn.iter().all(|(_, cell, _)| cell.grid == slot));
    assert!(
        drawn
            .iter()
            .all(|(_, cell, _)| cell.item == Id::for_test(1))
    );
    assert_eq!(drawn.iter().filter(|(.., pressable)| *pressable).count(), 1);
}

#[test]
fn an_aim_tints_the_slot_without_drawing_the_thing_again() {
    let mut app = app();
    let shown = SlotShown {
        look: SlotLook::Kept,
        thing: Some(apple(1)),
        ..default()
    };
    let slot = app.world_mut().spawn(shown).id();
    app.update();
    let before = cells(&mut app);

    app.world_mut().get_mut::<SlotShown>(slot).unwrap().aim = Some(false);
    app.update();
    assert_eq!(app.world().get::<BackgroundColor>(slot).unwrap().0, REFUSED);
    let after = cells(&mut app);
    let entities = |cells: &[(Entity, ThingCell, bool)]| -> Vec<Entity> {
        cells.iter().map(|(entity, ..)| *entity).collect()
    };
    assert_eq!(entities(&before), entities(&after));

    // Taken out: nothing left in it.
    let mut shown = app.world_mut().get_mut::<SlotShown>(slot).unwrap();
    shown.thing = None;
    shown.aim = None;
    app.update();
    assert!(cells(&mut app).is_empty());
}

#[test]
fn something_that_cant_be_taken_isnt_pressable() {
    let mut app = app();
    let thing = SlotThing {
        pressable: false,
        ..apple(1)
    };
    let shown = SlotShown {
        thing: Some(thing),
        ..default()
    };
    app.world_mut().spawn(shown);
    app.update();
    assert!(cells(&mut app).iter().all(|(.., pressable)| !pressable));
}
