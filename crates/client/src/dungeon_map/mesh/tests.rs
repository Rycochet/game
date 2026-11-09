use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use std::collections::BTreeSet;

use super::super::CELL_SIZE;
use super::super::theme::Shape;
use super::*;
use crate::dungeon_map::WALL_HEIGHT;

/// What each viewer sees from its cell.
fn views_from(level: &Level, cells: &[(i64, i64)]) -> Vec<View> {
    cells
        .iter()
        .map(|&(x, y)| {
            let mut sight = Sight::new(level);
            sight.add_view(level, x, y, None);
            View {
                cell: (x, y),
                sight,
            }
        })
        .collect()
}

/// The solid faces of a level seen by viewers, with the camera straight
/// above a point well to the south, so it sees everything facing south.
fn faces_seen(level: &Level, cells: &[(i64, i64)]) -> Vec<WallFace> {
    let views = views_from(level, cells);
    let mut sight = Sight::new(level);
    for view in &views {
        sight.add_view(level, view.cell.0, view.cell.1, None);
    }
    let light = LightGrid::new(level);
    let camera = Vec3::new(0.0, 600.0, 2000.0);
    build(
        level,
        &views,
        &sight,
        &sight,
        &light,
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        camera,
        Shape::Built,
        &Sight::new(level),
        None,
    )
    .solid_faces
}

/// A row of wall with floor all round it, in a big room.
fn room_with_wall() -> Level {
    let mut level = Level::new(-8, -8, 17, 17);
    for y in -7..=7 {
        for x in -7..=7 {
            level.set(x, y, Cell::Floor);
        }
    }
    for x in -2..=2 {
        level.set(x, 0, Cell::Wall);
    }
    level
}

#[test]
fn only_the_side_of_a_wall_facing_a_viewer_is_drawn() {
    let level = room_with_wall();
    let thin = CELL_SIZE * 0.05;
    // From the south, the wall's south faces show.
    let on_wall = |face: &&WallFace| face.centre.z.abs() < CELL_SIZE && face.centre.x.abs() < 24.0;
    let south = faces_seen(&level, &[(0, 3)]);
    let south: Vec<_> = south.iter().filter(on_wall).collect();
    assert!(!south.is_empty());
    for face in south {
        assert_eq!(face.normal, Vec3::Z);
        assert!((face.centre.z - thin).abs() < 0.01);
    }
    // From the north it shows none (those faces are turned from the
    // camera, so aren't solid).
    let north = faces_seen(&level, &[(0, -3)]);
    assert_eq!(north.iter().filter(on_wall).count(), 0);
}

/// A viewer in line with a wall sees both its sides, both see-through,
/// whichever way the camera is.
#[test]
fn walls_seen_side_on_are_in_the_way() {
    let mut level = room_with_wall();
    for x in -2..=2 {
        level.set(x, 0, Cell::Floor);
    }
    for y in -5..=-1 {
        level.set(0, y, Cell::Wall);
    }
    let views = views_from(&level, &[(0, 3)]);
    let sight = &views[0].sight;
    let light = LightGrid::new(&level);
    // East of the wall's line, so its east sides face the camera.
    let camera = Vec3::new(500.0, 600.0, 900.0);
    let shape = Shape::Built;
    let meshes = build(
        &level,
        &views,
        sight,
        sight,
        &light,
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        camera,
        shape,
        &Sight::new(&level),
        None,
    );
    // In the way: solid now (`cutaway`), but still not clicked on.
    assert!(!meshes.has_see_through);
    let on_the_wall = meshes
        .solid_faces
        .iter()
        .filter(|face| face.centre.x.abs() < CELL_SIZE)
        .filter(|face| (-6.0 * CELL_SIZE..-CELL_SIZE / 2.0).contains(&face.centre.z));
    assert_eq!(on_the_wall.count(), 0);
}

/// The far side of a column, or of a wall's end, shows from round the
/// corner: it is drawn as memory, not left as a hole.
#[test]
fn remembered_far_sides_are_drawn_as_memory() {
    let mut level = room_with_wall();
    for x in -2..=2 {
        level.set(x, 0, Cell::Floor);
    }
    level.set(0, 0, Cell::Wall);
    let views = views_from(&level, &[(0, 3)]);
    let sight = &views[0].sight;
    let light = LightGrid::new(&level);
    // North of the column, so its north side faces the camera.
    let camera = Vec3::new(0.0, 600.0, -900.0);
    let shape = Shape::Built;
    let north_side = |meshes: &DungeonMeshes| {
        meshes.memory[Surface::Wall as usize]
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|values| values.as_float3())
            .unwrap()
            .iter()
            .any(|p| (p[2] + CELL_SIZE * 0.05).abs() < 0.01 && p[1] < 0.01)
    };
    let meshes = build(
        &level,
        &views,
        sight,
        sight,
        &light,
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        camera,
        shape,
        &Sight::new(&level),
        None,
    );
    assert!(north_side(&meshes));
}

