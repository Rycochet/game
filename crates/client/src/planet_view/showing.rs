//! What is on show while the view is open, and building the globe for a
//! choice of seed and generator (again, whenever the bar changes either).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::rng::Seed;
use shared::surface::SurfaceGenerator;

use super::bar::Bar;
use super::clouds::{PlanetCloudMaterial, spawn_clouds};
use super::globe::{Scene, spawn_scene};
use super::layer::Layer;
use super::planet::Planet;
use super::relief::new_relief_texture;
use super::sky::Sky;
use super::texture::{Fill, new_texture, texture_size};
use crate::common::CurrentWorld;

/// Whether the planet is on show, and what it is made of.
#[derive(Resource, Default)]
pub(super) struct PlanetView {
    pub(super) showing: Option<Showing>,
}

pub(super) struct Showing {
    /// The black layer and everything on it.
    pub(super) layer: Entity,
    /// What the scene's camera draws into, shown on the layer.
    pub(super) picture: Handle<Image>,
    pub(super) bar: Bar,
    pub(super) choice: Choice,
    pub(super) globe: Globe,
    /// The world's hour on opening, and the real time then.
    pub(super) start_hours: f64,
    pub(super) opened: f64,
    /// Whether the seed box had the keyboard last frame: the click that takes
    /// it away mustn't also close the view.
    pub(super) typing: bool,
}

/// What the bar has chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Choice {
    pub(super) seed: Seed,
    pub(super) generator: SurfaceGenerator,
    pub(super) layer: Layer,
}

impl Choice {
    /// The current world as it is.
    pub(super) fn of(world: &CurrentWorld) -> Self {
        Self {
            seed: world.params.seed,
            generator: world.params.generator,
            layer: Layer::Surface,
        }
    }
}

/// The globe for one seed and generator.
pub(super) struct Globe {
    /// The seed and generator it was built for, and the layer its picture
    /// shows: when the choice differs, it is built or coloured again.
    pub(super) built: (Seed, SurfaceGenerator),
    pub(super) layer: Layer,
    pub(super) planet: Planet,
    pub(super) scene: Scene,
    /// The clouds' material, which `clouds::move_clouds` keeps up to date.
    pub(super) clouds: Handle<PlanetCloudMaterial>,
    pub(super) sky: Sky,
    /// The surface's picture, its size and how far it is filled.
    pub(super) texture: Handle<Image>,
    /// The relief, and each pixel's height in metres (the sea flat) it is
    /// worked out from, kept as the picture fills.
    pub(super) relief: Handle<Image>,
    pub(super) heights: Vec<f32>,
    pub(super) size: UVec2,
    pub(super) fill: Fill,
}

/// The stores the scene's meshes, materials and pictures go in.
#[derive(SystemParam)]
pub(super) struct SceneAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    clouds: ResMut<'w, Assets<PlanetCloudMaterial>>,
    pub(super) images: ResMut<'w, Assets<Image>>,
}

/// The globe, suns, moons and clouds for a choice, drawn into `picture`.
pub(super) fn build_globe(
    commands: &mut Commands,
    assets: &mut SceneAssets,
    world: &CurrentWorld,
    choice: Choice,
    picture: Handle<Image>,
) -> Globe {
    let planet = Planet::new(world, choice.seed, choice.generator);
    let size = texture_size(planet.surface.width());
    let texture = assets.images.add(new_texture(size));
    let relief = assets.images.add(new_relief_texture(size));
    let celestial = planet.celestial(world);
    let scene = spawn_scene(
        commands,
        (choice.seed, celestial),
        (texture.clone(), relief.clone()),
        picture,
        (
            &mut assets.meshes,
            &mut assets.materials,
            &mut assets.images,
        ),
    );
    let clouds = spawn_clouds(
        commands,
        scene.globe,
        (choice.seed, relief.clone()),
        (&mut assets.meshes, &mut assets.clouds),
    );
    Globe {
        built: (choice.seed, choice.generator),
        layer: choice.layer,
        sky: Sky::new(celestial),
        planet,
        scene,
        clouds,
        texture,
        heights: vec![0.0; (size.x * size.y) as usize],
        relief,
        size,
        fill: Fill::default(),
    }
}
