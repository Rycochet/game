//! The dungeon (and building) view: a grid of floor and wall cells in 3D,
//! seen through `view3d`'s camera with the same tokens as the world map. See
//! `docs/context/client/dungeon_map.md`.
//!
//! Only what the player (or anything sharing its sight) can see is drawn:
//! floors, stairs, and thin walls with only the faces someone sees (`walls`).
//! A seen side facing away from the camera is drawn half see-through. What
//! was seen before but is out of sight now is drawn dim, grey and half
//! see-through. Walls and floors use tile pictures chosen by the kind of place
//! (`theme`). Light is dull everywhere, and brighter near light sources, which
//! walls block. Left-click a seen walkable cell to travel there.

mod field;
mod herbs;
pub use field::CROP_STAGES;
pub use herbs::HERB_STAGES;
mod holes;
mod liquid;
mod mesh;
mod picking;
mod rain;
mod stairs;
mod theme;
mod town;
mod walls;

pub use theme::TOWN_ATLAS;
pub use town::door_picture;

use std::collections::BTreeMap;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::light::NotShadowCaster;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use shared::dungeon::{Cell, Sight};
use shared::poi::PoiKind;
use shared::protocol::ClientMessage;
use shared::surface::SurfaceSample;

use crate::common::{
    ClickMode, CurrentDungeon, CurrentWorld, GameClock, GameView, HerbGrowthShift, LightSource,
    MapPosition, PointerPosition, SharesSight, SmoothPosition, ViewStats, Weather, WorldReturn,
};
use crate::connection::ToServer;
use crate::player::{Player, PlayerSystems, TravelTo};
use crate::surface_style::SurfaceStyle;
use crate::view3d::{ClickedToken, TOKEN_DIAMETER, TokenView, View3dSystems, ViewCamera, ViewGrid};
use mesh::{LightGrid, Surface, View, WallFace};
use theme::Theme;

/// Size of a cell in 3D units: about the size of a token's picture.
pub const CELL_SIZE: f32 = TOKEN_DIAMETER * 1.5;

/// How tall walls stand.
const WALL_HEIGHT: f32 = CELL_SIZE;

/// The dull light everywhere, as a fraction of full light.
const AMBIENT_LIGHT: f32 = 0.1;

/// Opacity of wall sides facing away from the camera.
const SEE_THROUGH_ALPHA: f32 = 0.5;

/// Opacity of what is remembered but out of sight.
const MEMORY_ALPHA: f32 = 0.5;

/// Draws a wall's fade across the floor after the floor, before the walls.
const BLEND_DEPTH_BIAS: f32 = -0.5;

/// A town's fixed light on tokens: it is out in the open.
const TOWN_AMBIENT_BRIGHTNESS: f32 = 1500.0;

/// Tokens are lit by real lights (the tiles' light is baked into their
/// colours): a point light this high above each light source, and a dim
/// ambient fill so tokens in the dark are still visible.
const TOKEN_LIGHT_HEIGHT: f32 = CELL_SIZE;
const TOKEN_LIGHT_LUMENS: f32 = 1.0e8;
const TOKEN_AMBIENT_BRIGHTNESS: f32 = 150.0;

pub struct DungeonMapPlugin;

impl Plugin for DungeonMapPlugin {
    fn build(&self, app: &mut App) {
        let in_view = in_state(GameView::Dungeon);
        app.init_resource::<Shown>()
            .add_plugins(MaterialPlugin::<rain::RainMaterial>::default())
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
                (
                    apply_theme,
                    colour_ground,
                    update_meshes,
                    update_crops,
                    update_herbs,
                    hide_unseen_tokens,
                    sync_lights,
                    rain::update,
                )
                    .chain()
                    .after(PlayerSystems::Travel)
                    .after(View3dSystems::Place)
                    .run_if(in_view),
            )
            .add_systems(Update, grey_pictures);
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
    /// Counts the times sight was worked out again, or the level changed.
    looks: u64,
}

