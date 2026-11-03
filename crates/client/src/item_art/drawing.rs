//! A drawing being made: its parts, the light, and waiting for shaders.

use bevy::prelude::*;
use bevy::render::MainWorld;
use bevy::render::render_resource::PipelineCache;

/// One piece of a model.
pub(super) struct Part {
    pub(super) mesh: Handle<Mesh>,
    pub(super) material: Handle<StandardMaterial>,
    pub(super) transform: Transform,
}

/// Everything for one picture while it is being drawn, under one entity.
#[derive(Component)]
pub(super) struct Drawing {
    pub(super) slot: usize,
    pub(super) frames_left: u8,
}

/// The lights all pictures share, there only while any are being drawn so as not to
/// count against the views' lights.
#[derive(Component)]
pub(super) struct PictureLight;

/// How many of the graphics card's programs are still being compiled. Anything
/// needing one isn't drawn until it is ready, which would leave a picture empty.
#[derive(Resource, Default)]
pub(super) struct StillCompiling(pub(super) usize);

/// Tells the main world, from the world that renders.
pub(super) fn count_compiling(mut main_world: ResMut<MainWorld>, cache: Res<PipelineCache>) {
    main_world.insert_resource(StillCompiling(cache.waiting_pipelines().count()));
}
