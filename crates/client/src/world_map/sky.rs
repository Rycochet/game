//! Clouds, the darkening below them, moving water and precipitation.
//!
//! `shared::weather` gives cloud cover and rain at any place and time. Here it
//! is sampled onto coarse textures around the player: two moments of the sky,
//! a second apart, which the shaders fade between, so it changes smoothly
//! however seldom it is worked out. Three shaders read them: the terrain
//! (`assets/shaders/terrain.wesl`), the cloud sheet (`clouds.wesl`) and
//! precipitation (`precipitation.wesl`). All purely visual.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::ImageSampler;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;
use shared::rng::Seed;
use shared::world::GAME_HOUR_SECONDS;

use super::{FollowsPlayer, WorldMapScene, view_square};
use crate::common::{CurrentWorld, MapPosition, ServerClock, SkyNow};
use crate::player::Player;
use crate::view3d::{OrbitCamera, ViewCamera};

/// Sky texture points each way, and the grid units between them: low
/// resolution, and light to work out. It covers more than the window, so the
/// player can walk a while before it moves.
const SKY_SIDE: usize = 100;
const SKY_SPACING: i64 = 10;

/// Seconds between the two moments of the sky.
const SKY_INTERVAL: f64 = 1.0;

/// Grid units the player can be from a texture's centre before it is worked
/// out again at once, rather than at the next moment (teleports).
const SKY_REACH: i64 = 180;

/// Height of the cloud sheet, above the highest ground there is.
const CLOUD_HEIGHT: f32 = 170.0;

/// How much of the ground's own colour is added round the player, in the dark.
/// None in full light, where it would only clip.
const PLAYER_GLOW: f32 = 0.35;

/// Quads of precipitation over the shader's `DROP_TILE`, shown in proportion to
/// how hard it is falling. About half are over the window.
const DROPS: u32 = 6000;

pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainSky>;

/// Everything the sky's shaders share. Order matches `struct Sky` in them.
#[derive(ShaderType, Reflect, Clone, Copy, Debug, Default)]
pub struct SkyParams {
    /// World position of each sky texture's corner, and their size.
    origin_a: Vec2,
    origin_b: Vec2,
    extent: f32,
    /// How far from the first moment of the sky to the second, `0..=1`.
    blend: f32,
    /// Seconds, for things moving.
    time: f32,
    cloud_height: f32,
    /// The terrain window (see `ViewSquare`).
    half: f32,
    right: Vec2,
    towards: Vec2,
    centre: Vec2,
    /// `rgb`: what to multiply colours by to look lit by the sky (`SkyNow::lit`).
    /// `a`: how strong the glow round the player is.
    light: Vec4,
}

/// Adds the sky to the terrain's standard material.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct TerrainSky {
    #[uniform(100)]
    params: SkyParams,
    #[texture(101)]
    #[sampler(102)]
    sky_a: Handle<Image>,
    #[texture(103)]
    sky_b: Handle<Image>,
}

impl MaterialExtension for TerrainSky {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wesl".into()
    }
}

/// The see-through sheet of cloud.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CloudMaterial {
    #[uniform(0)]
    params: SkyParams,
    #[texture(1)]
    #[sampler(2)]
    sky_a: Handle<Image>,
    #[texture(3)]
    sky_b: Handle<Image>,
}

impl Material for CloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/clouds.wesl".into()
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
}

/// Falling rain and snow.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PrecipitationMaterial {
    #[uniform(0)]
    params: SkyParams,
    #[texture(1)]
    #[sampler(2)]
    sky_a: Handle<Image>,
    #[texture(3)]
    sky_b: Handle<Image>,
}

impl Material for PrecipitationMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/precipitation.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/precipitation.wesl".into()
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

/// The sky textures and what shows them.
#[derive(Resource)]
pub struct SkyAssets {
    /// The older moment, then the newer. Swapped as time goes on.
    moments: [Handle<Image>; 2],
    pub terrain: Handle<TerrainMaterial>,
    /// The same, for rivers: drawn from both sides.
    pub rivers: Handle<TerrainMaterial>,
    clouds: Handle<CloudMaterial>,
    precipitation: Handle<PrecipitationMaterial>,
    cloud_sheet: Handle<Mesh>,
}