/// The materials of the solid meshes (in `Surface::ALL` order), of the
/// remembered ones and of the see-through walls, so their pictures can
/// follow the theme.
#[derive(Resource)]
struct SurfaceMaterials {
    solid: Vec<Handle<StandardMaterial>>,
    memory: Vec<Handle<StandardMaterial>>,
    see_through: Handle<StandardMaterial>,
    /// Of stairs in the way of a viewer, in `Surface::ALL` order.
    stairs_through: Vec<Handle<StandardMaterial>>,
    /// The outside ground's picture as drawn, for `colour_ground` to copy.
    ground_picture: Handle<Image>,
    /// Pictures waiting to load so a grey copy can be made for the
    /// remembered surface, by surface index.
    to_grey: Vec<(usize, Handle<Image>)>,
    /// Grey copies already made, by the picture they copy.
    greys: BTreeMap<AssetId<Image>, Handle<Image>>,
}

/// The mesh of one kind of surface (floors, solid walls...).
#[derive(Component)]
struct SolidPart(Surface);

/// The mesh of one kind of surface remembered but out of sight.
#[derive(Component)]
struct MemoryPart(Surface);

/// The mesh of half see-through walls.
#[derive(Component)]
struct SeeThroughPart;

/// The mesh of one kind of surface of stairs in the way of a viewer.
#[derive(Component)]
struct StairsThroughPart(Surface);

/// The mesh of crops standing in fields.
#[derive(Component)]
struct CropPart;

