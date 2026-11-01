//! Rain and snow over a site's outside ground: the weather at the site's place
//! on the world map, falling only on outside cells in sight. Looks like the
//! world map's (`assets/shaders/site_precipitation.wesl`). Purely visual.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use shared::rng::Seed;

use super::daylight::OutsideLight;
use super::{CELL_SIZE, Shown};
use crate::common::{CurrentDungeon, CurrentWorld, ServerClock, WorldReturn};

/// Quads of precipitation per cell, shown in proportion to how hard it is
/// falling: the same density as on the world map.
const DROPS_PER_CELL: u32 = 10;

/// How far above the ground drops start.
const FALL_HEIGHT: f32 = CELL_SIZE * 4.0;

/// Order matches `struct Weather` in the shader.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct RainParams {
    rain: f32,
    temperature: f32,
    /// Seconds, for things moving.
    time: f32,
    fall_height: f32,
    /// What to multiply colours by to look lit (`OutsideLight::lit`).
    light: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct RainMaterial {
    #[uniform(0)]
    params: RainParams,
}

impl Material for RainMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/site_precipitation.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/site_precipitation.wesl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

/// The mesh of drops over the outside cells in sight.
#[derive(Component)]
pub struct RainPart;

/// Spawns the (empty, hidden) drops under the scene.
pub fn spawn(
    commands: &mut Commands,
    scene: Entity,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<RainMaterial>,
) {
    commands.entity(scene).with_child((
        RainPart,
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        MeshMaterial3d(materials.add(RainMaterial {
            params: RainParams::default(),
        })),
        NoFrustumCulling,
        NotShadowCaster,
        Visibility::Hidden,
    ));
}

/// `DROPS_PER_CELL` quads over each cell, each corner holding the drop's place
/// on the ground, with a random number as its height for the shader. Drops
/// come from their cell's position, so they stay put as others come into sight.
fn drops_mesh(cells: &[(i64, i64)]) -> Mesh {
    let seed = Seed::new(1);
    let mut positions = Vec::new();
    let mut corners = Vec::new();
    let mut indices = Vec::new();
    for &(x, y) in cells {
        let mut rng = Seed::new(seed.at(&[x, y])).rng();
        for _ in 0..DROPS_PER_CELL {
            let place = [
                (x as f32 + rng.range_f64(-0.5, 0.5) as f32) * CELL_SIZE,
                rng.range_f64(0.0, 1.0) as f32,
                (y as f32 + rng.range_f64(-0.5, 0.5) as f32) * CELL_SIZE,
            ];
            let first = positions.len() as u32;
            for corner in [[-0.5f32, 0.5], [-0.5, -0.5], [0.5, -0.5], [0.5, 0.5]] {
                positions.push(place);
                corners.push(corner);
            }
            indices.extend_from_slice(&[first, first + 1, first + 2, first, first + 2, first + 3]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, corners)
    .with_inserted_indices(Indices::U32(indices))
}

/// Every frame: the weather at the site's place for the shader, and, when what
/// is in sight changed, drops over the outside cells in it.
#[allow(clippy::too_many_arguments)]
pub fn update(
    time: Res<Time>,
    clock: Res<ServerClock>,
    world: Option<Res<CurrentWorld>>,
    place: Res<WorldReturn>,
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    outside_light: Res<OutsideLight>,
    part: Single<(&Mesh3d, &MeshMaterial3d<RainMaterial>, &mut Visibility), With<RainPart>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<RainMaterial>>,
    mut outside: Local<Vec<(i64, i64)>>,
) {
    let Some(world) = world else {
        return;
    };
    let (mesh, material, mut visibility) = part.into_inner();

    if shown.is_changed()
        && let Some(sight) = &shown.sight
    {
        let level = &dungeon.level;
        let (min_x, min_y, width, height) = level.bounds();
        let mut cells = Vec::new();
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if level.cell(x, y).is_outside()
                    && sight.is_seen(x, y)
                    && dungeon.known.is_seen(x, y)
                {
                    cells.push((x, y));
                }
            }
        }
        if cells != *outside {
            visibility.set_if_neq(if cells.is_empty() {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            });
            if !cells.is_empty()
                && let Some(mut stored) = meshes.get_mut(&mesh.0)
            {
                *stored = drops_mesh(&cells);
            }
            *outside = cells;
        }
    }

    let (x, y) = (place.0.x, place.0.y);
    // Worked out as `world_map::sky` does, so it matches the map outside.
    let weather = world.weather.sample(x, y, clock.hours());
    let temperature = world.climate.sample(x, y, world.params.sea_level);
    if let Some(mut material) = materials.get_mut(&material.0) {
        material.params = RainParams {
            rain: weather.rain as f32,
            temperature: temperature.temperature as f32,
            time: time.elapsed_secs(),
            fall_height: FALL_HEIGHT,
            light: outside_light.lit().extend(1.0),
        };
    }
}