/// A room with a door in its south wall at `(0, 1)`, floor beyond it, seen
/// from just north of the door with the camera to the south.
fn through_a_door(views: &[View], sight: &Sight, level: &Level) -> DungeonMeshes {
    build(
        level,
        views,
        sight,
        sight,
        &LightGrid::new(level),
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        Vec3::new(0.0, 600.0, 900.0),
        Shape::Built,
        &Sight::new(level),
        None,
    )
}

fn door_in_a_wall() -> Level {
    let mut level = room_with_wall();
    for x in -7..=7 {
        level.set(x, 0, Cell::Floor);
        level.set(x, 1, Cell::Wall);
    }
    level.set(0, 1, Cell::Door);
    level
}

fn has_point(mesh: &Mesh, point: [f32; 3]) -> bool {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|values| values.as_float3())
        .is_some_and(|positions| {
            positions
                .iter()
                .any(|p| (0..3).all(|i| (p[i] - point[i]).abs() < 0.01))
        })
}

/// Past a door, the far side of the wall beside it is out of sight, but the
/// floor there can be in sight: as memory the mask would hide it, leaving
/// a black hole.
#[test]
fn floor_in_sight_beside_a_far_side_is_lit() {
    let level = door_in_a_wall();
    let views = views_from(&level, &[(0, 0)]);
    let meshes = through_a_door(&views, &views[0].sight, &level);
    // Where the south side of the wall just east of the door meets the
    // door's cell.
    let foot = [CELL_SIZE / 2.0, 0.0, CELL_SIZE * 1.05];
    assert!(has_point(&meshes.solid[Surface::Floor as usize], foot));
    assert!(!has_point(&meshes.memory[Surface::Floor as usize], foot));
}

/// A wall only the real lines of sight reach is in sight for that viewer:
/// as memory the mask would hide it, leaving it invisible.
#[test]
fn wall_a_viewer_reaches_is_solid() {
    let level = room_with_wall();
    let mut sight = Sight::new(&level);
    // Lines reach the floor south of the wall's middle, and the wall.
    sight.reveal(&level, 0, 1);
    let views = [View {
        cell: (0, 3),
        sight: sight.clone(),
    }];
    let meshes = through_a_door(&views, &sight, &level);
    assert!(
        meshes
            .solid_faces
            .iter()
            .any(|face| { face.normal == Vec3::Z && face.centre.x.abs() < CELL_SIZE / 2.0 })
    );
}

/// The edge of sight can run along a wall's foot without reaching the open
/// cell beside it: that quarter of a wall in sight is still lit, to show
/// inside the edge.
#[test]
fn every_open_quarter_of_a_wall_in_sight_is_lit() {
    let level = room_with_wall();
    let mut sight = Sight::new(&level);
    // Reaches the wall at (0, 0), but not the floor south of its west half.
    sight.reveal(&level, 1, 1);
    let views = [View {
        cell: (1, 3),
        sight: sight.clone(),
    }];
    let meshes = through_a_door(&views, &sight, &level);
    assert!(meshes.solid_faces.iter().any(|face| {
        face.normal == Vec3::Z && (-CELL_SIZE / 2.0..0.0).contains(&face.centre.x)
    }));
}

/// Where remembered walls were drawn, for a viewer who was to the north
/// and has gone, with the camera to the south.
fn memory_seen_from_the_south(level: &Level) -> Vec<[f32; 3]> {
    let views = views_from(level, &[(0, -3)]);
    let remembered = &views[0].sight;
    let meshes = build(
        level,
        &[],
        &Sight::new(level),
        remembered,
        &LightGrid::new(level),
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        Vec3::new(0.0, 600.0, 900.0),
        Shape::Built,
        &Sight::new(level),
        None,
    );
    meshes.memory[Surface::Wall as usize]
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|values| values.as_float3())
        .unwrap()
        .to_vec()
}

/// A wall seen only from the north still shows with the camera south of
/// it, where the side seen is turned away: not as a hole.
#[test]
fn a_wall_known_from_one_side_shows_from_the_other() {
    let positions = memory_seen_from_the_south(&room_with_wall());
    let south_side = CELL_SIZE * 0.05;
    assert!(
        positions
            .iter()
            .any(|p| (p[2] - south_side).abs() < 0.01 && p[1] < 0.01)
    );
}