/// The mesh of what grows in a herbalist's patch.
#[derive(Component)]
struct HerbPart;

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
    mut rain_materials: ResMut<Assets<rain::RainMaterial>>,
) {
    let theme = Theme::of(None);
    let pictures: Vec<Handle<Image>> = Surface::ALL
        .iter()
        .map(|&surface| load_tile(&asset_server, theme.picture(surface)))
        .collect();
    // Unlit: the light is in the vertex colours, worked out on the grid.
    // Memory shows the coloured pictures until their grey copies are made.
    let unlit = |picture: &Handle<Image>| StandardMaterial {
        base_color_texture: Some(picture.clone()),
        unlit: true,
        ..default()
    };
    let solid: Vec<_> = Surface::ALL
        .iter()
        .zip(&pictures)
        .map(|(&surface, picture)| {
            let opacity = liquid::opacity(surface);
            materials.add(if surface == Surface::WallBlend {
                StandardMaterial {
                    alpha_mode: AlphaMode::Blend,
                    depth_bias: BLEND_DEPTH_BIAS,
                    ..unlit(picture)
                }
            } else if opacity < 1.0 {
                // A surface with the bed showing through it, drawn before
                // anything blended that stands in it.
                StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, opacity),
                    alpha_mode: AlphaMode::Blend,
                    depth_bias: -1.0,
                    ..unlit(picture)
                }
            } else {
                unlit(picture)
            })
        })
        .collect();
    // Blended meshes don't sort against each other by themselves: floors are
    // pushed back so they are always drawn before the walls standing on them,
    // with the wall's fade across the floor in between.
    let memory: Vec<_> = Surface::ALL
        .iter()
        .zip(&pictures)
        .map(|(&surface, picture)| {
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, MEMORY_ALPHA),
                // Solid even as memory: a whole town is remembered at once,
                // and blended meshes wouldn't sort.
                alpha_mode: if surface == Surface::Town {
                    AlphaMode::Opaque
                } else {
                    AlphaMode::Blend
                },
                depth_bias: match surface {
                    Surface::Wall | Surface::Town => 0.0,
                    Surface::WallBlend => BLEND_DEPTH_BIAS,
                    _ => -1.0,
                },
                ..unlit(picture)
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
    // Back faces stay culled, so a model doesn't show its far side as well.
    let stairs_through: Vec<_> = pictures
        .iter()
        .map(|picture| {
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, SEE_THROUGH_ALPHA),
                alpha_mode: AlphaMode::Blend,
                ..unlit(picture)
            })
        })
        .collect();
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
    for (&surface, material) in Surface::ALL.iter().zip(&memory) {
        commands.entity(scene).with_child((
            MemoryPart(surface),
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
            MeshMaterial3d(material.clone()),
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
    for (&surface, material) in Surface::ALL.iter().zip(&stairs_through) {
        commands.entity(scene).with_child((
            StairsThroughPart(surface),
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
            MeshMaterial3d(material.clone()),
            NoFrustumCulling,
            NotShadowCaster,
        ));
    }
    commands.entity(scene).with_child((
        CropPart,
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(load_tile(&asset_server, field::CROPS_PICTURE)),
            unlit: true,
            alpha_mode: AlphaMode::Mask(0.5),
            // Seen from both sides.
            cull_mode: None,
            ..default()
        })),
        NoFrustumCulling,
        NotShadowCaster,
    ));
    commands.entity(scene).with_child((
        HerbPart,
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(load_tile(&asset_server, herbs::HERBS_PICTURE)),
            unlit: true,
            alpha_mode: AlphaMode::Mask(0.5),
            // Seen from both sides.
            cull_mode: None,
            ..default()
        })),
        NoFrustumCulling,
        NotShadowCaster,
    ));
    rain::spawn(&mut commands, scene, &mut meshes, &mut rain_materials);
    commands.insert_resource(SurfaceMaterials {
        solid,
        memory,
        see_through,
        stairs_through,
        ground_picture: pictures[Surface::Ground as usize].clone(),
        to_grey: pictures.into_iter().enumerate().collect(),
        greys: BTreeMap::new(),
    });
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
    mut commands: Commands,
    dungeon: Res<CurrentDungeon>,
    asset_server: Res<AssetServer>,
    mut surfaces: ResMut<SurfaceMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !dungeon.is_changed() {
        return;
    }
    commands.insert_resource(GlobalAmbientLight {
        brightness: if dungeon.is_town() {
            TOWN_AMBIENT_BRIGHTNESS
        } else {
            TOKEN_AMBIENT_BRIGHTNESS
        },
        ..default()
    });
    let theme = Theme::of(dungeon.info.as_ref().map(|info| info.kind));
    for surface in [Surface::Floor, Surface::Wall, Surface::WallBlend] {
        let picture = load_tile(&asset_server, theme.picture(surface));
        if let Some(mut material) = materials.get_mut(&surfaces.solid[surface as usize]) {
            material.base_color_texture = Some(picture.clone());
        }
        if let Some(mut material) = materials.get_mut(&surfaces.stairs_through[surface as usize]) {
            material.base_color_texture = Some(picture.clone());
        }
        surfaces.to_grey.push((surface as usize, picture));
    }
    if let Some(mut material) = materials.get_mut(&surfaces.see_through) {
        material.base_color_texture = Some(load_tile(&asset_server, theme.wall));
    }
}

/// Paints the outside ground the colour of the land the site stands on (its
/// biome, or snow), once the picture has loaded and whenever that changes.
#[allow(clippy::too_many_arguments)]
fn colour_ground(
    world: Option<Res<CurrentWorld>>,
    place: Res<WorldReturn>,
    style: Res<SurfaceStyle>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    surfaces: Res<SurfaceMaterials>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut painted: Local<Option<[u8; 4]>>,
) {
    let Some(world) = world else {
        return;
    };
    let MapPosition { x, y } = place.0;
    // No jitter: it only roughens edges on the map, and here one point's would
    // paint the whole site.
    let sample = SurfaceSample {
        jitter: 0.0,
        ..world.surface.sample(x, y)
    };
    let climate = world.climate.sample(x, y, sample.height);
    let conditions = world.conditions(&clock, &weather);
    let colour = style.ground_colour(sample, climate, conditions.water, conditions.snow_line);
    if *painted == Some(colour) {
        return;
    }
    let Some(picture) = images.get(&surfaces.ground_picture) else {
        return;
    };
    // Made first: `images` can't be added to while borrowed.
    let copy = recoloured(picture, colour);
    let copy = images.add(copy);
    let ground = Surface::Ground as usize;
    for handle in [&surfaces.solid[ground], &surfaces.stairs_through[ground]] {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base_color_texture = Some(copy.clone());
        }
    }
    *painted = Some(colour);
}

