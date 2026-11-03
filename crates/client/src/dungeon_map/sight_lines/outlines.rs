//! Working out the outlines again when viewers move, and keeping the mask over the view.

use bevy::camera::ScalingMode;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::dungeon::Sight;

use super::{
    Area, MASK_SPARE, MASK_TOO_BIG, MaskCamera, SightLines, SurfaceMaterial, Vec2d, WallMap,
    add_cells, add_fan, cross_cells, fan_mesh, touch_cells, view_area,
};
use crate::common::{CurrentDungeon, SharesSight, SmoothPosition};
use crate::player::Player;
use crate::view3d::ViewCamera;

/// Marks the mesh of the outlines.
#[derive(Component)]
pub struct OutlineMesh;

/// When a viewer moves or the level changes: the outlines again, what they
/// touch, and the mesh the mask's camera draws.
#[allow(clippy::type_complexity)]
pub fn update_outlines(
    dungeon: Res<CurrentDungeon>,
    player: Single<&SmoothPosition, With<Player>>,
    sharers: Query<&SmoothPosition, (With<SharesSight>, Without<Player>)>,
    mut lines: ResMut<SightLines>,
    mut meshes: ResMut<Assets<Mesh>>,
    outline: Single<&mut Visibility, With<OutlineMesh>>,
) {
    let from: Vec<Vec2d> = std::iter::once(*player)
        .chain(sharers.iter())
        .map(|position| Vec2d {
            x: position.x,
            y: position.y,
        })
        .collect();
    if !dungeon.is_changed() && lines.from == from {
        return;
    }
    let start = Instant::now();
    let level = &dungeon.level;
    if dungeon.is_changed() {
        lines.walls = WallMap::of(level);
    }
    let mut touched_by = Vec::new();
    let mut crossed = Sight::new(level);
    let mut triangles = Vec::new();
    for &viewer in &from {
        let outline = lines.walls.outline(viewer);
        let mut touched = Sight::new(level);
        touch_cells(level, viewer, &outline, &mut touched);
        touched_by.push(touched);
        cross_cells(&outline, &mut crossed);
        add_fan(&mut triangles, viewer, &outline);
    }
    let mut touched = Sight::new(level);
    for one in &touched_by {
        add_cells(level, one, &mut touched);
    }
    let mut visibility = outline.into_inner();
    visibility.set_if_neq(if triangles.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    if !triangles.is_empty()
        && let Some(mut stored) = meshes.get_mut(&lines.outline_mesh)
    {
        *stored = fan_mesh(triangles);
    }
    lines.from = from;
    lines.crossed = Some(crossed);
    lines.cost_ms = start.elapsed().as_secs_f32() * 1000.0;
    if lines.touched_by != touched_by {
        lines.touched = Some(touched);
        lines.touched_by = touched_by;
        lines.changes += 1;
    }
}

/// Moves the mask to cover what the camera shows, keeping it where it is while
/// it still does, so the materials aren't all changed every frame.
#[allow(clippy::type_complexity)]
pub fn place_mask(
    dungeon: Res<CurrentDungeon>,
    view: Single<(&Camera, &Transform), (With<ViewCamera>, Without<MaskCamera>)>,
    mask_camera: Single<(&mut Transform, &mut Projection), With<MaskCamera>>,
    mut lines: ResMut<SightLines>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
) {
    let (camera, transform) = *view;
    let Some(wanted) = view_area(camera, transform, dungeon.level.bounds()) else {
        return;
    };
    if lines.area.is_some_and(|area| {
        let share = wanted.size().element_product() / area.size().element_product();
        area.contains(wanted) && share > MASK_TOO_BIG
    }) {
        return;
    }
    let spare = wanted.size() * MASK_SPARE;
    let area = Area {
        min: wanted.min - spare,
        max: wanted.max + spare,
    };
    lines.area = Some(area);
    let (mut mask_transform, mut projection) = mask_camera.into_inner();
    let centre = (area.min + area.max) / 2.0;
    mask_transform.translation = Vec3::new(centre.x, 1.0, centre.y);
    if let Projection::Orthographic(orthographic) = &mut *projection {
        orthographic.scaling_mode = ScalingMode::Fixed {
            width: area.size().x,
            height: area.size().y,
        };
    }
    let ids: Vec<_> = materials.ids().collect();
    for id in ids {
        if let Some(mut material) = materials.get_mut(id) {
            material.extension.params.origin = area.min;
            material.extension.params.per_unit = area.size().recip();
        }
    }
}

/// The mask's camera only works in this view.
pub fn set_mask_active<const ACTIVE: bool>(mut camera: Single<&mut Camera, With<MaskCamera>>) {
    camera.is_active = ACTIVE;
}
