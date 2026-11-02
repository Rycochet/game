//! Bags on the ground of a site: where the server says there are some, or where the
//! character remembers them, with a sparkle behind them while in sight. The grid of
//! what is in one is `inventory`'s. See `docs/context/client/ground.md`.

mod bag;

use std::collections::BTreeMap;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, Face, TextureDimension, TextureFormat};
use shared::ground::Load;
use shared::protocol::ServerMessage;

use crate::common::{ClickMode, HoveredBag, MapPosition, PointerPosition};
use crate::connection::FromServer;
use crate::dungeon_map::{CELL_SIZE, Shown};
use crate::inventory::OpenCharacter;
use crate::player::{Arrived, Player, PlayerSystems, TakeStep, TravelTo};
use crate::view3d::{ClickedToken, View3dSystems, ViewCamera};
use bag::{Model, size_of};

/// A sparkle's main star: how long its points are, and how thin.
const STAR_LONG: f32 = 7.0;
const STAR_SHORT: f32 = 1.8;
/// A small glint beside it, as part of the main star's size.
const GLINT_SCALE: f32 = 0.4;
/// How big a sparkle is on the biggest bag, as part of those sizes; it shrinks with the bag.
const SPARKLE_SCALE: f32 = 0.5;
/// How far a star's outline stands out as drawn, always and while the pointer is on it,
/// so that shrinking doesn't lose it.
const STAR_OUTLINE: f32 = 0.2;
const STAR_HOVER_OUTLINE: f32 = 0.45;
/// How far a sparkle grows and shrinks, and how fast, by real time.
const TWINKLE: f32 = 0.18;
const TWINKLE_PER_SECOND: f32 = 2.2;
/// How fast it slowly turns.
const TURN_PER_SECOND: f32 = 0.35;
/// How far above the floor it never goes below, on the biggest bag.
const LOWEST_SPARKLE: f32 = 4.0;
/// A light gold, showing whatever the light.
const SPARKLE: Color = Color::srgb(1.0, 0.92, 0.55);
/// A star's outline.
const STAR_EDGE: Color = Color::BLACK;
/// How many pixels a side the weave's picture has.
const WEAVE_PIXELS: usize = 16;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Bags>()
            .init_resource::<HoveredBag>()
            .init_resource::<Heading>()
            .add_systems(Startup, make_assets)
            // After the steps there have gone, as a shop's door does.
            .add_systems(PostUpdate, open_on_arrival)
            .add_systems(
                Update,
                (
                    read_server,
                    find_hovered.in_set(View3dSystems::Click),
                    (change_of_mind, follow_click)
                        .chain()
                        .in_set(PlayerSystems::Input)
                        .after(View3dSystems::Click),
                    show_bags
                        .after(read_server)
                        .after(PlayerSystems::Travel)
                        .after(View3dSystems::Place),
                ),
            );
        #[cfg(not(target_arch = "wasm32"))]
        app.add_message::<GroundDebug>()
            .add_systems(Update, run_debug.before(read_server));
    }
}

/// For looking at the bags (`dev_tools`).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Message, Clone, Copy, Debug)]
pub enum GroundDebug {
    /// A row of every look, starting from the player's cell and going east.
    ShowLooks,
}

/// The bags known of on this level, by cell, as last seen.
#[derive(Resource, Default)]
struct Bags(BTreeMap<(i64, i64), Bag>);

struct Bag {
    load: Load,
    root: Entity,
}

/// The bag the player is walking to, to open on arrival.
#[derive(Resource, Default)]
struct Heading(Option<(i64, i64)>);

/// What a bag is drawn with.
#[derive(Resource)]
struct BagAssets {
    /// By look and variety: the bag in colour, in grey and its outlines.
    bags: BTreeMap<(Load, u8), BagMeshes>,
    star: Handle<Mesh>,
    lit: Handle<StandardMaterial>,
    remembered: Handle<StandardMaterial>,
    sparkle: Handle<StandardMaterial>,
    outline: Handle<StandardMaterial>,
    star_edge: Handle<StandardMaterial>,
}

