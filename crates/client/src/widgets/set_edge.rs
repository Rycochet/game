//! The metal edge of a set item (`assets/shaders/set_edge.wesl`): one material for
//! each quality, all with the same bands, which slide with `common::BandShift`.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use shared::item::Quality;

use super::Floating;
use crate::common::{BandShift, quality_colour};

/// Order matches `struct Edge` in the shader.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct EdgeParams {
    colour: Vec4,
    shift: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct SetEdgeMaterial {
    #[uniform(0)]
    params: EdgeParams,
}

impl UiMaterial for SetEdgeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/set_edge.wesl".into()
    }
}

/// On a node drawn as the metal edge of something of this quality.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetEdge(pub Option<Quality>);

/// The materials, in the order of `Quality::ALL`.
#[derive(Resource)]
pub(super) struct SetEdges(Vec<Handle<SetEdgeMaterial>>);

impl SetEdges {
    fn of(&self, quality: Option<Quality>) -> Handle<SetEdgeMaterial> {
        let quality = quality.unwrap_or_default();
        let index = Quality::ALL.iter().position(|&one| one == quality);
        self.0[index.unwrap_or(0)].clone()
    }
}

/// A metal edge `width` thick, in a parent it stands `out` beyond the inside of.
/// It is not what the pointer is on.
pub fn set_edge(quality: Option<Quality>, width: f32, out: f32) -> impl Bundle {
    (
        SetEdge(quality),
        Floating,
        Node {
            position_type: PositionType::Absolute,
            left: px(-out),
            top: px(-out),
            right: px(-out),
            bottom: px(-out),
            border: UiRect::all(px(width)),
            ..default()
        },
    )
}

pub(super) fn make_set_edges(
    mut materials: ResMut<Assets<SetEdgeMaterial>>,
    mut commands: Commands,
) {
    let handles = Quality::ALL.iter().map(|&quality| {
        let colour = LinearRgba::from(quality_colour(quality)).to_vec4();
        let params = EdgeParams { colour, shift: 0.0 };
        materials.add(SetEdgeMaterial { params })
    });
    commands.insert_resource(SetEdges(handles.collect()));
}

pub(super) fn give_set_edges(
    edges: Res<SetEdges>,
    new: Query<(Entity, &SetEdge), Changed<SetEdge>>,
    mut commands: Commands,
) {
    for (entity, edge) in &new {
        commands
            .entity(entity)
            .insert(MaterialNode(edges.of(edge.0)));
    }
}

pub(super) fn slide_bands(
    shift: Res<BandShift>,
    edges: Res<SetEdges>,
    mut materials: ResMut<Assets<SetEdgeMaterial>>,
) {
    if !shift.is_changed() {
        return;
    }
    for handle in &edges.0 {
        if let Some(mut material) = materials.get_mut(handle) {
            material.params.shift = shift.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bevy only finds two parameters clashing when a system starts.
    #[test]
    fn no_system_asks_for_what_another_of_its_parameters_has() {
        IntoSystem::into_system(give_set_edges).initialize(&mut World::new());
        IntoSystem::into_system(slide_bands).initialize(&mut World::new());
    }
}
