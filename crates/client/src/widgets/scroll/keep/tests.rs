use bevy::ecs::system::RunSystemOnce;

use super::*;

/// An area `height` tall, with `contents` in it, at a scale of 1.
fn laid_out(height: f32, contents: f32) -> ComputedNode {
    ComputedNode {
        size: Vec2::new(100.0, height),
        content_size: Vec2::new(100.0, contents),
        inverse_scale_factor: 1.0,
        ..default()
    }
}

fn world() -> World {
    let mut world = World::new();
    world.init_resource::<ScrollMemory>();
    world
}

fn run(world: &mut World) {
    world.run_system_once(keep_places).unwrap();
}

fn place(world: &World, area: Entity) -> f32 {
    world.get::<ScrollPosition>(area).unwrap().y
}

#[test]
fn shares_and_places_turn_into_each_other() {
    assert_eq!(place_of(0.5, 300.0), 150.0);
    assert_eq!(place_of(2.0, 300.0), 300.0);
    assert_eq!(share_of(150.0, 300.0, 0.0), 0.5);
    assert_eq!(share_of(0.0, 0.0, 0.25), 0.25);
}

#[test]
fn an_area_keeps_its_share_when_its_room_changes() {
    let mut world = world();
    let area = world
        .spawn((
            ScrollArea,
            laid_out(100.0, 500.0),
            ScrollPosition::default(),
        ))
        .id();
    run(&mut world);
    // Scrolled halfway by hand.
    world.get_mut::<ScrollPosition>(area).unwrap().y = 200.0;
    run(&mut world);
    // A bigger window: less room to scroll.
    *world.get_mut::<ComputedNode>(area).unwrap() = laid_out(300.0, 500.0);
    run(&mut world);
    assert_eq!(place(&world, area), 100.0);
}

#[test]
fn a_hidden_area_keeps_its_share() {
    let mut world = world();
    let area = world
        .spawn((
            ScrollArea,
            laid_out(100.0, 500.0),
            ScrollPosition::default(),
        ))
        .id();
    run(&mut world);
    world.get_mut::<ScrollPosition>(area).unwrap().y = 400.0;
    run(&mut world);
    *world.get_mut::<ComputedNode>(area).unwrap() = ComputedNode::default();
    world.get_mut::<ScrollPosition>(area).unwrap().y = 0.0;
    run(&mut world);
    *world.get_mut::<ComputedNode>(area).unwrap() = laid_out(100.0, 500.0);
    run(&mut world);
    assert_eq!(place(&world, area), 400.0);
}

#[test]
fn an_area_built_again_comes_back_to_its_place() {
    let mut world = world();
    let key = ScrollKey::new("test");
    let first = world
        .spawn((
            ScrollArea,
            key,
            laid_out(100.0, 500.0),
            ScrollPosition::default(),
        ))
        .id();
    run(&mut world);
    world.get_mut::<ScrollPosition>(first).unwrap().y = 300.0;
    run(&mut world);
    world.despawn(first);

    // Built again, not laid out yet: it starts at the old place, so its top
    // doesn't flash, then keeps its share of what it turns out to hold.
    let again = world
        .spawn((
            ScrollArea,
            key,
            ComputedNode::default(),
            ScrollPosition::default(),
        ))
        .id();
    world.run_system_once(bring_back_places).unwrap();
    assert_eq!(place(&world, again), 300.0);
    *world.get_mut::<ComputedNode>(again).unwrap() = laid_out(100.0, 900.0);
    run(&mut world);
    assert_eq!(place(&world, again), 600.0);
}

#[test]
fn an_area_never_seen_starts_at_the_top() {
    let mut world = world();
    world.resource_mut::<ScrollMemory>().0.insert(
        ScrollKey::new("other"),
        ScrollShare {
            share: 1.0,
            furthest: Some(400.0),
        },
    );
    let area = world
        .spawn((
            ScrollArea,
            ScrollKey::new("new"),
            laid_out(100.0, 500.0),
            ScrollPosition::default(),
        ))
        .id();
    world.run_system_once(bring_back_places).unwrap();
    run(&mut world);
    assert_eq!(place(&world, area), 0.0);
}