struct BagMeshes {
    lit: Handle<Mesh>,
    remembered: Handle<Mesh>,
    outline: Handle<Mesh>,
    hover_outline: Handle<Mesh>,
    star_outline: Handle<Mesh>,
    star_hover_outline: Handle<Mesh>,
}

/// How big a look's sparkle is drawn.
fn sparkle_scale(load: Load) -> f32 {
    SPARKLE_SCALE * size_of(load).scale
}

/// How far up a look's sparkle stands.
fn sparkle_up(load: Load) -> f32 {
    let size = size_of(load);
    (size.height * 0.85).max(LOWEST_SPARKLE * size.scale)
}

/// What depends on the state of the bag at this cell.
#[derive(Component, Clone, Copy)]
enum BagPart {
    /// Shows while it is in sight.
    Lit,
    /// Shows while it is out of sight.
    Remembered,
    /// Shows while it is hovered: wider than the outline it always has.
    HoverOutline,
    /// Shows while it is hovered and in sight: the same for its sparkle.
    SparkleHoverOutline,
}

/// On each of a bag's parts: whose it is.
#[derive(Component, Clone, Copy)]
struct OfBag((i64, i64));

/// A sparkle, standing behind its bag and facing the camera.
#[derive(Component, Clone, Copy)]
struct Sparkle {
    cell: (i64, i64),
    /// How far up it stands, how far behind the middle, and how big it is.
    up: f32,
    behind: f32,
    scale: f32,
    /// So that bags don't twinkle in step.
    phase: f32,
}

/// One star of a sparkle: how big it is, and how far out of step with the others.
#[derive(Component, Clone, Copy)]
struct Star {
    scale: f32,
    phase: f32,
}

fn to_mesh(model: Model) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, model.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, model.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, model.uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, model.colours)
    .with_inserted_indices(Indices::U32(model.indices))
}

/// Canvas: threads over and under, near enough white for its colour to come from the
/// mesh, drawn here so there is no file to ship for it.
fn weave_picture() -> Image {
    let mut bytes = Vec::with_capacity(WEAVE_PIXELS * WEAVE_PIXELS * 4);
    for y in 0..WEAVE_PIXELS {
        for x in 0..WEAVE_PIXELS {
            let over = (x / 2 + y / 2).is_multiple_of(2);
            let edge = if over { x % 2 == 0 } else { y % 2 == 0 };
            let shade = match (over, edge) {
                (true, false) => 255,
                (false, false) => 228,
                (_, true) => 200,
            };
            bytes.extend([shade, shade, shade, 255]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: WEAVE_PIXELS as u32,
            height: WEAVE_PIXELS as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..default()
    });
    image
}

fn make_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut bags = BTreeMap::new();
    for load in Load::ALL {
        let scale = sparkle_scale(load);
        for variety in 0..bag::VARIETIES {
            let model = bag::bag(load, variety);
            let star_outline = bag::star(STAR_LONG, STAR_SHORT, STAR_OUTLINE / scale);
            let star_hover_outline = bag::star(STAR_LONG, STAR_SHORT, STAR_HOVER_OUTLINE / scale);
            let remembered = meshes.add(to_mesh(model.greyed()));
            let outline = meshes.add(to_mesh(bag::outline(&model)));
            let hover_outline = meshes.add(to_mesh(bag::hover_outline(&model)));
            let lit = meshes.add(to_mesh(model));
            bags.insert(
                (load, variety),
                BagMeshes {
                    lit,
                    remembered,
                    outline,
                    hover_outline,
                    star_outline: meshes.add(to_mesh(star_outline)),
                    star_hover_outline: meshes.add(to_mesh(star_hover_outline)),
                },
            );
        }
    }
    let weave = images.add(weave_picture());
    commands.insert_resource(BagAssets {
        bags,
        star: meshes.add(to_mesh(bag::star(STAR_LONG, STAR_SHORT, 0.0))),
        lit: materials.add(StandardMaterial {
            base_color_texture: Some(weave.clone()),
            perceptual_roughness: 0.95,
            ..default()
        }),
        remembered: materials.add(StandardMaterial {
            base_color_texture: Some(weave),
            unlit: true,
            ..default()
        }),
        sparkle: materials.add(StandardMaterial {
            base_color: SPARKLE,
            unlit: true,
            ..default()
        }),
        // Drawn from inside, so only what stands out past the bag shows.
        outline: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            cull_mode: Some(Face::Front),
            ..default()
        }),
        star_edge: materials.add(StandardMaterial {
            base_color: STAR_EDGE,
            unlit: true,
            cull_mode: None,
            ..default()
        }),
    });
}