/// The same behind two rows of wall: nothing is open at their inner
/// corners, so what lies between them stays dark.
#[test]
fn a_thick_wall_stays_dark_inside() {
    let mut level = room_with_wall();
    for x in -2..=2 {
        level.set(x, 1, Cell::Wall);
    }
    let positions = memory_seen_from_the_south(&level);
    let middle = positions
        .iter()
        .filter(|p| p[0].abs() < CELL_SIZE && p[2] > 0.01);
    assert_eq!(middle.count(), 0);
}

/// The floor of a wall's cell, seen from the south, stops at the wall's
/// foot; only natural walls fade into the floor.
#[test]
fn floors_stop_at_the_foot_of_the_wall() {
    use super::super::walls::half_thickness;
    let level = room_with_wall();
    let views = views_from(&level, &[(0, 3)]);
    let sight = &views[0].sight;
    let light = LightGrid::new(&level);
    let camera = Vec3::new(0.0, 600.0, 900.0);
    for shape in [Shape::Built, Shape::Natural] {
        let meshes = build(
            &level,
            &views,
            sight,
            sight,
            &light,
            &[],
            &[],
            &BTreeSet::new(),
            &[],
            false,
            camera,
            shape,
            &Sight::new(&level),
            None,
        );
        let positions = meshes.solid[Surface::Floor as usize]
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|values| values.as_float3())
            .unwrap();
        let foot = half_thickness(shape, 0.0);
        // The wall's cell: x within -2..=2 and z on row 0.
        let in_cell = positions
            .iter()
            .filter(|p| p[0].abs() < 2.0 * CELL_SIZE && p[2].abs() < CELL_SIZE / 2.0 - 0.01);
        let nearest = in_cell.map(|p| p[2].abs()).fold(f32::MAX, f32::min);
        assert!((nearest - foot).abs() < 0.01, "{shape:?}: {nearest}");
        let blends = meshes.has_solid[Surface::WallBlend as usize];
        assert_eq!(blends, shape == Shape::Natural);
    }
}

/// The ground under a solid wall is filled, so the curve never shows the
/// background; see-through walls keep their dark band.
#[test]
fn the_ground_is_filled_only_under_solid_walls() {
    let level = room_with_wall();
    let views = views_from(&level, &[(0, 3)]);
    let sight = &views[0].sight;
    let light = LightGrid::new(&level);
    let shape = Shape::Natural;
    let filled = |camera: Vec3| {
        let meshes = build(
            &level,
            &views,
            sight,
            sight,
            &light,
            &[],
            &[],
            &BTreeSet::new(),
            &[],
            false,
            camera,
            shape,
            &Sight::new(&level),
            None,
        );
        let solid = &meshes.solid[Surface::Wall as usize];
        solid
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|values| values.as_float3())
            .is_some_and(|positions| {
                positions
                    .iter()
                    .any(|p| p[0].abs() < 0.01 && p[1].abs() < 0.01 && p[2].abs() < 0.01)
            })
    };
    assert!(filled(Vec3::new(0.0, 600.0, 900.0)));
    assert!(!filled(Vec3::new(0.0, 600.0, -900.0)));
}

#[test]
fn a_door_is_a_wall_with_a_hole() {
    let mut level = room_with_wall();
    level.set(0, 0, Cell::Door);
    let faces = faces_seen(&level, &[(0, 3)]);
    // Beside the hole and above it, not across the bottom of it.
    let at_door = |x: f32| {
        faces.iter().any(|face| {
            let across = (face.centre.x - x).abs() <= face.half_across;
            let up = (face.centre.y - 2.0).abs() <= face.half_up;
            face.normal == Vec3::Z && face.centre.z.abs() < CELL_SIZE && across && up
        })
    };
    assert!(at_door(CELL_SIZE * 0.45));
    assert!(!at_door(0.0));
    assert!(
        faces
            .iter()
            .any(|face| { face.centre.x.abs() < 0.01 && face.centre.y > WALL_HEIGHT * 0.78 })
    );
}

#[test]
fn a_window_has_a_taller_sill_than_lintel() {
    let mut level = room_with_wall();
    level.set(0, 0, Cell::Window);
    let faces = faces_seen(&level, &[(0, 3)]);
    let mut over_hole: Vec<_> = faces
        .iter()
        .filter(|face| face.centre.x.abs() < 0.01 && face.normal == Vec3::Z)
        .filter(|face| face.centre.z.abs() < CELL_SIZE)
        .collect();
    over_hole.sort_by(|a, b| a.centre.y.total_cmp(&b.centre.y));
    let [sill, lintel] = over_hole[..] else {
        panic!("{over_hole:?}");
    };
    assert!(sill.half_up > lintel.half_up);
}

