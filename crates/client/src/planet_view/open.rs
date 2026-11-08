//! Opening the view: a black layer over everything with the globe's picture
//! in the middle, the bar along the top, and the scene that picture is drawn
//! from.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::ViewPlanet;
use super::bar::{Bar, spawn_bar};
use super::orbit::Orbit;
use super::showing::{Choice, PlanetView, SceneAssets, Showing, build_globe};
use crate::common::{CurrentWorld, ServerClock};
use crate::widgets::{layer_z, leaves_presses};

/// Above the menus and their layers.
const VIEW_LAYER: i32 = 50;
/// The picture's least and greatest size in pixels.
const SMALLEST_PICTURE: u32 = 256;
const LARGEST_PICTURE: u32 = 2048;
/// The note at the bottom.
const NOTE: &str =
    "Right-drag or two fingers to turn, wheel or pinch to zoom; click, tap or Esc to close";
const NOTE_COLOUR: Color = Color::srgb(0.55, 0.55, 0.6);

#[allow(clippy::too_many_arguments)]
pub(super) fn open(
    mut asked: MessageReader<ViewPlanet>,
    mut view: ResMut<PlanetView>,
    world: Option<Res<CurrentWorld>>,
    clock: Res<ServerClock>,
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
    let shortest = window.physical_width().min(window.physical_height());
    let pixels = shortest.clamp(SMALLEST_PICTURE, LARGEST_PICTURE);
    let picture = assets.images.add(Image::new_target_texture(
        pixels,
        pixels,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    let choice = Choice::of(&world);
    let globe = build_globe(&mut commands, &mut assets, &world, choice, picture.clone());
    let (layer, image, bar) = spawn_layer(&mut commands, picture.clone(), choice);
    view.showing = Some(Showing {
        layer,
        picture,
        bar,
        choice,
        globe,
        orbit: Orbit::default(),
        image,
        pointed: None,
        clicked: false,
        start_hours: clock.hours(),
        opened: time.elapsed_secs_f64(),
        typing: false,
    });
}

/// The black layer, with the picture in the middle as large as fits, the bar
/// at the top and a note at the bottom. Presses on it are left for the view
/// (to close it), and go no further.
fn spawn_layer(
    commands: &mut Commands,
    picture: Handle<Image>,
    choice: Choice,
) -> (Entity, Entity, Bar) {
    let mut layer = commands.spawn((
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
    ));
    let entity = layer.id();
    layer.insert(leaves_presses(entity));
    let (mut bar, mut image) = (None, None);
    layer.with_children(|layer| {
        let node = layer.spawn((
            ImageNode::new(picture),
            Node {
                width: Val::VMin(100.0),
                height: Val::VMin(100.0),
                ..default()
            },
        ));
        image = Some(node.id());
        bar = Some(spawn_bar(layer, choice));
        layer.spawn((
            Text::new(NOTE),
            TextColor(NOTE_COLOUR),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(16),
                ..default()
            },
        ));
    });
    let spawned = "the layer's children are spawned straight away";
    (entity, image.expect(spawned), bar.expect(spawned))
}
