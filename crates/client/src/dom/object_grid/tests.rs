use shared::id::Id;
use shared::item::{Item, ItemKind, Rotation};

use super::*;

fn app() -> App {
    let mut app = App::new();
    app.add_systems(Update, draw_object_grids);
    app
}

fn apple(n: u128, x: u8) -> GridThing {
    GridThing {
        item: Item::new(Id::for_test(n), ItemKind::Apple),
        x,
        y: 0,
        rotation: Rotation::None,
        dimmed: false,
    }
}

fn children(app: &App, entity: Entity) -> usize {
    app.world()
        .get::<Children>(entity)
        .map_or(0, |held| held.len())
}

fn cells_of(app: &mut App, item: Id) -> usize {
    let mut cells = app.world_mut().query::<&ThingCell>();
    cells
        .iter(app.world())
        .filter(|cell| cell.item == item)
        .count()
}

#[test]
fn a_grid_draws_its_cells_and_what_is_in_it() {
    let mut app = app();
    let shown = GridShown {
        columns: 3,
        rows: 2,
        things: vec![apple(1, 0)],
        ..default()
    };
    let grid = app.world_mut().spawn((Node::default(), shown)).id();
    app.update();
    let GridBox(cells) = *app.world().get::<GridBox>(grid).unwrap();
    // Six cells' lines, and the layers for things and for the aim.
    assert_eq!(children(&app, cells), 3 * 2 + 2);
    assert!(cells_of(&mut app, Id::for_test(1)) > 0);
    let cell = app
        .world_mut()
        .query::<&ThingCell>()
        .iter(app.world())
        .next();
    assert_eq!(cell.unwrap().grid, grid);
}

#[test]
fn a_change_draws_only_what_changed() {
    let mut app = app();
    let shown = GridShown {
        columns: 3,
        rows: 2,
        things: vec![apple(1, 0)],
        ..default()
    };
    let grid = app.world_mut().spawn((Node::default(), shown)).id();
    app.update();
    let GridBox(first) = *app.world().get::<GridBox>(grid).unwrap();

    // Something else in it, and an aim: the same cells.
    let mut shown = app.world_mut().get_mut::<GridShown>(grid).unwrap();
    shown.things = vec![apple(2, 1)];
    shown.aim = Some(GridAim {
        cells: vec![(0, 0), (9, 9)],
        allowed: false,
    });
    app.update();
    let GridBox(second) = *app.world().get::<GridBox>(grid).unwrap();
    assert_eq!(first, second);
    assert_eq!(cells_of(&mut app, Id::for_test(1)), 0);
    assert!(cells_of(&mut app, Id::for_test(2)) > 0);
    // Only the aimed cell inside the grid is tinted.
    let drawn = app.world().get::<GridDrawn>(grid).unwrap();
    assert_eq!(children(&app, drawn.aim), 1);

    // Another size: new cells, with what is in it drawn in them.
    app.world_mut().get_mut::<GridShown>(grid).unwrap().rows = 4;
    app.update();
    let GridBox(third) = *app.world().get::<GridBox>(grid).unwrap();
    assert_ne!(second, third);
    assert!(app.world().get_entity(second).is_err());
    assert_eq!(children(&app, grid), 1);
    assert!(cells_of(&mut app, Id::for_test(2)) > 0);
}

#[test]
fn a_grid_taking_gifts_says_so_on_its_cells() {
    let mut app = app();
    let who = Id::for_test(9);
    let shown = GridShown {
        columns: 2,
        rows: 1,
        takes_gifts: Some(who),
        ..default()
    };
    app.world_mut().spawn((Node::default(), shown));
    app.update();
    let mut takers = app.world_mut().query::<&TakesGifts>();
    // The grid, its box of cells and both cells' lines.
    assert_eq!(takers.iter(app.world()).filter(|to| to.0 == who).count(), 4);
}
