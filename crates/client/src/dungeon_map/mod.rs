//! The dungeon (and building) view: a grid of floor and wall cells in 3D,
//! seen through `view3d`'s camera with the same tokens as the world map. See
//! `docs/context/client/dungeon_map.md`.
//!
//! Only what the player (or anything sharing its sight) can see is drawn:
//! floors, stairs, and only the wall sides someone sees (never their tops).
//! A seen side facing away from the camera is drawn half see-through. Walls
//! and floors use tile pictures chosen by the kind of place (`theme`). Light
//! is dull everywhere, and brighter near light sources, which walls block.
//! Left-click a seen walkable cell to travel there.

mod mesh;
mod picking;
mod theme;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::light::NotShadowCaster;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use shared::dungeon::Sight;
use shared::site::StairsDirection;

use crate::common::{
    CurrentDungeon, GameView, LightSource, MapPosition, SharesSight, SmoothPosition, ViewStats,
};
use crate::player::{Player, PlayerSystems, TravelTo};
use crate::view3d::{ClickedToken, TOKEN_DIAMETER, TokenView, View3dSystems, ViewCamera, ViewGrid};
use mesh::{LightGrid, Surface, View, WallFace};
use theme::Theme;

/// Size of a cell in 3D units: about the size of a token's picture.
pub const CELL_SIZE: f32 = TOKEN_DIAMETER * 1.5;

/// How tall walls stand.
const WALL_HEIGHT: f32 = CELL_SIZE;

/// The dull light everywhere, as a fraction of full light.
const AMBIENT_LIGHT: f32 = 0.2;

/// Opacity of wall sides facing away from the camera.
const SEE_THROUGH_ALPHA: f32 = 0.5;

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
                    .after(View3dSystems::Click)
                    .run_if(in_view.clone()),
            )
            .add_systems(
                Update,
                (apply_theme, update_meshes, hide_unseen_tokens, sync_lights)
                    .chain()
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

/// What was last drawn, for click picking: the combined sight, the solid
/// wall sides, and the cells with stairs up (which stand above the floor).
#[derive(Resource, Default)]
struct Shown {
    sight: Option<Sight>,
    solid_faces: Vec<WallFace>,
    stairs_up: Vec<(i64, i64)>,
}

/// The materials of the solid meshes (in `Surface::ALL` order) and of the
/// see-through walls, so their pictures can follow the theme.
#[derive(Resource)]
struct SurfaceMaterials {
    solid: Vec<Handle<StandardMaterial>>,
    see_through: Handle<StandardMaterial>,
}

/// The mesh of one kind of surface (floors, solid walls...).
#[derive(Component)]
struct SolidPart(Surface);

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
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let theme = Theme::of(None);
    // Unlit: the light is in the vertex colours, worked out on the grid.
    let solid: Vec<Handle<StandardMaterial>> = Surface::ALL
        .iter()
        .map(|&surface| {
            materials.add(StandardMaterial {
                base_color_texture: Some(load_tile(&asset_server, theme.picture(surface))),
                unlit: true,
                ..default()
            })
        })
        .collect();
    let see_through = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, SEE_THROUGH_ALPHA),
        base_color_texture: Some(load_tile(&asset_server, theme.wall)),
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
    let scene = commands
        .spawn((DungeonScene, Transform::default(), visibility))
        .id();
    for (&surface, material) in Surface::ALL.iter().zip(&solid) {
        commands.entity(scene).with_child((
            SolidPart(surface),
            // Placeholder until the first update builds the real mesh.
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
            MeshMaterial3d(material.clone()),
            // The mesh is replaced as things change, so its bounds go stale.
            NoFrustumCulling,
            NotShadowCaster,
        ));
    }
    commands.entity(scene).with_child((
        SeeThroughPart,
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        MeshMaterial3d(see_through.clone()),
        NoFrustumCulling,
        NotShadowCaster,
    ));
    commands.insert_resource(SurfaceMaterials { solid, see_through });
}

/// Loads a tile picture, sampled pixel by pixel to keep the pixel art crisp.
fn load_tile(asset_server: &AssetServer, path: &'static str) -> Handle<Image> {
    asset_server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        })
        .load(path)
}

