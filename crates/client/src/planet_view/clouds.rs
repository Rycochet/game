//! Clouds on the globe, for looks only (not `shared::weather`, whose detail
//! would blur at this size): a see-through shell just above the ground,
//! turning with it, drawn by `shaders/planet_clouds.wesl`. They change as
//! they go: the shader moves through its noise with the hours, and bands of
//! latitude drift east or west at their own speeds.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use shared::rng::Seed;
use std::f32::consts::FRAC_PI_2;

use super::open::PlanetView;
use super::{HOURS_PER_SECOND, PLANET_LAYER};

/// How far above the ground the clouds are, in the globe's radii: at this
/// size they all but sit on it.
const CLOUD_HEIGHT: f32 = 1.006;

/// What the shader is told. Order matches `struct Clouds` in it.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
pub(super) struct CloudParams {
    /// `xyz`: where in the noise this world's clouds start, from its seed;
    /// `w`: game hours since the view opened.
    pub(super) start_hours: Vec4,
    /// `xyz`: towards the main sun; `w` unused.
    pub(super) sun: Vec4,
}

/// The clouds' see-through material.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct PlanetCloudMaterial {
    #[uniform(0)]
    pub(super) params: CloudParams,
}

impl Material for PlanetCloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/planet_clouds.wesl".into()
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

/// The shell, as a child of the globe so it turns with it, tipped as the
/// globe's own sphere is, so its picture coordinates are longitude and
/// latitude.
pub(super) fn spawn_clouds(
    commands: &mut Commands,
    globe: Entity,
    terrain: Seed,
    assets: (&mut Assets<Mesh>, &mut Assets<PlanetCloudMaterial>),
) -> Handle<PlanetCloudMaterial> {
    let (meshes, materials) = assets;
    let mut rng = terrain.derive("planet view clouds").rng();
    let mut start = || rng.range_f64(0.0, 100.0) as f32;
    let material = materials.add(PlanetCloudMaterial {
        params: CloudParams {
            start_hours: Vec4::new(start(), start(), start(), 0.0),
            sun: Vec4::Y,
        },
    });
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(CLOUD_HEIGHT).mesh().uv(96, 48))),
        MeshMaterial3d(material.clone()),
        Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        RenderLayers::layer(PLANET_LAYER),
        ChildOf(globe),
    ));
    material
}

/// Tells the shader the hours since opening and where the main sun is, after
/// `spin` has placed its light.
pub(super) fn move_clouds(
    view: Res<PlanetView>,
    time: Res<Time<Real>>,
    lights: Query<&Transform>,
    mut materials: ResMut<Assets<PlanetCloudMaterial>>,
) {
    let Some(showing) = view.showing.as_ref() else {
        return;
    };
    let Some(mut material) = materials.get_mut(&showing.clouds) else {
        return;
    };
    let hours = (time.elapsed_secs_f64() - showing.opened) * HOURS_PER_SECOND;
    material.params.start_hours.w = hours as f32;
    // The lights sit unturned at the scene's root, so this is in the world's
    // frame, as the shader's normals are.
    if let Some(light) = showing
        .scene
        .suns
        .first()
        .and_then(|&sun| lights.get(sun).ok())
    {
        material.params.sun = (-light.forward().as_vec3()).extend(0.0);
    }
}
