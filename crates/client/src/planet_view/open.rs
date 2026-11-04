//! Opening the view: a black layer over everything with the globe's picture
//! in the middle, and the scene that picture is drawn from.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::ViewPlanet;
use super::clouds::{PlanetCloudMaterial, spawn_clouds};
use super::globe::{Scene, spawn_scene};
use super::sky::Sky;
use super::texture::{new_texture, texture_size};
use crate::common::{CurrentWorld, ServerClock, Weather};
use crate::widgets::layer_z;

/// Above the menus and their layers.
const VIEW_LAYER: i32 = 50;
/// The picture's least and greatest size in pixels.
const SMALLEST_PICTURE: u32 = 256;
const LARGEST_PICTURE: u32 = 2048;
/// The note at the bottom.
const NOTE: &str = "Click, tap or press Esc to close";
const NOTE_COLOUR: Color = Color::srgb(0.55, 0.55, 0.6);

/// Whether the planet is on show, and what it is made of.
#[derive(Resource, Default)]
pub(super) struct PlanetView {
    pub(super) showing: Option<Showing>,
}

pub(super) struct Showing {
    /// The black layer and everything on it.
    pub(super) layer: Entity,
    pub(super) scene: Scene,
    /// The clouds' material, which `clouds::move_clouds` keeps up to date.
    pub(super) clouds: Handle<PlanetCloudMaterial>,
    pub(super) sky: Sky,
    /// The surface's picture, its size and how many rows are filled.
    pub(super) texture: Handle<Image>,
    pub(super) size: UVec2,
    pub(super) rows_done: u32,
    /// The water and snow lines it is coloured for: the sea at mid-tide, and
    /// the snow line as it was on opening.
    pub(super) water: f64,
    pub(super) snow_line: f64,
    /// The world's hour on opening, and the real time then.
    pub(super) start_hours: f64,
    pub(super) opened: f64,
}

/// The stores the scene's meshes, materials and pictures go in.
#[derive(SystemParam)]
pub(super) struct SceneAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    clouds: ResMut<'w, Assets<PlanetCloudMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn open(
    mut asked: MessageReader<ViewPlanet>,
    mut view: ResMut<PlanetView>,
    world: Option<Res<CurrentWorld>>,
    clock: Res<ServerClock>,
    weather: Res<Weather>,
    window: Single<&Window, With<PrimaryWindow>>,
    time: Res<Time<Real>>,
    mut commands: Commands,
    mut assets: SceneAssets,
) {
    if asked.read().count() == 0 || view.showing.is_some() {
        return;
    }
    let Some(world) = world else {
        return;
    };
    let size = texture_size(world.surface.width());
    let texture = assets.images.add(new_texture(size));
    let shortest = window.physical_width().min(window.physical_height());
    let pixels = shortest.clamp(SMALLEST_PICTURE, LARGEST_PICTURE);
    let picture = assets.images.add(Image::new_target_texture(
        pixels,
        pixels,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    let scene = spawn_scene(
        &mut commands,
        &world,
        texture.clone(),
        picture.clone(),
        (
            &mut assets.meshes,
            &mut assets.materials,
            &mut assets.images,
        ),
    );
    let clouds = spawn_clouds(
        &mut commands,
        scene.globe,
        world.params.seed,
        (&mut assets.meshes, &mut assets.clouds),
    );
    view.showing = Some(Showing {
        layer: spawn_layer(&mut commands, picture),
        scene,
        clouds,
        sky: Sky::new(&world.celestial),
        texture,
        size,
        rows_done: 0,
        water: world.params.sea_level,
        snow_line: weather.snow_line,
        start_hours: clock.hours(),
        opened: time.elapsed_secs_f64(),
    });
}

/// The black layer, with the picture in the middle as large as fits and a
/// note at the bottom.
fn spawn_layer(commands: &mut Commands, picture: Handle<Image>) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(layer_z(VIEW_LAYER)),
        ))
        .with_children(|layer| {
            layer.spawn((
                ImageNode::new(picture),
                Node {
                    width: Val::VMin(100.0),
                    height: Val::VMin(100.0),
                    ..default()
                },
            ));
            layer.spawn((
                Text::new(NOTE),
                TextColor(NOTE_COLOUR),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(16),
                    ..default()
                },
            ));
        })
        .id()
}
