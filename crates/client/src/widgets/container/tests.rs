use super::*;

/// The parts of a frame with this look, spawned in a world.
fn spawned(look: ContainerLook) -> (World, ContainerParts) {
    let mut world = World::new();
    let mut parts = None;
    world
        .commands()
        .spawn(Node::default())
        .with_children(|parent| parts = Some(spawn_container(parent, look)));
    world.flush();
    (world, parts.expect("spawned"))
}

const CARRIED: ContainerLook = ContainerLook {
    title: None,
    controls: true,
    size: ContainerSize::Fixed,
    weight: true,
    slots: true,
};

#[test]
fn a_carried_grid_has_controls_weight_and_slots() {
    let (world, parts) = spawned(CARRIED);
    for part in [parts.frame, parts.grid] {
        assert!(world.get_entity(part).is_ok());
    }
    let weight = parts.weight.expect("a weight");
    assert!(world.get::<Text>(weight).is_some());
    assert!(parts.controls.is_some() && parts.slots.is_some());
}

#[test]
fn stock_shows_no_weight_and_grows_in_a_scroll_area() {
    let look = ContainerLook {
        controls: false,
        size: ContainerSize::Grows(Val::Px(300.0)),
        weight: false,
        slots: false,
        ..CARRIED
    };
    let (world, parts) = spawned(look);
    assert!(parts.weight.is_none() && parts.controls.is_none() && parts.slots.is_none());
    let grid = world.get::<Node>(parts.grid).expect("a node");
    assert_eq!(grid.max_height, Val::Px(300.0));
    // Only the grid is in the frame: no empty rows above or below it.
    let rows = world
        .get::<Children>(parts.frame)
        .map_or(0, |rows| rows.len());
    assert_eq!(rows, 1);
}