/// A copy of an 8-bit RGBA picture in one colour, keeping its detail: each
/// pixel is the colour, lighter or darker as the pixel is against the picture's
/// average (anything else is copied as it is).
fn recoloured(image: &Image, colour: [u8; 4]) -> Image {
    let mut copy = image.clone();
    let rgba = matches!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm
    );
    if let (true, Some(data)) = (rgba, copy.data.as_mut()) {
        let luma = |pixel: &[u8; 4]| {
            let [r, g, b] = [pixel[0], pixel[1], pixel[2]].map(f32::from);
            0.299 * r + 0.587 * g + 0.114 * b
        };
        let pixels = data.as_chunks_mut::<4>().0;
        let total: f32 = pixels.iter().map(luma).sum();
        let average = (total / pixels.len().max(1) as f32).max(1.0);
        for pixel in pixels {
            let scale = luma(pixel) / average;
            for (channel, base) in pixel[..3].iter_mut().zip(colour) {
                *channel = (f32::from(base) * scale).clamp(0.0, 255.0) as u8;
            }
        }
    }
    copy
}

/// Gives remembered surfaces grey copies of their pictures, once the
/// pictures have loaded (each is copied once).
fn grey_pictures(
    mut surfaces: ResMut<SurfaceMaterials>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if surfaces.to_grey.is_empty() {
        return;
    }
    let surfaces = &mut *surfaces;
    surfaces.to_grey.retain(|(index, picture)| {
        let grey = match surfaces.greys.get(&picture.id()) {
            Some(grey) => grey.clone(),
            None => {
                let Some(image) = images.get(picture) else {
                    // Not loaded yet: try again next frame.
                    return true;
                };
                // Made first: `images` can't be added to while borrowed.
                let copy = greyed(image);
                let grey = images.add(copy);
                surfaces.greys.insert(picture.id(), grey.clone());
                grey
            }
        };
        if let Some(mut material) = materials.get_mut(&surfaces.memory[*index]) {
            material.base_color_texture = Some(grey);
        }
        false
    });
}