/// Makes the sky's assets and spawns the cloud sheet and precipitation.
#[allow(clippy::too_many_arguments)]
pub fn spawn(
    commands: &mut Commands,
    visibility: Visibility,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    terrain_materials: &mut Assets<TerrainMaterial>,
    clouds: &mut Assets<CloudMaterial>,
    precipitation: &mut Assets<PrecipitationMaterial>,
) {
    let mut new_moment = || {
        let mut image = Image::new_fill(
            Extent3d {
                width: SKY_SIDE as u32,
                height: SKY_SIDE as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 128, 255],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::linear();
        images.add(image)
    };
    let moments = [new_moment(), new_moment()];

    let params = SkyParams::default();
    let terrain_material = |cull_mode| TerrainMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            cull_mode,
            ..default()
        },
        extension: TerrainSky {
            params,
            sky_a: moments[0].clone(),
            sky_b: moments[1].clone(),
        },
    };
    let terrain = terrain_materials.add(terrain_material(StandardMaterial::default().cull_mode));
    // River strips may turn either way as they wander.
    let rivers = terrain_materials.add(terrain_material(None));

    let cloud_material = clouds.add(CloudMaterial {
        params,
        sky_a: moments[0].clone(),
        sky_b: moments[1].clone(),
    });
    let cloud_sheet = meshes.add(Mesh::from(Cuboid::from_length(0.0)));
    commands.spawn((
        WorldMapScene,
        FollowsPlayer,
        Mesh3d(cloud_sheet.clone()),
        MeshMaterial3d(cloud_material.clone()),
        NoFrustumCulling,
        NotShadowCaster,
        Transform::default(),
        visibility,
    ));

    let drops = precipitation.add(PrecipitationMaterial {
        params,
        sky_a: moments[0].clone(),
        sky_b: moments[1].clone(),
    });
    commands.spawn((
        WorldMapScene,
        Mesh3d(meshes.add(drops_mesh())),
        MeshMaterial3d(drops.clone()),
        NoFrustumCulling,
        NotShadowCaster,
        Transform::default(),
        visibility,
    ));

    commands.insert_resource(SkyAssets {
        moments,
        terrain,
        rivers,
        clouds: cloud_material,
        precipitation: drops,
        cloud_sheet,
    });
}

