//! The border round a poi set into the wilderness (`CurrentDungeon::borders`),
//! drawn as dark diagonal stripes over the ground: clicking or tapping a
//! cell of it walks there and leaves for the world map (`site`), while
//! walking over it does nothing.

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::CELL_SIZE;
use super::parts::DungeonScene;
use crate::common::{CurrentDungeon, DrawnGround};

/// How far apart the stripes are, and how wide, in cells: lines where the
/// sum of a point's coordinates is a multiple of `GAP`, so they run on from
/// cell to cell.
const GAP: f32 = 0.5;
const WIDTH: f32 = 0.12;

/// How far above the ground the stripes lie, in cells, so the ground doesn't
/// show through them.
const LIFT: f32 = 0.02;

/// How dark the stripes are (their alpha over the ground).
const DARKNESS: f32 = 0.4;

/// The border's mesh, a child of the scene.
#[derive(Component)]
pub(super) struct BorderPart;

/// Draws the border again whenever its cells change: made the first time it
/// is needed, under the scene, so it shows and hides with it.
pub(super) fn draw_borders(
    (dungeon, heights): (Res<CurrentDungeon>, Res<DrawnGround>),
    scene: Query<Entity, With<DungeonScene>>,
    mut part: Query<&mut Mesh3d, With<BorderPart>>,
    mut drawn: Local<Option<Vec<(i64, i64)>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if !dungeon.is_changed() && !heights.is_changed() {
        return;
    }
    let cells: Vec<(i64, i64)> = dungeon.borders.iter().copied().collect();
    if drawn.as_ref() == Some(&cells) && !heights.is_changed() {
        return;
    }
    let mesh = meshes.add(stripes(&cells, &heights));
    if let Ok(mut current) = part.single_mut() {
        current.0 = mesh;
    } else if let Ok(scene) = scene.single() {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, DARKNESS),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        commands.entity(scene).with_child((
            BorderPart,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::default(),
            NotShadowCaster,
        ));
    } else {
        return;
    }
    *drawn = Some(cells);
}

/// The stripes over `cells`: in each, the pieces of the lines `u + v = k`
/// (`u`, `v` from 0 to 1 across the cell) for `k` a multiple of `GAP`.
/// Each on the ground under it.
fn stripes(cells: &[(i64, i64)], heights: &DrawnGround) -> Mesh {
    let (mut positions, mut indices) = (Vec::new(), Vec::new());
    let lines = (1..(2.0 / GAP) as i32).map(|step| step as f32 * GAP);
    let lines: Vec<f32> = lines.collect();
    // Half the stripe's width, across it (it runs at 45°).
    let half = WIDTH / 2.0 * std::f32::consts::FRAC_1_SQRT_2;
    for &(x, y) in cells {
        let place = |u: f32, v: f32| {
            let (gx, gy) = (x as f32 - 0.5 + u, y as f32 - 0.5 + v);
            let ground = heights.drawn_between(gx, gy);
            [gx * CELL_SIZE, (ground + LIFT) * CELL_SIZE, gy * CELL_SIZE]
        };
        for &k in &lines {
            let (from, to) = if k <= 1.0 {
                ((k, 0.0), (0.0, k))
            } else {
                ((1.0, k - 1.0), (k - 1.0, 1.0))
            };
            let first = positions.len() as u32;
            positions.push(place(from.0 - half, from.1 - half));
            positions.push(place(from.0 + half, from.1 + half));
            positions.push(place(to.0 + half, to.1 + half));
            positions.push(place(to.0 - half, to.1 - half));
            indices.extend_from_slice(&[first, first + 1, first + 2, first, first + 2, first + 3]);
        }
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}
