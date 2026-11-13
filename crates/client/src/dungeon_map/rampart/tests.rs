use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use shared::site::{Gate, Rect, WallLook, WallMaterial};
use std::collections::{BTreeMap, BTreeSet};

use super::super::mesh::{LightGrid, Surface, View, build_posed};
use super::super::theme::Shape;
use super::LOG_TILE;
use crate::dungeon_map::town::atlas_uv;

/// A walled square of ground, its gate in the south side.
fn walled() -> (Level, WallLook) {
    let mut level = Level::new(-2, -2, 12, 12);
    for y in -2..10 {
        for x in -2..10 {
            level.set(x, y, Cell::Ground);
        }
    }
    let wall = WallLook {
        rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 8,
        },
        material: WallMaterial::Log,
        gates: vec![Gate {
            x: 3,
            y: 7,
            runs_east_west: true,
        }],
    };
    for y in 0..8 {
        for x in 0..8 {
            if wall.is_gate(x, y) {
                level.set(x, y, Cell::Door);
            } else if wall.has(x, y) {
                level.set(x, y, Cell::Wall);
            }
        }
    }
    (level, wall)
}

/// A wall round a site is drawn by its material in the town's picture, never
/// as the place's own walls.
#[test]
fn a_rampart_is_drawn_in_its_material() {
    let (level, wall) = walled();
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 4, 9, None);
    let views = vec![View {
        cell: (4, 9),
        sight: sight.clone(),
    }];
    let meshes = build_posed(
        &level,
        (&views, &sight, &sight, &Sight::new(&level)),
        &LightGrid::new(&level),
        &[],
        (
            &[],
            &BTreeSet::new(),
            &[],
            std::slice::from_ref(&wall),
            false,
        ),
        (Vec3::new(0.0, 600.0, 2000.0), Shape::Built),
        None,
        &BTreeMap::new(),
    );
    assert!(!meshes.has_solid[Surface::Wall as usize], "no plain walls");
    let town = &meshes.solid[Surface::Town as usize];
    let Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) =
        town.attribute(Mesh::ATTRIBUTE_UV_0)
    else {
        panic!("the town mesh has picture coordinates");
    };
    let [u0, v0] = atlas_uv(LOG_TILE, 0.0, 0.0);
    let [u1, v1] = atlas_uv(LOG_TILE, 1.0, 1.0);
    assert!(!uvs.is_empty());
    for &[u, v] in uvs {
        assert!((u0..=u1).contains(&u) && (v0..=v1).contains(&v), "{u}, {v}");
    }
}
