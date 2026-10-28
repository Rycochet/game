//! The dungeon (and building) view: a grid of floor and wall cells in 3D,
//! seen through `view3d`'s camera with the same tokens as the world map. See
//! `docs/context/client/dungeon_map.md`.
//!
//! Only what the player (or anything sharing its sight) can see is drawn:
//! floors, and only the wall sides someone sees (never their tops). A seen
//! side facing away from the camera is drawn half see-through. Light is dull
//! everywhere, and brighter near light sources, which walls block. Left-click
//! a seen floor to travel there.

mod mesh;
mod picking;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::ImageSampler;
use bevy::light::NotShadowCaster;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use shared::dungeon::Sight;

use crate::common::{CurrentDungeon, GameView, LightSource, SharesSight, ViewStats};
use crate::player::{MapPosition, Player, PlayerSystems, SmoothPosition, TravelTo};
use crate::view3d::{TOKEN_DIAMETER, View3dSystems, ViewCamera, ViewGrid};
use mesh::{LightGrid, View, WallFace};

/// Size of a cell in 3D units: about the size of a token's picture.
pub const CELL_SIZE: f32 = TOKEN_DIAMETER * 1.5;

/// How tall walls stand.
const WALL_HEIGHT: f32 = CELL_SIZE;

/// Placeholder tile colours until there are tile pictures: floors dark grey,
/// walls medium brown, both with a black border (from the texture).
const FLOOR_COLOUR: Color = Color::srgb_u8(0x40, 0x40, 0x40);
const WALL_COLOUR: Color = Color::srgb_u8(0x8b, 0x5a, 0x2b);

/// The dull light everywhere, as a fraction of full light.
const AMBIENT_LIGHT: f32 = 0.2;

/// Opacity of wall sides facing away from the camera.
const SEE_THROUGH_ALPHA: f32 = 0.5;

/// The tile texture: white with a black border this many pixels wide, on a
/// square this many pixels across.
const TILE_PIXELS: u32 = 32;
const TILE_BORDER_PIXELS: u32 = 1;

/// Tokens are lit by real lights (the tiles' light is baked into their
/// colours): a point light this high above each light source, and a dim
/// ambient fill so tokens in the dark are still visible.
const TOKEN_LIGHT_HEIGHT: f32 = CELL_SIZE;
const TOKEN_LIGHT_LUMENS: f32 = 1.0e8;
const TOKEN_AMBIENT_BRIGHTNESS: f32 = 300.0;

pub struct DungeonMapPlugin;

impl Plugin for DungeonMapPlugin {
    fn build(&self, app: &mut App) {
        let in_view = in_state(GameView::Dungeon);
        app.init_resource::<Shown>()
            .add_systems(Startup, spawn_scene)
            .add_systems(OnEnter(GameView::Dungeon), show_scene)
            .add_systems(OnExit(GameView::Dungeon), hide_scene)
            .add_systems(
                Update,
                pick_on_click
                    .in_set(PlayerSystems::Input)
                    .run_if(in_view.clone()),
            )
            .add_systems(
                Update,
                (update_meshes, sync_lights)
                    .after(PlayerSystems::Travel)
                    .after(View3dSystems::Place)
                    .run_if(in_view),
            );
    }
}

/// The root of everything this view draws, hidden while another view is
/// current.
#[derive(Component)]
struct DungeonScene;

/// What was last drawn, for click picking: the combined sight and the solid
/// wall sides.
#[derive(Resource, Default)]
struct Shown {
    sight: Option<Sight>,
    solid_faces: Vec<WallFace>,
}

/// The mesh of floors and solid walls.
#[derive(Component)]
struct SolidPart;

/// The mesh of half see-through walls.
#[derive(Component)]
struct SeeThroughPart;

/// A real light for tokens, following `owner`'s `LightSource`.
#[derive(Component)]
struct TokenLight {
    owner: Entity,
}

fn spawn_scene(
    mut commands: Commands,
    view: Res<State<GameView>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let tile = images.add(tile_texture());
    // Unlit: the light is in the vertex colours, worked out on the grid.
    let solid = materials.add(StandardMaterial {
        base_color_texture: Some(tile.clone()),
        unlit: true,
        ..default()
    });
    let see_through = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, SEE_THROUGH_ALPHA),
        base_color_texture: Some(tile),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        // These sides face away from the camera, so they are back faces.
        cull_mode: None,
        ..default()
    });
    let visibility = if *view.get() == GameView::Dungeon {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    commands.spawn((
        DungeonScene,
        Transform::default(),
        visibility,
        children![
            (
                SolidPart,
                // Placeholder until the first update builds the real mesh.
                Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
                MeshMaterial3d(solid),
                // The mesh is replaced as things change, so its bounds go stale.
                NoFrustumCulling,
                NotShadowCaster,
            ),
            (
                SeeThroughPart,
                Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
                MeshMaterial3d(see_through),
                NoFrustumCulling,
                NotShadowCaster,
            ),
        ],
    ));
}