/// The wall and floor pictures for the kind of place the player is in.
fn apply_theme(
    dungeon: Res<CurrentDungeon>,
    asset_server: Res<AssetServer>,
    surfaces: Res<SurfaceMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !dungeon.is_changed() {
        return;
    }
    let theme = Theme::of(dungeon.site.as_ref().map(|site| site.kind()));
    for surface in [Surface::Floor, Surface::Wall] {
        if let Some(mut material) = materials.get_mut(&surfaces.solid[surface as usize]) {
            material.base_color_texture = Some(load_tile(&asset_server, theme.picture(surface)));
        }
    }
    if let Some(mut material) = materials.get_mut(&surfaces.see_through) {
        material.base_color_texture = Some(load_tile(&asset_server, theme.wall));
    }
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

fn hide_scene(
    mut scene: Query<&mut Visibility, With<DungeonScene>>,
    mut tokens: Query<&mut Visibility, (With<TokenView>, Without<DungeonScene>)>,
) {
    for mut visibility in &mut scene {
        *visibility = Visibility::Hidden;
    }
    // Other views show every token they don't hide themselves.
    for mut visibility in &mut tokens {
        visibility.set_if_neq(Visibility::Inherited);
    }
}

/// Tokens are shown only on cells someone sees.
fn hide_unseen_tokens(
    shown: Res<Shown>,
    positions: Query<&MapPosition>,
    mut tokens: Query<(&TokenView, &mut Visibility)>,
) {
    let Some(sight) = &shown.sight else {
        return;
    };
    for (view, mut visibility) in &mut tokens {
        let seen = positions
            .get(view.owner)
            .is_ok_and(|position| sight.is_seen(position.x, position.y));
        visibility.set_if_neq(if seen {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
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
    mut solid: Query<(&SolidPart, &Mesh3d, &mut Visibility), Without<SeeThroughPart>>,
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
    let built = mesh::build(level, &views, &sight, &light, &dungeon.stairs, camera);
    shown.sight = Some(sight);
    shown.solid_faces = built.solid_faces;
    shown.stairs_up = dungeon
        .stairs
        .iter()
        .filter(|stairs| stairs.direction == StairsDirection::Up)
        .map(|stairs| (stairs.x, stairs.y))
        .collect();

    // An empty mesh is hidden rather than drawn with no triangles.
    let shown_if = |has_faces: bool| {
        if has_faces {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    let mut solid_meshes: Vec<Option<Mesh>> = built.solid.into_iter().map(Some).collect();
    for (part, handle, mut visibility) in &mut solid {
        let i = part.0 as usize;
        visibility.set_if_neq(shown_if(built.has_solid[i]));
        // An empty mesh is left as it was (hidden): Bevy warns about meshes
        // with no vertices.
        if !built.has_solid[i] {
            continue;
        }
        if let (Some(mut stored), Some(mesh)) = (meshes.get_mut(&handle.0), solid_meshes[i].take())
        {
            *stored = mesh;
        }
    }
    let (see_through_mesh, mut see_through_visibility) = see_through.into_inner();
    see_through_visibility.set_if_neq(shown_if(built.has_see_through));
    if built.has_see_through
        && let Some(mut stored) = meshes.get_mut(&see_through_mesh.0)
    {
        *stored = built.see_through;
    }
    let ms = start.elapsed().as_secs_f32() * 1000.0;
    stats.summary = format!("{}: sight, light and meshes {ms:.2} ms", dungeon.describe());
}

/// A left click on a seen walkable cell sends the player travelling there (in
/// a straight line for now: walls don't stop travel until there is
/// pathfinding). Clicks on clickable tokens are left to `site`; so is using
/// stairs, by arriving on their cell.
fn pick_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    clicked: Res<ClickedToken>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    mut travel: MessageWriter<TravelTo>,
) {
    if !mouse.just_pressed(MouseButton::Left) || clicked.0.is_some() {
        return;
    }
    let (Some(cursor), Some(sight)) = (window.cursor_position(), &shown.sight) else {
        return;
    };
    let (camera, camera_transform) = *camera;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    if let Some((x, y)) = picking::pick(
        ray,
        &dungeon.level,
        sight,
        &shown.solid_faces,
        &shown.stairs_up,
    ) {
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