#[test]
fn natural_walls_are_narrowest_half_way_up() {
    use super::super::walls::half_thickness;
    let middle = half_thickness(Shape::Natural, WALL_HEIGHT / 2.0);
    assert!((middle - CELL_SIZE * 0.05).abs() < 0.01);
    assert!(half_thickness(Shape::Natural, 0.0) > middle * 2.0);
    assert!(half_thickness(Shape::Natural, WALL_HEIGHT) > middle * 2.0);
    assert_eq!(half_thickness(Shape::Built, 0.0), middle);
}

#[test]
fn sides_face_the_camera_by_its_position() {
    // Camera south of the room, a little east of the middle.
    let camera = Vec3::new(20.0, 600.0, 900.0);
    let north_wall = Vec3::new(0.0, 12.0, -84.0);
    assert!(faces_camera(north_wall, Vec3::Z, camera));
    assert!(!faces_camera(-north_wall, Vec3::NEG_Z, camera));
    // A west-facing side east of the camera faces it, even though the
    // camera looks a little away from it...
    let east_wall = Vec3::new(180.0, 12.0, 0.0);
    assert!(faces_camera(east_wall, Vec3::NEG_X, camera));
    // ...until the camera passes its plane.
    assert!(!faces_camera(
        east_wall,
        Vec3::NEG_X,
        Vec3::new(200.0, 600.0, 900.0)
    ));
}

#[test]
fn light_fades_and_stops_at_walls() {
    let level = Level::test_room();
    let mut light = LightGrid::new(&level);
    light.add_light(&level, (0, 0), 6, Vec3::ONE);
    assert_eq!(light.at(0, 0), Vec3::ONE);
    assert!(light.at(3, 0).x < light.at(1, 0).x);
    assert!(light.at(6, 0).x > 0.0);
    assert_eq!(light.at(7, 0), Vec3::ZERO);
    // Behind the short wall.
    assert_eq!(light.at(-3, 0), Vec3::ZERO);
    // Lights add up but never go past full.
    light.add_light(&level, (1, 0), 6, Vec3::ONE);
    assert_eq!(light.at(0, 0), Vec3::ONE);
}

#[test]
fn only_what_is_seen_is_built() {
    let level = Level::test_room();
    let views = views_from(&level, &[(0, 0)]);
    let light = LightGrid::new(&level);
    let camera = Vec3::new(0.0, 600.0, 900.0);
    let sight = &views[0].sight;
    let shape = Shape::Built;
    let meshes = build(
        &level,
        &views,
        sight,
        sight,
        &light,
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        camera,
        shape,
        &Sight::new(&level),
        None,
    );
    assert!(meshes.has_solid.iter().any(|&has| has));
    // The south wall's inner sides face away from the camera: in the way,
    // so drawn solid (`cutaway`), not see-through.
    assert!(!meshes.has_see_through);
    assert!(!meshes.solid_faces.is_empty());
    // In a plain room everything is in sight, so nothing is only
    // remembered. (The short wall above has a far side, drawn as memory.)
    let mut room = Level::new(-4, -4, 9, 9);
    for y in -3..=3 {
        for x in -3..=3 {
            room.set(x, y, Cell::Floor);
        }
    }
    let room_views = views_from(&room, &[(0, 0)]);
    let room_sight = &room_views[0].sight;
    let room_light = LightGrid::new(&room);
    let plain = build(
        &room,
        &room_views,
        room_sight,
        room_sight,
        &room_light,
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        camera,
        shape,
        &Sight::new(&room),
        None,
    );
    assert!(plain.has_memory.iter().all(|&has| !has));

    // Cells the mask cuts have a remembered copy of what is lit, in the
    // same places, apart from raised tops.
    let cut = build(
        &room,
        &room_views,
        room_sight,
        room_sight,
        &room_light,
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        camera,
        shape,
        room_sight,
        None,
    );
    assert!(cut.has_memory[Surface::Floor as usize]);
    assert!(cut.has_memory[Surface::Wall as usize]);

    let nothing = Sight::new(&level);
    let meshes = build(
        &level,
        &[],
        &nothing,
        &nothing,
        &light,
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        camera,
        shape,
        &Sight::new(&level),
        None,
    );
    assert!(meshes.has_solid.iter().all(|&has| !has));
    assert!(!meshes.has_see_through);

    // Remembered, but out of sight: drawn in the grey meshes only.
    let meshes = build(
        &level,
        &[],
        &nothing,
        sight,
        &light,
        &[],
        &[],
        &BTreeSet::new(),
        &[],
        false,
        camera,
        shape,
        &Sight::new(&level),
        None,
    );
    assert!(meshes.has_solid.iter().all(|&has| !has));
    assert!(meshes.has_memory[Surface::Floor as usize]);
    assert!(meshes.has_memory[Surface::Wall as usize]);
}
