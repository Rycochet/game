//! The metal edge of a set item (`assets/shaders/set_edge.wesl`): one material for
//! each quality, all with the same bands, which slide with `common::BandShift`: slowly
//! by themselves, and faster as the view changes (`view3d`).

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use shared::item::Quality;

use super::Floating;
use crate::common::{BandShift, quality_colour};

/// How fast the bands drift by themselves, in pixels a second: slowly, so they
/// catch the light even while nothing moves (owner's wish, speed Claude's).
const DRIFT_SPEED: f32 = 12.0;
/// The bands' spacing, as `BAND_PIXELS` in the shader: sliding this far looks the
/// same, so the shift is kept below it.
const BAND_PIXELS: f32 = 58.0;

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

/// A metal edge with these sides (`border`), in a parent it stands `out` beyond the
/// inside of on each side it has, so a shape's cells join up. It is not what the
/// pointer is on.
pub fn set_edge(quality: Option<Quality>, border: UiRect, out: f32) -> impl Bundle {
    (SetEdge(quality), Floating, edge_node(border, out))
}

/// The node `set_edge` draws: standing out only where it has a side.
fn edge_node(border: UiRect, out: f32) -> Node {
    let beyond = |side: Val| if side == px(0) { px(0) } else { px(-out) };
    Node {
        position_type: PositionType::Absolute,
        left: beyond(border.left),
        top: beyond(border.top),
        right: beyond(border.right),
        bottom: beyond(border.bottom),
        border,
        ..default()
    }
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

/// Slides the bands a little every frame, by real time so pausing doesn't stop it.
pub(super) fn drift_bands(time: Res<Time<Real>>, mut shift: ResMut<BandShift>) {
    shift.0 = drifted(shift.0, time.delta_secs());
}

/// The shift after drifting for `seconds`, kept within one band.
fn drifted(shift: f32, seconds: f32) -> f32 {
    (shift + DRIFT_SPEED * seconds).rem_euclid(BAND_PIXELS)
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
mod tests;