/// `DROPS` quads whose positions are random numbers, for the shader to place.
fn drops_mesh() -> Mesh {
    let mut rng = Seed::new(1).rng();
    let mut positions = Vec::new();
    let mut corners = Vec::new();
    let mut indices = Vec::new();
    for drop in 0..DROPS {
        let seed = [
            rng.range_f64(0.0, 1.0) as f32,
            rng.range_f64(0.0, 1.0) as f32,
            rng.range_f64(0.0, 1.0) as f32,
        ];
        for corner in [[-0.5f32, 0.5], [-0.5, -0.5], [0.5, -0.5], [0.5, 0.5]] {
            positions.push(seed);
            corners.push(corner);
        }
        let first = drop * 4;
        indices.extend_from_slice(&[first, first + 1, first + 2, first, first + 2, first + 3]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, corners)
    .with_inserted_indices(Indices::U32(indices))
}

/// One moment of the sky, written into a texture.
fn work_out_moment(
    world: &CurrentWorld,
    image: &mut Image,
    temperatures: &mut Temperatures,
    origin: (i64, i64),
    hours: f64,
) {
    temperatures.update(world, origin);
    let Some(data) = image.data.as_mut() else {
        return;
    };
    for row in 0..SKY_SIDE {
        for column in 0..SKY_SIDE {
            let weather = world.weather.sample(
                origin.0 + column as i64 * SKY_SPACING,
                origin.1 + row as i64 * SKY_SPACING,
                hours,
            );
            let i = row * SKY_SIDE + column;
            data[i * 4] = (weather.cover * 255.0) as u8;
            data[i * 4 + 1] = (weather.rain * 255.0) as u8;
            data[i * 4 + 2] = temperatures.values[i];
        }
    }
}

/// Temperature at each texture point, kept until the texture moves: it doesn't
/// change with time.
#[derive(Default)]
struct Temperatures {
    origin: Option<(i64, i64)>,
    values: Vec<u8>,
}

impl Temperatures {
    fn update(&mut self, world: &CurrentWorld, origin: (i64, i64)) {
        if self.origin == Some(origin) {
            return;
        }
        self.origin = Some(origin);
        self.values.clear();
        let sea_level = world.params.sea_level;
        for row in 0..SKY_SIDE as i64 {
            for column in 0..SKY_SIDE as i64 {
                let (x, y) = (
                    origin.0 + column * SKY_SPACING,
                    origin.1 + row * SKY_SPACING,
                );
                let temperature = world.climate.sample(x, y, sea_level).temperature;
                self.values.push((temperature * 255.0) as u8);
            }
        }
    }
}

/// What the sky was last worked out from.
#[derive(Default)]
pub struct SkyState {
    /// Each moment's texture position, older first.
    origins: [Option<(i64, i64)>; 2],
    /// The clock's time at the newer moment.
    newer_hours: f64,
    temperatures: Temperatures,
    square: Option<(f32, f32)>,
}

/// Every frame: passes the time and window to the shaders, and each
/// `SKY_INTERVAL` works out the next moment of the sky around the player.
#[allow(clippy::too_many_arguments)]
pub fn update(
    time: Res<Time>,
    world: Res<CurrentWorld>,
    clock: Res<ServerClock>,
    sky_now: Res<SkyNow>,
    assets: Option<ResMut<SkyAssets>>,
    player: Single<&MapPosition, With<Player>>,
    orbit: Single<&OrbitCamera, With<ViewCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut clouds: ResMut<Assets<CloudMaterial>>,
    mut precipitation: ResMut<Assets<PrecipitationMaterial>>,
    mut state: Local<SkyState>,
) {
    let Some(mut assets) = assets else {
        return;
    };
    let (px, py) = (player.x, player.y);
    let hours = clock.hours();
    let interval = SKY_INTERVAL / GAME_HOUR_SECONDS as f64;

    // The texture snaps to whole spacings, so clouds stay put as the player
    // walks.
    let half_side = (SKY_SIDE as i64 / 2) * SKY_SPACING;
    let origin = (
        px.div_euclid(SKY_SPACING) * SKY_SPACING - half_side,
        py.div_euclid(SKY_SPACING) * SKY_SPACING - half_side,
    );
    let far_off = state.origins.iter().any(|o| {
        o.is_none_or(|o| {
            (o.0 + half_side - px).abs() > SKY_REACH || (o.1 + half_side - py).abs() > SKY_REACH
        })
    });
    // Just started, teleported, or the clock jumped: both moments, now.
    if far_off
        || hours < state.newer_hours - 2.0 * interval
        || hours > state.newer_hours + 2.0 * interval
    {
        let SkyState {
            temperatures,
            origins,
            newer_hours,
            ..
        } = &mut *state;
        for (i, moment) in [hours, hours + interval].into_iter().enumerate() {
            if let Some(mut image) = images.get_mut(&assets.moments[i]) {
                work_out_moment(&world, &mut image, temperatures, origin, moment);
            }
            origins[i] = Some(origin);
        }
        *newer_hours = hours + interval;
    } else if hours >= state.newer_hours {
        // The newer moment becomes the older; the next is worked out into
        // the older's texture.
        assets.moments.swap(0, 1);
        state.origins.swap(0, 1);
        let next = state.newer_hours + interval;
        let SkyState {
            temperatures,
            origins,
            ..
        } = &mut *state;
        if let Some(mut image) = images.get_mut(&assets.moments[1]) {
            work_out_moment(&world, &mut image, temperatures, origin, next);
        }
        origins[1] = Some(origin);
        state.newer_hours = next;
    }

    let corner = |o: Option<(i64, i64)>| {
        let (x, y) = o.unwrap_or_default();
        // Texel centres are half a spacing in from the texture's corner.
        Vec2::new(x as f32, y as f32) - Vec2::splat(SKY_SPACING as f32 / 2.0)
    };
    let square = view_square(&orbit);
    let params = SkyParams {
        origin_a: corner(state.origins[0]),
        origin_b: corner(state.origins[1]),
        extent: (SKY_SIDE as i64 * SKY_SPACING) as f32,
        blend: (1.0 - (state.newer_hours - hours) / interval).clamp(0.0, 1.0) as f32,
        time: time.elapsed_secs(),
        cloud_height: CLOUD_HEIGHT,
        half: square.half,
        right: square.right,
        towards: square.towards,
        centre: Vec2::new(px as f32, py as f32),
        light: sky_now
            .lit()
            .extend(PLAYER_GLOW * (1.0 - sky_now.flat.max_element().min(1.0))),
    };
    let [older, newer] = &assets.moments;
    // Writing to these also tells Bevy to pick up changed textures.
    for handle in [&assets.terrain, &assets.rivers] {
        if let Some(mut material) = terrain_materials.get_mut(handle) {
            material.extension.params = params;
            material.extension.sky_a = older.clone();
            material.extension.sky_b = newer.clone();
        }
    }
    if let Some(mut material) = clouds.get_mut(&assets.clouds) {
        material.params = params;
        material.sky_a = older.clone();
        material.sky_b = newer.clone();
    }
    if let Some(mut material) = precipitation.get_mut(&assets.precipitation) {
        material.params = params;
        material.sky_a = older.clone();
        material.sky_b = newer.clone();
    }

    // The cloud sheet: the window's square, straight above it.
    if state.square != Some((orbit.yaw, orbit.distance))
        && let Some(mut mesh) = meshes.get_mut(&assets.cloud_sheet)
    {
        state.square = Some((orbit.yaw, orbit.distance));
        let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .map(|(u, v)| (square.right * u + square.towards * v) * square.half);
        let positions: Vec<[f32; 3]> = corners.iter().map(|c| [c.x, CLOUD_HEIGHT, c.y]).collect();
        *mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 4])
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        // Seen from above whichever way it was wound.
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3, 0, 2, 1, 0, 3, 2]));
    }
}