/// A grey copy of an 8-bit RGBA picture (anything else is copied as it is).
fn greyed(image: &Image) -> Image {
    let mut grey = image.clone();
    let rgba = matches!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm
    );
    if let (true, Some(data)) = (rgba, grey.data.as_mut()) {
        for pixel in data.as_chunks_mut::<4>().0 {
            let [r, g, b] = [pixel[0], pixel[1], pixel[2]].map(f32::from);
            // Rec. 601 weights on the stored values: good enough for looks.
            let luma = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            pixel[..3].fill(luma);
        }
    }
    grey
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
/// is seen (and so remembered from now on) and how it is lit, and rebuilds
/// the meshes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_meshes(
    mut dungeon: ResMut<CurrentDungeon>,
    player: Single<Ref<MapPosition>, With<Player>>,
    sharers: Query<Ref<MapPosition>, (With<SharesSight>, Without<Player>)>,
    lights: Query<(Ref<MapPosition>, Ref<LightSource>)>,
    camera: Single<&Transform, With<ViewCamera>>,
    mut solid: Query<
        (&SolidPart, &Mesh3d, &mut Visibility),
        (Without<SeeThroughPart>, Without<MemoryPart>),
    >,
    mut memory: Query<
        (&MemoryPart, &Mesh3d, &mut Visibility),
        (Without<SeeThroughPart>, Without<SolidPart>),
    >,
    see_through: Single<(&Mesh3d, &mut Visibility), With<SeeThroughPart>>,
    mut stairs_through: Query<
        (&StairsThroughPart, &Mesh3d, &mut Visibility),
        (
            Without<SeeThroughPart>,
            Without<SolidPart>,
            Without<MemoryPart>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stats: ResMut<ViewStats>,
    mut shown: ResMut<Shown>,
    // The camera position last built for.
    mut built_for: Local<Option<Vec3>>,
    // What the viewers saw when last worked out, kept while only the camera
    // moves.
    mut seen: Local<Option<(Vec<View>, Sight)>>,
) {
    let camera = camera.translation;
    let looked_again = dungeon.is_changed()
        || player.is_changed()
        || sharers.iter().any(|position| position.is_changed());
    let changed = *built_for != Some(camera)
        || looked_again
        || lights
            .iter()
            .any(|(position, light)| position.is_changed() || light.is_changed());
    if !changed {
        return;
    }
    *built_for = Some(camera);

    let start = Instant::now();
    let level = &dungeon.level;
    if looked_again || seen.is_none() {
        let views: Vec<View> = std::iter::once(&**player)
            .chain(sharers.iter().map(|position| position.into_inner()))
            .map(|position| {
                let mut sight = Sight::new(level);
                sight.add_glimpses(level, position.x, position.y, None);
                View {
                    cell: (position.x, position.y),
                    sight,
                }
            })
            .collect();
        let mut sight = Sight::new(level);
        for view in &views {
            sight.add_glimpses(level, view.cell.0, view.cell.1, None);
        }
        *seen = Some((views, sight));
        shown.looks += 1;
    }
    let Some((views, sight)) = &*seen else {
        return;
    };
    // What is in sight is remembered from now on (only cells the server has
    // sent: the rest are wall here until it does). Not a change that needs
    // anything redrawn, so it doesn't mark the level changed.
    let mut remembered = dungeon.remembered.clone();
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if sight.is_seen(x, y) && dungeon.known.is_seen(x, y) {
                remembered.insert(x, y);
            }
        }
    }
    // Out in the open, a town has fixed light for now.
    let is_town = dungeon.is_town();
    let mut light = if is_town {
        LightGrid::filled(level, Vec3::ONE)
    } else {
        LightGrid::new(level)
    };
    for (position, source) in lights.iter().filter(|_| !is_town) {
        let colour = LinearRgba::from(source.colour);
        light.add_light(
            level,
            (position.x, position.y),
            source.radius,
            Vec3::new(colour.red, colour.green, colour.blue),
        );
    }
    if dungeon
        .info
        .as_ref()
        .is_some_and(|i| matches!(i.kind, PoiKind::Farm | PoiKind::HerbalistHut))
    {
        light.light_outside(level);
    }
    let ruined = dungeon.info.as_ref().is_some_and(|info| info.ruined);
    let theme = Theme::of(dungeon.info.as_ref().map(|info| info.kind));
    let built = mesh::build(
        level,
        views,
        sight,
        &remembered,
        &light,
        &dungeon.stairs,
        &dungeon.buildings,
        &dungeon.fields,
        ruined,
        camera,
        theme.shape,
    );
    shown.sight = Some(sight.clone());
    dungeon.bypass_change_detection().remembered = remembered;
    shown.solid_faces = built.solid_faces;

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
    let mut memory_meshes: Vec<Option<Mesh>> = built.memory.into_iter().map(Some).collect();
    for (part, handle, mut visibility) in &mut memory {
        let i = part.0 as usize;
        visibility.set_if_neq(shown_if(built.has_memory[i]));
        if !built.has_memory[i] {
            continue;
        }
        if let (Some(mut stored), Some(mesh)) = (meshes.get_mut(&handle.0), memory_meshes[i].take())
        {
            *stored = mesh;
        }
    }
    let mut through_meshes: Vec<Option<Mesh>> =
        built.stairs_through.into_iter().map(Some).collect();
    for (part, handle, mut visibility) in &mut stairs_through {
        let i = part.0 as usize;
        visibility.set_if_neq(shown_if(built.has_stairs_through[i]));
        if !built.has_stairs_through[i] {
            continue;
        }
        if let (Some(mut stored), Some(mesh)) =
            (meshes.get_mut(&handle.0), through_meshes[i].take())
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
    let place = dungeon.describe();
    let at_door = dungeon
        .door_facing(player.x, player.y)
        .map_or(String::new(), |b| {
            format!(", at the door of: {}", b.kind.name())
        });
    stats.summary = format!("{place}{at_door}: sight, light and meshes {ms:.2} ms");
}

