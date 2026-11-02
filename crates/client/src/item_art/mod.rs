//! Pictures of items, drawn once in 3D by a camera of their own and kept as images.
//! See `docs/context/client/item_art.md`.

mod gem;

use std::collections::{HashMap, VecDeque};
use std::f32::consts::TAU;

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::render_resource::{PipelineCache, TextureFormat};
use bevy::render::{ExtractSchedule, MainWorld, RenderApp};
use shared::item::{Item, Quality};
use shared::item_form::{GemForm, ItemForm};

/// Render layers kept for this, past any the views use, one per picture being drawn.
const FIRST_LAYER: usize = 16;
const SLOTS: usize = 8;
/// Frames a picture's camera stays once nothing is compiling, to be sure of being
/// drawn once: something new to draw only starts compiling after a frame.
const FRAMES_ALIVE: u8 = 4;
/// How much of the world a picture shows, across: a model reaches about 1 each way.
const VIEW_WIDTH: f32 = 2.5;
/// Looking down at a slant, the way a thing lies on a table.
const CAMERA_AT: Vec3 = Vec3::new(0.0, 3.0, 1.0);
const CAMERA_UP: Vec3 = Vec3::NEG_Z;
/// How thick the black edge round every picture is, in its own pixels: pictures are
/// drawn at twice the size they are shown.
const OUTLINE_PIXELS: f32 = 4.0;
/// How many black copies make the edge: enough that it looks round.
const OUTLINE_COPIES: usize = 16;
/// How far behind a model its black copies go, to be behind every part of it.
const OUTLINE_BEHIND: f32 = 3.0;

/// Everything a picture shows. Ids stay out of it, so things that look the same share
/// one picture, and things that change their looks get a new one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Look {
    Gem {
        gem: GemForm,
        quality: Option<Quality>,
    },
}

/// How this item looks, if it has a form to go by.
#[allow(dead_code)] // Until pictures are shown in the inventory.
pub fn look_of(item: &Item) -> Option<Look> {
    match item.form.as_ref()? {
        ItemForm::Gem(gem) => Some(Look::Gem {
            gem: *gem,
            quality: item.quality,
        }),
        // No model yet.
        ItemForm::Ring(_) => None,
    }
}

/// The pictures made or being made. Ask for one with `picture`: it is empty for the
/// frames until it has been drawn.
#[derive(Resource, Default)]
pub struct ItemPictures {
    made: HashMap<(Look, u32), Handle<Image>>,
    waiting: VecDeque<(Look, u32, Handle<Image>)>,
}

impl ItemPictures {
    /// The picture of this, square, this many pixels across.
    pub fn picture(
        &mut self,
        look: Look,
        pixels: u32,
        images: &mut Assets<Image>,
    ) -> Handle<Image> {
        if let Some(picture) = self.made.get(&(look, pixels)) {
            return picture.clone();
        }
        let picture = images.add(Image::new_target_texture(
            pixels,
            pixels,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        self.made.insert((look, pixels), picture.clone());
        self.waiting.push_back((look, pixels, picture.clone()));
        picture
    }
}

/// Everything for one picture while it is being drawn, under one entity.
#[derive(Component)]
struct Drawing {
    slot: usize,
    frames_left: u8,
}

/// The lights all pictures share, there only while any are being drawn so as not to
/// count against the views' lights.
#[derive(Component)]
struct PictureLight;

/// How many of the graphics card's programs are still being compiled. Anything
/// needing one isn't drawn until it is ready, which would leave a picture empty.
#[derive(Resource, Default)]
struct StillCompiling(usize);

/// Tells the main world, from the world that renders.
fn count_compiling(mut main_world: ResMut<MainWorld>, cache: Res<PipelineCache>) {
    main_world.insert_resource(StillCompiling(cache.waiting_pipelines().count()));
}

pub struct ItemArtPlugin;

impl Plugin for ItemArtPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemPictures>()
            .init_resource::<StillCompiling>()
            .add_systems(PostUpdate, draw_pictures);
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(ExtractSchedule, count_compiling);
        }
    }
}

