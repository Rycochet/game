use bevy::prelude::*;
use shared::dungeon::{Cell, RoomId};
use shared::dungeon::{Level, Sight};
use shared::site::Rect as CellRect;
use shared::site::{BuildingKind, BuildingLook, RoofKind, WallKind};
use std::collections::BTreeSet;

use super::super::mesh::Surface;
use super::super::mesh::{LightGrid, View, build};
use super::super::theme::Shape;
use super::*;

/// A stone building in the middle of a road.
fn town() -> (Level, Vec<BuildingLook>) {
    let mut level = Level::new(0, 0, 11, 11);
    for y in 0..11 {
        for x in 0..11 {
            level.set(x, y, Cell::Cobblestone);
        }
    }
    for y in 3..8 {
        for x in 3..8 {
            level.set(x, y, Cell::Wall);
            level.set_room(x, y, Some(RoomId(0)));
        }
    }
    let look = BuildingLook {
        id: 0,
        kind: BuildingKind::Bank,
        walls: WallKind::Stone,
        roof: RoofKind::SlateTiles,
        rects: vec![CellRect {
            x: 3,
            y: 3,
            width: 5,
            height: 5,
        }],
        door: (5, 7),
        faces: (0, 1),
    };
    level.set(5, 7, Cell::BuildingDoor);
    (level, vec![look])
}

/// Every wall face, on any side, shows the building's own walls.
#[test]
fn all_sides_of_a_building_use_its_wall_picture() {
    let (level, buildings) = town();
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 9, 9, None);
    let view = View {
        cell: (9, 9),
        sight: sight.clone(),
    };
    let light = LightGrid::filled(&level, Vec3::ONE);
    let camera = Vec3::new(1000.0, 800.0, 1000.0);
    let meshes = build(
        &level,
        &[view],
        &sight,
        &sight,
        &light,
        &[],
        &buildings,
        &BTreeSet::new(),
        &[],
        false,
        camera,
        Shape::Built,
        &Sight::new(&level),
    );
    let mesh = &meshes.solid[Surface::Town as usize];
    let normals = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|v| v.as_float3())
        .unwrap();
    let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
        mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
    else {
        panic!("no picture coordinates");
    };
    let tile = wall_tile(WallKind::Stone);
    let (low, high) = (atlas_uv(tile, 0.0, 0.0), atlas_uv(tile, 1.0, 1.0));
    let door_tile = FIRST_DOOR_TILE + BuildingKind::Bank as usize;
    let (door_low, door_high) = (atlas_uv(door_tile, 0.0, 0.0), atlas_uv(door_tile, 1.0, 1.0));
    let inside = |uv: &[f32; 2], low: [f32; 2], high: [f32; 2]| {
        (low[0]..=high[0]).contains(&uv[0]) && (low[1]..=high[1]).contains(&uv[1])
    };
    let mut sides = 0;
    for (normal, uv) in normals.iter().zip(uvs) {
        if normal[1] == 0.0 {
            sides += 1;
            assert!(
                inside(uv, low, high) || inside(uv, door_low, door_high),
                "{normal:?}: {uv:?}"
            );
        }
    }
    assert!(sides > 0);
}