/// A number from a cell, the same each time, for what differs from bag to bag.
fn cell_hash(cell: (i64, i64)) -> u32 {
    let mixed = (cell.0 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (cell.1 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    (mixed >> 32) as u32
}

fn spawn_bag(commands: &mut Commands, assets: &BagAssets, cell: (i64, i64), load: Load) -> Entity {
    let hash = cell_hash(cell);
    let variety = (hash % bag::VARIETIES as u32) as u8;
    let meshes = &assets.bags[&(load, variety)];
    let size = size_of(load);
    let turned = Quat::from_rotation_y((hash >> 8) as f32 % 8.0 * std::f32::consts::FRAC_PI_4);
    let of_bag = OfBag(cell);
    let place = Transform::from_xyz(cell.0 as f32 * CELL_SIZE, 0.0, cell.1 as f32 * CELL_SIZE);
    let body = Transform::from_rotation(turned);
    let phase = (hash >> 16) as f32 % 100.0;
    let root = commands.spawn((place, Visibility::Inherited)).id();
    let hidden = Visibility::Hidden;
    let parts = [
        commands
            .spawn((
                BagPart::Lit,
                of_bag,
                Mesh3d(meshes.lit.clone()),
                MeshMaterial3d(assets.lit.clone()),
                body,
                hidden,
            ))
            .id(),
        commands
            .spawn((
                BagPart::Remembered,
                of_bag,
                Mesh3d(meshes.remembered.clone()),
                MeshMaterial3d(assets.remembered.clone()),
                body,
                hidden,
            ))
            .id(),
        commands
            .spawn((
                Mesh3d(meshes.outline.clone()),
                MeshMaterial3d(assets.outline.clone()),
                body,
            ))
            .id(),
        commands
            .spawn((
                BagPart::HoverOutline,
                of_bag,
                Mesh3d(meshes.hover_outline.clone()),
                MeshMaterial3d(assets.outline.clone()),
                body,
                hidden,
            ))
            .id(),
    ];
    let sparkle = commands
        .spawn((
            Sparkle {
                cell,
                up: sparkle_up(load),
                behind: size.radius,
                scale: sparkle_scale(load),
                phase,
            },
            of_bag,
            Transform::default(),
            hidden,
        ))
        .id();
    // The main star, and glints off to each side of it.
    for (scale, offset, star_phase) in [
        (1.0, Vec3::ZERO, 0.0),
        (GLINT_SCALE, Vec3::new(5.5, 4.5, 0.2), 1.9),
        (GLINT_SCALE * 0.8, Vec3::new(-6.0, 2.5, 0.2), 3.7),
    ] {
        let star = commands
            .spawn((
                Star {
                    scale,
                    phase: star_phase,
                },
                Mesh3d(assets.star.clone()),
                MeshMaterial3d(assets.sparkle.clone()),
                Transform::from_translation(offset).with_scale(Vec3::splat(scale)),
                ChildOf(sparkle),
            ))
            .id();
        // Just behind it.
        commands.spawn((
            Mesh3d(meshes.star_outline.clone()),
            MeshMaterial3d(assets.star_edge.clone()),
            Transform::from_xyz(0.0, 0.0, -0.5),
            ChildOf(star),
        ));
        commands.spawn((
            BagPart::SparkleHoverOutline,
            of_bag,
            Mesh3d(meshes.star_hover_outline.clone()),
            MeshMaterial3d(assets.star_edge.clone()),
            Transform::from_xyz(0.0, 0.0, -0.6),
            hidden,
            ChildOf(star),
        ));
    }
    commands
        .entity(root)
        .add_children(&parts)
        .add_child(sparkle);
    root
}

/// Puts a bag in the list, or changes how it looks.
fn set_bag(
    bags: &mut Bags,
    commands: &mut Commands,
    assets: &BagAssets,
    cell: (i64, i64),
    load: Load,
) {
    if let Some(old) = bags.0.get(&cell) {
        if old.load == load {
            return;
        }
        commands.entity(old.root).despawn();
    }
    let root = spawn_bag(commands, assets, cell, load);
    bags.0.insert(cell, Bag { load, root });
}

/// What the server says of bags. Anything out of sight isn't mentioned, so it stays as
/// remembered. A new map forgets them all: the server says again what is remembered.
fn read_server(
    mut from_server: MessageReader<FromServer>,
    assets: Res<BagAssets>,
    mut bags: ResMut<Bags>,
    mut commands: Commands,
) {
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::GroundSeen { x, y, load } => {
                set_bag(&mut bags, &mut commands, &assets, (*x, *y), *load);
            }
            ServerMessage::GroundGone { x, y } => {
                if let Some(old) = bags.0.remove(&(*x, *y)) {
                    commands.entity(old.root).despawn();
                }
            }
            ServerMessage::EnterLevel(_)
            | ServerMessage::OnWorldMap { .. }
            | ServerMessage::Teleported { .. } => {
                for (_, old) in std::mem::take(&mut bags.0) {
                    commands.entity(old.root).despawn();
                }
            }
            _ => {}
        }
    }
}