/// White with a black border, sampled pixel by pixel: multiplied by the
/// vertex colours, it gives each tile its colour and outline.
fn tile_texture() -> Image {
    let mut data = Vec::with_capacity((TILE_PIXELS * TILE_PIXELS * 4) as usize);
    let edge = TILE_PIXELS - TILE_BORDER_PIXELS;
    for y in 0..TILE_PIXELS {
        for x in 0..TILE_PIXELS {
            let border = x < TILE_BORDER_PIXELS || y < TILE_BORDER_PIXELS || x >= edge || y >= edge;
            let value = if border { 0 } else { 255 };
            data.extend([value, value, value, 255]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: TILE_PIXELS,
            height: TILE_PIXELS,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Becoming the current view: shows the scene, sets the dim fill light for
/// tokens, and tells `view3d` the grid's size, with the floor at 0.
fn show_scene(mut commands: Commands, mut scene: Query<&mut Visibility, With<DungeonScene>>) {
    for mut visibility in &mut scene {
        *visibility = Visibility::Inherited;
    }
    commands.insert_resource(GlobalAmbientLight {
        brightness: TOKEN_AMBIENT_BRIGHTNESS,
        ..default()
    });
    commands.insert_resource(ViewGrid {
        cell_size: CELL_SIZE,
        floor: 0.0,
    });
}

fn hide_scene(mut scene: Query<&mut Visibility, With<DungeonScene>>) {
    for mut visibility in &mut scene {
        *visibility = Visibility::Hidden;
    }
}

/// When anyone who sees or gives light moves, the level changes, or the
/// camera moves (turning, zooming or following the player): works out what
/// is seen and how it is lit, and rebuilds the meshes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_meshes(
    dungeon: Res<CurrentDungeon>,
    player: Single<Ref<MapPosition>, With<Player>>,
    sharers: Query<Ref<MapPosition>, (With<SharesSight>, Without<Player>)>,
    lights: Query<(Ref<MapPosition>, Ref<LightSource>)>,
    camera: Single<&Transform, With<ViewCamera>>,
    solid: Single<(&Mesh3d, &mut Visibility), (With<SolidPart>, Without<SeeThroughPart>)>,
    see_through: Single<(&Mesh3d, &mut Visibility), With<SeeThroughPart>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stats: ResMut<ViewStats>,
    mut shown: ResMut<Shown>,
    // The camera position last built for.
    mut built_for: Local<Option<Vec3>>,
) {
    let camera = camera.translation;
    let changed = *built_for != Some(camera)
        || dungeon.is_changed()
        || player.is_changed()
        || sharers.iter().any(|position| position.is_changed())
        || lights
            .iter()
            .any(|(position, light)| position.is_changed() || light.is_changed());
    if !changed {
        return;
    }
    *built_for = Some(camera);

    let start = Instant::now();
    let level = &dungeon.level;
    let views: Vec<View> = std::iter::once(&**player)
        .chain(sharers.iter().map(|position| position.into_inner()))
        .map(|position| {
            let mut sight = Sight::new(level);
            sight.add_view(level, position.x, position.y, None);
            View {
                cell: (position.x, position.y),
                sight,
            }
        })
        .collect();
    let mut sight = Sight::new(level);
    for view in &views {
        sight.add_view(level, view.cell.0, view.cell.1, None);
    }
    let mut light = LightGrid::new(level);
    for (position, source) in &lights {
        let colour = LinearRgba::from(source.colour);
        light.add_light(
            level,
            (position.x, position.y),
            source.radius,
            Vec3::new(colour.red, colour.green, colour.blue),
        );
    }
    let built_meshes = mesh::build(level, &views, &sight, &light, camera);
    shown.sight = Some(sight);
    shown.solid_faces = built_meshes.solid_faces;

    let (solid_mesh, mut solid_visibility) = solid.into_inner();
    let (see_through_mesh, mut see_through_visibility) = see_through.into_inner();
    for (handle, visibility, mesh, has_faces) in [
        (
            &solid_mesh.0,
            &mut solid_visibility,
            built_meshes.solid,
            built_meshes.has_solid,
        ),
        (
            &see_through_mesh.0,
            &mut see_through_visibility,
            built_meshes.see_through,
            built_meshes.has_see_through,
        ),
    ] {
        // An empty mesh is hidden rather than drawn with no triangles.
        visibility.set_if_neq(if has_faces {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if let Some(mut stored) = meshes.get_mut(handle) {
            *stored = mesh;
        }
    }
    let ms = start.elapsed().as_secs_f32() * 1000.0;
    stats.summary = format!("Dungeon: sight, light and meshes {ms:.2} ms");
}

/// A left click on a seen floor sends the player travelling there (in a
/// straight line for now: walls don't stop travel until there is pathfinding).
fn pick_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    mut travel: MessageWriter<TravelTo>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let (Some(cursor), Some(sight)) = (window.cursor_position(), &shown.sight) else {
        return;
    };
    let (camera, camera_transform) = *camera;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    if let Some((x, y)) = picking::pick(ray, &dungeon.level, sight, &shown.solid_faces) {
        travel.write(TravelTo { x, y });
    }
}

/// Keeps a real point light over each `LightSource`, so tokens near it are
/// lit like the tiles are.
fn sync_lights(
    mut commands: Commands,
    scene: Single<Entity, With<DungeonScene>>,
    sources: Query<(Entity, &LightSource, &SmoothPosition)>,
    mut lights: Query<(Entity, &TokenLight, &mut PointLight, &mut Transform)>,
) {
    for (entity, token_light, mut point, mut transform) in &mut lights {
        let Ok((_, source, position)) = sources.get(token_light.owner) else {
            commands.entity(entity).despawn();
            continue;
        };
        point.color = source.colour;
        point.range = (source.radius as f32 + 1.0) * CELL_SIZE;
        transform.translation = Vec3::new(
            position.x as f32 * CELL_SIZE,
            TOKEN_LIGHT_HEIGHT,
            position.y as f32 * CELL_SIZE,
        );
    }
    for (owner, source, _) in &sources {
        if lights.iter().any(|(_, light, _, _)| light.owner == owner) {
            continue;
        }
        commands.entity(*scene).with_child((
            TokenLight { owner },
            PointLight {
                color: source.colour,
                intensity: TOKEN_LIGHT_LUMENS,
                range: (source.radius as f32 + 1.0) * CELL_SIZE,
                ..default()
            },
            // Placed on the next run.
            Transform::default(),
        ));
    }
}
