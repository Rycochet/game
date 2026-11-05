use bevy::ecs::system::RunSystemOnce;

use super::*;

/// A world with the waters' materials, a gate's water for each of `places`,
/// and these gates gone through; their materials given.
fn coloured(places: &[u32], visited: &[u32]) -> (World, Vec<Entity>) {
    let mut world = World::new();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.init_resource::<WaterMaterials>();
    world.insert_resource(VisitedGates(visited.iter().copied().collect()));
    let waters = places
        .iter()
        .map(|&place| world.spawn(GateWater(place)).id())
        .collect();
    world.run_system_once(colour_water).unwrap();
    (world, waters)
}

fn material_of(world: &World, water: Entity) -> Handle<StandardMaterial> {
    world
        .get::<MeshMaterial3d<StandardMaterial>>(water)
        .unwrap()
        .0
        .clone()
}

#[test]
fn gates_gone_through_have_orange_water_and_the_rest_blue() {
    let (world, waters) = coloured(&[7, 8], &[8]);
    let materials = world.resource::<WaterMaterials>();
    assert_eq!(material_of(&world, waters[0]), materials.blue);
    assert_eq!(material_of(&world, waters[1]), materials.orange);
}

#[test]
fn water_already_standing_turns_orange_when_gone_through() {
    let (mut world, waters) = coloured(&[7], &[]);
    world.resource_mut::<VisitedGates>().0.insert(7);
    world.run_system_once(colour_water).unwrap();
    let orange = world.resource::<WaterMaterials>().orange.clone();
    assert_eq!(material_of(&world, waters[0]), orange);
}

#[test]
fn the_two_waters_look_different() {
    let pixel = |visited| GateModel::water_picture(visited).data.unwrap()[..4].to_vec();
    let (blue, orange) = (pixel(false), pixel(true));
    assert!(blue[2] > blue[0], "blue: {blue:?}");
    assert!(orange[0] > orange[2], "orange: {orange:?}");
}
