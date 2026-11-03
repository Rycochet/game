//! Pictures of items, drawn once in 3D by a camera of their own and kept as images.
//! See `docs/context/client/item_art.md`.

mod armour;
mod blade;
mod body;
mod gem;
mod looks;
mod ring;

use std::collections::{HashMap, VecDeque};
use std::f32::consts::TAU;

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::render_resource::{PipelineCache, TextureFormat};
use bevy::render::{ExtractSchedule, MainWorld, RenderApp};
use bevy::window::PrimaryWindow;
use shared::blade::BladeForm;
use shared::creature::{CreatureKind, Size};
use shared::item::{Item, Quality};
use shared::item_form::{BandWidth, GemForm, ItemForm, Mount, Pattern, Profile};
use shared::material::Material;

pub use looks::{HandsLook, HeadLook, ShouldersLook, TorsoLook};

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
/// The most times over the pixels asked for a picture is drawn with, for a large
/// interface (see `ItemPictures::detail`).
const MOST_DETAIL: u32 = 2;
/// The screen's pixels to an interface pixel that pictures as asked for are sharp up
/// to: they are asked for at twice the size they are shown.
const SHARP_UP_TO: f32 = 2.0;
/// How many black copies make the edge: enough that it looks round.
const OUTLINE_COPIES: usize = 16;
/// How far behind a model its black copies go, to be behind every part of it.
const OUTLINE_BEHIND: f32 = 3.0;

/// Everything a picture shows. Ids stay out of it, so things that look the same share
/// one picture, and things that change their looks get a new one.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Look {
    Gem {
        gem: GemForm,
        quality: Option<Quality>,
    },
    Ring {
        ring: RingLook,
        quality: Option<Quality>,
    },
    Blade {
        blade: BladeLook,
        quality: Option<Quality>,
    },
    /// A pair of gauntlets or other hand pieces.
    Hands {
        hands: HandsLook,
        quality: Option<Quality>,
    },
    /// A pair of pauldrons or other shoulder pieces, as worn.
    Shoulders {
        shoulders: ShouldersLook,
        quality: Option<Quality>,
    },
    /// A cuirass or other torso piece, as worn.
    Torso {
        torso: TorsoLook,
        quality: Option<Quality>,
    },
    /// A helmet, coif or arming cap, as worn.
    Head {
        head: HeadLook,
        quality: Option<Quality>,
    },
    /// A bare body of a kind that stands on two legs, standing: what worn things are
    /// built on, and later a character with what it wears.
    Body { kind: CreatureKind },
}

impl Look {
    /// The cells it is drawn to fill, wide and long: only the proportions matter.
    pub fn cells(&self) -> (u8, u8) {
        match self {
            Self::Gem { .. } | Self::Ring { .. } => (1, 1),
            Self::Blade { blade, .. } => blade.cells,
            Self::Hands { hands, .. } => hands.cells,
            Self::Shoulders { shoulders, .. } => shoulders.cells,
            Self::Torso { torso, .. } => torso.cells,
            Self::Head { head, .. } => head.cells,
            Self::Body { .. } => body::CELLS,
        }
    }

    /// The most cells it reaches along either side.
    pub fn longest(&self) -> u8 {
        let (wide, long) = self.cells();
        wide.max(long)
    }

    /// How big its picture is, given how many pixels its longest side has.
    pub fn picture_size(&self, pixels: u32) -> UVec2 {
        let (wide, long) = self.cells();
        let longest = self.longest() as u32;
        UVec2::new(wide as u32, long as u32) * pixels / longest
    }
}

/// What can be seen of a weapon: lengths to the nearest centimetre, so that nearly the
/// same weapons share a picture, and without what doesn't show.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BladeLook {
    form: BladeForm,
    /// What it takes up, wide and long, not turned.
    cells: (u8, u8),
}

/// A length to the nearest centimetre.
fn to_centimetre(millimetres: u16) -> u16 {
    (millimetres + 5) / 10 * 10
}