fn draw_pictures(
    mut commands: Commands,
    mut pictures: ResMut<ItemPictures>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drawing: Query<(Entity, &mut Drawing)>,
    lights: Query<Entity, With<PictureLight>>,
    compiling: Res<StillCompiling>,
) {
    let mut free = [true; SLOTS];
    for (entity, mut one) in &mut drawing {
        if one.frames_left == 0 {
            commands.entity(entity).despawn();
        } else {
            if compiling.0 == 0 {
                one.frames_left -= 1;
            }
            free[one.slot] = false;
        }
    }
    let any_left = free.iter().any(|&slot| !slot);
    if !any_left && pictures.waiting.is_empty() {
        for light in &lights {
            commands.entity(light).despawn();
        }
        return;
    }
    if lights.is_empty() {
        spawn_lights(&mut commands);
    }
    for slot in (0..SLOTS).filter(|&slot| free[slot]) {
        let Some((look, pixels, picture)) = pictures.waiting.pop_front() else {
            break;
        };
        let parts = match look {
            Look::Gem { gem, quality } => {
                gem::gem_parts(&gem, quality, &mut meshes, &mut materials)
            }
        };
        let black = materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            cull_mode: None,
            ..default()
        });
        let outline = outline_copies(&parts, pixels, &black);
        spawn_drawing(&mut commands, slot, picture, parts, outline);
    }
}

/// Strongest from the top right, as things are lit in the interface, with enough from
/// elsewhere that no face goes black, and shiny things have something to catch.
fn spawn_lights(commands: &mut Commands) {
    let layers = RenderLayers::from_layers(&(FIRST_LAYER..FIRST_LAYER + SLOTS).collect::<Vec<_>>());
    let lights = [
        (Vec3::new(2.0, 3.0, 1.0), 4000.0),
        (Vec3::new(-3.0, 1.5, 1.5), 1500.0),
        (Vec3::new(0.5, 0.6, -3.0), 1000.0),
    ];
    for (from, illuminance) in lights {
        commands.spawn((
            PictureLight,
            DirectionalLight {
                illuminance,
                ..default()
            },
            Transform::from_translation(from).looking_at(Vec3::ZERO, Vec3::Y),
            layers.clone(),
        ));
    }
}

/// A black edge of the same thickness whatever is drawn: each mesh again in black,
/// moved a little each way across the picture, and back so that only what sticks out
/// shows. Going by what the camera sees, it needs nothing of a model's shape.
fn outline_copies(
    parts: &[(Handle<Mesh>, Handle<StandardMaterial>)],
    pixels: u32,
    black: &Handle<StandardMaterial>,
) -> Vec<(Handle<Mesh>, Handle<StandardMaterial>, Vec3)> {
    let looking = -CAMERA_AT.normalize();
    let right = looking.cross(CAMERA_UP).normalize();
    let up = right.cross(looking);
    let reach = OUTLINE_PIXELS * VIEW_WIDTH / pixels as f32;
    let mut meshes: Vec<&Handle<Mesh>> = parts.iter().map(|(mesh, _)| mesh).collect();
    meshes.dedup();
    let mut copies = Vec::new();
    for mesh in meshes {
        for copy in 0..OUTLINE_COPIES {
            let angle = copy as f32 / OUTLINE_COPIES as f32 * TAU;
            let across = (right * angle.cos() + up * angle.sin()) * reach;
            let moved = across + looking * OUTLINE_BEHIND;
            copies.push((mesh.clone(), black.clone(), moved));
        }
    }
    copies
}

fn spawn_drawing(
    commands: &mut Commands,
    slot: usize,
    picture: Handle<Image>,
    parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>)>,
    outline: Vec<(Handle<Mesh>, Handle<StandardMaterial>, Vec3)>,
) {
    let layer = RenderLayers::layer(FIRST_LAYER + slot);
    commands
        .spawn((
            Drawing {
                slot,
                frames_left: FRAMES_ALIVE,
            },
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|drawing| {
            drawing.spawn((
                Camera3d::default(),
                Camera {
                    // Before anything that might show it.
                    order: -10 - slot as isize,
                    clear_color: Color::NONE.into(),
                    ..default()
                },
                AmbientLight {
                    brightness: 150.0,
                    ..default()
                },
                RenderTarget::Image(picture.into()),
                Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::Fixed {
                        width: VIEW_WIDTH,
                        height: VIEW_WIDTH,
                    },
                    ..OrthographicProjection::default_3d()
                }),
                Tonemapping::None,
                Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP),
                layer.clone(),
            ));
            for (mesh, material) in parts {
                drawing.spawn((Mesh3d(mesh), MeshMaterial3d(material), layer.clone()));
            }
            for (mesh, material, moved) in outline {
                drawing.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(moved),
                    layer.clone(),
                ));
            }
        });
}