/// Builds the crops again when what is seen changes: never for the camera, as
/// fields can hold many more faces than everything else put together.
fn update_crops(
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    part: Single<(&Mesh3d, &mut Visibility), With<CropPart>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut built: Local<u64>,
) {
    let Some(sight) = shown.sight.as_ref().filter(|_| *built != shown.looks) else {
        return;
    };
    *built = shown.looks;
    let ruined = dungeon.info.as_ref().is_some_and(|info| info.ruined);
    let crops = field::build_crops(&dungeon.fields, sight, &dungeon.remembered, ruined);
    let (handle, mut visibility) = part.into_inner();
    let any = !crops.indices.is_empty();
    visibility.set_if_neq(if any {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if any && let Some(mut stored) = meshes.get_mut(&handle.0) {
        *stored = crops.into_mesh();
    }
}

/// Builds what grows in a herbalist's patch again when what is seen changes,
/// or an hour passes: some tile may move on a stage in any of them.
#[allow(clippy::too_many_arguments)]
fn update_herbs(
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    clock: Res<GameClock>,
    shift: Res<HerbGrowthShift>,
    place: Res<WorldReturn>,
    part: Single<(&Mesh3d, &mut Visibility), With<HerbPart>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut built: Local<(u64, u32)>,
) {
    let grown = herbs::growth(clock.hour, shift.0);
    let now = (shown.looks, grown);
    let Some(sight) = shown.sight.as_ref().filter(|_| *built != now) else {
        return;
    };
    *built = now;
    let at = (place.0.x, place.0.y);
    let plants = herbs::build_plants(&dungeon.level, sight, &dungeon.remembered, at, grown);
    let (handle, mut visibility) = part.into_inner();
    let any = !plants.indices.is_empty();
    visibility.set_if_neq(if any {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if any && let Some(mut stored) = meshes.get_mut(&handle.0) {
        *stored = plants.into_mesh();
    }
}

/// A left click on a seen (or remembered) walkable cell sends the player
/// travelling there (in
/// a straight line for now: walls don't stop travel until there is
/// pathfinding). Clicks on clickable tokens are left to `site`; so is using
/// stairs, by arriving on their cell. In Teleport Mode it asks the server to teleport
/// the player there, and in Change Cell Mode it changes the cell (floor, wall,
/// door, window, and round again) to try walls out (dev tools).
#[allow(clippy::too_many_arguments)]
fn pick_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mode: Res<ClickMode>,
    clicked: Res<ClickedToken>,
    pointer: Res<PointerPosition>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    mut travel: MessageWriter<TravelTo>,
    mut to_server: MessageWriter<ToServer>,
) {
    if !mouse.just_pressed(MouseButton::Left) || clicked.0.is_some() {
        return;
    }
    let Some(cursor) = pointer.0 else {
        return;
    };
    let (camera, camera_transform) = *camera;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    if *mode == ClickMode::ChangeCell {
        if let Some((x, y)) = picking::floor_cell(ray) {
            let cell = match dungeon.level.cell(x, y) {
                Cell::Floor => Cell::Wall,
                Cell::Wall => Cell::Door,
                Cell::Door => Cell::Window,
                _ => Cell::Floor,
            };
            to_server.write(ToServer(ClientMessage::DevSetCell { x, y, cell }));
        }
        return;
    }
    let door = picking::door_front(ray, &dungeon.buildings, &shown.solid_faces);
    let picked = door
        .or_else(|| picking::pick(ray, &dungeon.level, &dungeon.remembered, &shown.solid_faces));
    if let Some((x, y)) = picked {
        // Dev tools: Teleport Mode jumps there instead (through walls), if the
        // server allows it.
        if *mode == ClickMode::Teleport {
            to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
        } else {
            travel.write(TravelTo { x, y });
        }
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