/// What can be seen of a ring. Sizes are against the ring's own, as how big it is
/// doesn't show: only a gem too big for it does.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RingLook {
    metal: Material,
    profile: Profile,
    width: BandWidth,
    pattern: Pattern,
    settings: Vec<SettingLook>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SettingLook {
    angle: u8,
    mount: Mount,
    /// Sizes bigger than the ring.
    bigger: i8,
    gem: Option<SetGemLook>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SetGemLook {
    gem: GemForm,
    quality: Option<Quality>,
    /// Sizes bigger than its setting.
    bigger: i8,
}

/// How many sizes `size` is above `than`, or below if negative.
fn sizes_above(size: Size, than: Size) -> i8 {
    size as i8 - than as i8
}

/// How this item looks, if it has a form to go by.
pub fn look_of(item: &Item) -> Option<Look> {
    let quality = item.quality;
    match item.form.as_ref()? {
        ItemForm::Blade(form) => {
            let shape = item.shape();
            Some(Look::Blade {
                blade: BladeLook {
                    form: BladeForm {
                        blade: to_centimetre(form.blade),
                        handle: to_centimetre(form.handle),
                        generic_only: false,
                        ..**form
                    },
                    cells: (shape.width, shape.height),
                },
                quality,
            })
        }
        ItemForm::Gem(gem) => Some(Look::Gem { gem: *gem, quality }),
        ItemForm::Hands(_) | ItemForm::Shoulders(_) | ItemForm::Torso(_) | ItemForm::Head(_) => {
            looks::armour_look(item)
        }
        ItemForm::Ring(ring) => {
            let settings = ring.settings.iter().map(|setting| SettingLook {
                angle: setting.angle,
                mount: setting.mount,
                bigger: sizes_above(setting.size, item.made.size),
                gem: setting.gem.as_ref().and_then(|gem| match &gem.form {
                    Some(ItemForm::Gem(form)) => Some(SetGemLook {
                        gem: *form,
                        quality: gem.quality,
                        bigger: sizes_above(gem.made.size, setting.size),
                    }),
                    _ => None,
                }),
            });
            Some(Look::Ring {
                ring: RingLook {
                    metal: ring.metal,
                    profile: ring.profile,
                    width: ring.width,
                    pattern: ring.pattern,
                    settings: settings.collect(),
                },
                quality,
            })
        }
    }
}

/// The pictures made or being made. Ask for one with `picture`: it is empty for the
/// frames until it has been drawn.
#[derive(Resource)]
pub struct ItemPictures {
    /// By the pixels they were drawn with.
    made: HashMap<(Look, u32), Handle<Image>>,
    /// With the pixels asked for, for the black edge.
    waiting: VecDeque<(Look, u32, Handle<Image>)>,
    /// How many times over the pixels asked for they are drawn with, so they stay
    /// sharp on a large interface.
    detail: u32,
}

impl Default for ItemPictures {
    fn default() -> Self {
        Self {
            made: HashMap::default(),
            waiting: VecDeque::default(),
            detail: 1,
        }
    }
}

impl ItemPictures {
    /// The picture of this, this many pixels along its longest side (more, on a
    /// large interface).
    pub fn picture(&mut self, look: Look, asked: u32, images: &mut Assets<Image>) -> Handle<Image> {
        let pixels = asked * self.detail;
        if let Some(picture) = self.made.get(&(look.clone(), pixels)) {
            return picture.clone();
        }
        let size = look.picture_size(pixels);
        let picture = images.add(Image::new_target_texture(
            size.x,
            size.y,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        self.made.insert((look.clone(), pixels), picture.clone());
        self.waiting.push_back((look, asked, picture.clone()));
        picture
    }
}

/// On an `ImageNode`: it shows this picture once there is one to show.
#[derive(Component, Clone, Debug)]
pub struct ItemPicture {
    pub look: Look,
    /// How many pixels along its longest side it is asked for (`ItemPictures::picture`).
    pub pixels: u32,
}

/// One piece of a model.
struct Part {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
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
            .add_systems(
                PostUpdate,
                (follow_interface_scale, fill_pictures, draw_pictures).chain(),
            );
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(ExtractSchedule, count_compiling);
        }
    }
}

/// How many times over the pixels asked for pictures are drawn with, for this many of
/// the screen's pixels to an interface pixel.
fn detail_for(screen_per_interface: f32) -> u32 {
    ((screen_per_interface / SHARP_UP_TO).ceil() as u32).clamp(1, MOST_DETAIL)
}

/// Draws pictures with more pixels when the interface is drawn larger, and
/// gives those on show the new ones.
fn follow_interface_scale(
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
    mut shown: Query<(&ItemPicture, &mut ImageNode)>,
) {
    let detail = detail_for(window.scale_factor() * ui_scale.0);
    if pictures.detail == detail {
        return;
    }
    pictures.detail = detail;
    for (picture, mut node) in &mut shown {
        node.image = pictures.picture(picture.look.clone(), picture.pixels, &mut images);
    }
}

fn fill_pictures(
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
    mut new: Query<(&ItemPicture, &mut ImageNode), Added<ItemPicture>>,
) {
    for (picture, mut node) in &mut new {
        node.image = pictures.picture(picture.look.clone(), picture.pixels, &mut images);
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
        let Some((look, asked, picture)) = pictures.waiting.pop_front() else {
            break;
        };
        let (wide, long) = look.cells();
        let longest = look.longest() as f32;
        let view = Vec2::new(wide as f32, long as f32) / longest * VIEW_WIDTH;
        let parts = match look {
            Look::Gem { gem, quality } => {
                gem::gem_parts(&gem, quality, &mut meshes, &mut materials)
                    .into_iter()
                    .map(|(mesh, material)| Part {
                        mesh,
                        material,
                        transform: Transform::default(),
                    })
                    .collect()
            }
            Look::Ring { ring, quality } => {
                ring::ring_parts(&ring, quality, &mut meshes, &mut materials)
            }
            Look::Blade { blade, quality } => {
                blade::blade_parts(&blade, quality, &mut meshes, &mut materials)
            }
            Look::Body { kind } => body::body_parts(kind, &mut meshes, &mut materials),
            look => armour::armour_parts(look, &mut meshes, &mut materials),
        };
        let black = materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            cull_mode: None,
            ..default()
        });
        // As thick on the screen whatever the detail.
        let outline = outline_copies(&parts, asked, &black);
        spawn_drawing(&mut commands, slot, picture, view, parts, outline);
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
fn outline_copies(parts: &[Part], pixels: u32, black: &Handle<StandardMaterial>) -> Vec<Part> {
    let looking = -CAMERA_AT.normalize();
    let right = looking.cross(CAMERA_UP).normalize();
    let up = right.cross(looking);
    let reach = OUTLINE_PIXELS * VIEW_WIDTH / pixels as f32;
    let mut copies = Vec::new();
    let mut last = None;
    for part in parts {
        // A gem is one mesh drawn twice, which needs only one edge.
        let this = (part.mesh.id(), part.transform);
        if last == Some(this) {
            continue;
        }
        last = Some(this);
        for copy in 0..OUTLINE_COPIES {
            let angle = copy as f32 / OUTLINE_COPIES as f32 * TAU;
            let across = (right * angle.cos() + up * angle.sin()) * reach;
            let mut transform = part.transform;
            transform.translation += across + looking * OUTLINE_BEHIND;
            copies.push(Part {
                mesh: part.mesh.clone(),
                material: black.clone(),
                transform,
            });
        }
    }
    copies
}

fn spawn_drawing(
    commands: &mut Commands,
    slot: usize,
    picture: Handle<Image>,
    view: Vec2,
    parts: Vec<Part>,
    outline: Vec<Part>,
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
                        width: view.x,
                        height: view.y,
                    },
                    ..OrthographicProjection::default_3d()
                }),
                Tonemapping::None,
                Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP),
                layer.clone(),
            ));
            for part in parts.into_iter().chain(outline) {
                drawing.spawn((
                    Mesh3d(part.mesh),
                    MeshMaterial3d(part.material),
                    part.transform,
                    layer.clone(),
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pictures_get_more_detail_on_a_large_interface() {
        assert_eq!(detail_for(1.0), 1);
        assert_eq!(detail_for(SHARP_UP_TO), 1);
        assert_eq!(detail_for(2.4), 2);
        assert_eq!(detail_for(9.0), MOST_DETAIL);
    }
}