/// Which bag the pointer is on, measured on screen, nearest the camera first. Its
/// sparkle counts while it is showing.
fn find_hovered(
    pointer: Res<PointerPosition>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    bags: Res<Bags>,
    shown: Res<Shown>,
    mut hovered: ResMut<HoveredBag>,
) {
    let (camera, transform) = *camera;
    let found = pointer.0.and_then(|cursor| {
        let mut best: Option<(f32, (i64, i64))> = None;
        for (&cell, bag) in &bags.0 {
            let size = size_of(bag.load);
            let foot = Vec3::new(cell.0 as f32 * CELL_SIZE, 0.0, cell.1 as f32 * CELL_SIZE);
            let in_sight = shown
                .sight
                .as_ref()
                .is_some_and(|sight| sight.is_seen(cell.0, cell.1));
            let tallest = if in_sight {
                size.height
                    .max(sparkle_up(bag.load) + STAR_LONG * sparkle_scale(bag.load))
            } else {
                size.height
            };
            let top = foot + Vec3::Y * tallest;
            let side = foot + transform.right() * size.radius;
            let (Ok(foot_at), Ok(top_at), Ok(side_at)) = (
                camera.world_to_viewport(transform, foot),
                camera.world_to_viewport(transform, top),
                camera.world_to_viewport(transform, side),
            ) else {
                continue;
            };
            let reach = foot_at.distance(side_at);
            let on_it = (cursor.x - foot_at.x).abs() <= reach
                && cursor.y >= top_at.y - reach / 2.0
                && cursor.y <= foot_at.y + reach / 2.0;
            let distance = transform.translation().distance(foot);
            if on_it && best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, cell));
            }
        }
        best.map(|(_, cell)| cell)
    });
    hovered.set_if_neq(HoveredBag(found));
}

/// Walking anywhere else, or a step of their own: they aren't going to the bag.
fn change_of_mind(
    mut steps: MessageReader<TakeStep>,
    mut journeys: MessageReader<TravelTo>,
    mut heading: ResMut<Heading>,
) {
    let stepped = steps.read().count() > 0;
    let elsewhere = journeys
        .read()
        .any(|journey| Some((journey.x, journey.y)) != heading.0);
    if stepped || elsewhere {
        heading.0 = None;
    }
}

/// A click on a bag opens the Character panel if the player is on it, or else walks
/// them there to open it on arriving. A tap arrives as a click.
#[allow(clippy::too_many_arguments)]
fn follow_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mode: Res<ClickMode>,
    clicked: Res<ClickedToken>,
    hovered: Res<HoveredBag>,
    player: Single<&MapPosition, With<Player>>,
    mut heading: ResMut<Heading>,
    mut travel: MessageWriter<TravelTo>,
    mut open: MessageWriter<OpenCharacter>,
) {
    let Some(cell) = hovered.0 else {
        return;
    };
    if !mouse.just_pressed(MouseButton::Left) || clicked.0.is_some() || *mode != ClickMode::Normal {
        return;
    }
    if (player.x, player.y) == cell {
        heading.0 = None;
        open.write(OpenCharacter);
    } else {
        heading.0 = Some(cell);
        travel.write(TravelTo {
            x: cell.0,
            y: cell.1,
        });
    }
}

fn open_on_arrival(
    mut arrivals: MessageReader<Arrived>,
    mut heading: ResMut<Heading>,
    bags: Res<Bags>,
    mut open: MessageWriter<OpenCharacter>,
) {
    let Some(&Arrived { x, y }) = arrivals.read().last() else {
        return;
    };
    if heading.0.take() == Some((x, y)) && bags.0.contains_key(&(x, y)) {
        open.write(OpenCharacter);
    }
}

/// Shows each bag as it is now: in colour with its sparkle while in sight, or in grey
/// without one, as last seen; a wider outline if the pointer is on it.
#[allow(clippy::type_complexity)]
fn show_bags(
    shown: Res<Shown>,
    hovered: Res<HoveredBag>,
    time: Res<Time<Real>>,
    camera: Single<&GlobalTransform, With<ViewCamera>>,
    mut parts: Query<(&BagPart, &OfBag, &mut Visibility), Without<Sparkle>>,
    mut sparkles: Query<(&Sparkle, &mut Transform, &mut Visibility), Without<Star>>,
    mut stars: Query<(&Star, &ChildOf, &mut Transform), Without<Sparkle>>,
) {
    let in_sight = |cell: (i64, i64)| {
        let sight = shown.sight.as_ref();
        sight.is_some_and(|sight| sight.is_seen(cell.0, cell.1))
    };
    for (part, of_bag, mut visibility) in &mut parts {
        let seen = in_sight(of_bag.0);
        let over = hovered.0 == Some(of_bag.0);
        let show = match part {
            BagPart::Lit => seen,
            BagPart::Remembered => !seen,
            BagPart::HoverOutline => over,
            BagPart::SparkleHoverOutline => over && seen,
        };
        visibility.set_if_neq(if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }

    let now = time.elapsed_secs();
    let camera_turn = camera.rotation();
    let forward = camera.forward();
    // Straight behind on the ground, so that it shows above the bag.
    let behind = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    for (sparkle, mut transform, mut visibility) in &mut sparkles {
        visibility.set_if_neq(if in_sight(sparkle.cell) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        transform.translation = Vec3::Y * sparkle.up + behind * sparkle.behind;
        transform.rotation = camera_turn;
        transform.scale = Vec3::splat(sparkle.scale);
    }
    for (star, parent, mut transform) in &mut stars {
        let Ok((sparkle, ..)) = sparkles.get(parent.parent()) else {
            continue;
        };
        let at = now + sparkle.phase;
        let pulse = 1.0 + TWINKLE * (at * TWINKLE_PER_SECOND + star.phase).sin();
        transform.scale = Vec3::splat(star.scale * pulse);
        transform.rotation = Quat::from_rotation_z(at * TURN_PER_SECOND + star.phase);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_debug(
    mut asked: MessageReader<GroundDebug>,
    assets: Res<BagAssets>,
    player: Single<&MapPosition, With<Player>>,
    mut bags: ResMut<Bags>,
    mut commands: Commands,
) {
    for GroundDebug::ShowLooks in asked.read() {
        for (index, load) in Load::ALL.into_iter().enumerate() {
            let cell = (player.x + index as i64, player.y);
            set_bag(&mut bags, &mut commands, &assets, cell, load);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bevy only finds two parameters clashing when a system starts.
    #[test]
    fn no_system_asks_for_what_another_of_its_parameters_has() {
        fn starts<M>(system: impl IntoSystem<(), (), M>) {
            IntoSystem::into_system(system).initialize(&mut World::new());
        }
        starts(make_assets);
        starts(read_server);
        starts(find_hovered);
        starts(change_of_mind);
        starts(follow_click);
        starts(open_on_arrival);
        starts(show_bags);
    }

    #[test]
    fn neighbouring_cells_are_told_apart() {
        let hashes: std::collections::BTreeSet<u32> =
            (0..8).map(|x| cell_hash((x, 3)) % 8).collect();
        assert!(hashes.len() > 1);
    }
}
