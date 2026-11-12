//! How good a skill is, as a ring round the inside of a round node's edge
//! (`assets/shaders/skill_ring.wesl`): filled clockwise from the top, from a sliver
//! when just learned to a full circle at 100, all in the colour of its fifth's item
//! quality (Common below 20 up to Legendary from 80). A specialised skill gets a
//! second, thinner ring inside the first for 100 to 120, as a set item's metal edge
//! in the Artifact colour with its sliding bands (owner, 2026-11-07).

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use shared::item::Quality;

use crate::common::{BandShift, quality_colour};

/// The value a full ring stands for, and how much more a specialisation's does.
pub const FULL: f32 = 100.0;
const SPECIALISED_MORE: f32 = 20.0;
/// The least of a ring shown once anything is learned.
const SLIVER: f32 = 0.02;
/// Where the ring starts, as a share of the node's radius: a band at its edge.
const INNER: f32 = 0.8;
/// Each fifth's colour, from the first.
const LEVELS: [Quality; 5] = [
    Quality::Common,
    Quality::Uncommon,
    Quality::Rare,
    Quality::Epic,
    Quality::Legendary,
];

/// What a ring shows.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct SkillRing {
    /// How much of the first ring is filled, 0 to 1.
    pub fill: f32,
    pub quality: Quality,
    /// Whether it has the specialisation's ring at all.
    pub specialised: bool,
    /// How much of the specialisation's ring is filled, 0 to 1.
    pub special: f32,
}

impl SkillRing {
    /// The ring for a skill's own value (not counting those above it), and whether
    /// it has been specialised.
    pub fn of(value: f32, specialised: bool) -> Self {
        let share = (value / FULL).clamp(0.0, 1.0);
        let fill = if value > 0.0 { share.max(SLIVER) } else { 0.0 };
        let fifth = ((share * LEVELS.len() as f32) as usize).min(LEVELS.len() - 1);
        let special = if specialised {
            ((value - FULL) / SPECIALISED_MORE).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            fill,
            quality: LEVELS[fifth],
            specialised,
            special,
        }
    }
}

/// Order matches `struct Ring` in the shader.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct RingParams {
    colour: Vec4,
    special_colour: Vec4,
    fill: f32,
    special: f32,
    shift: f32,
    inner: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct SkillRingMaterial {
    #[uniform(0)]
    params: RingParams,
}

impl UiMaterial for SkillRingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/skill_ring.wesl".into()
    }
}

/// A node covering its (round) parent, drawing this ring. Not what the pointer is on.
pub fn skill_ring(ring: SkillRing) -> impl Bundle {
    (
        ring,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
    )
}

fn params(ring: &SkillRing, shift: f32) -> RingParams {
    let colour = |quality| LinearRgba::from(quality_colour(quality)).to_vec4();
    RingParams {
        colour: colour(ring.quality),
        special_colour: colour(Quality::Artifact),
        fill: ring.fill,
        // Below 0 for no second ring, which the shader then leaves out.
        special: if ring.specialised { ring.special } else { -1.0 },
        shift,
        inner: INNER,
    }
}

/// Gives each new ring its own material, and changes a changed ring's in place.
/// `try_insert`: a ring may be despawned (its page drawn again) before the material
/// lands, which is no reason to crash.
#[allow(clippy::type_complexity)]
pub(super) fn give_rings(
    rings: Query<
        (Entity, &SkillRing, Option<&MaterialNode<SkillRingMaterial>>),
        Changed<SkillRing>,
    >,
    shift: Res<BandShift>,
    mut materials: ResMut<Assets<SkillRingMaterial>>,
    mut commands: Commands,
) {
    for (entity, ring, given) in &rings {
        let params = params(ring, shift.0);
        if let Some(mut material) = given.and_then(|node| materials.get_mut(&node.0)) {
            material.params = params;
            continue;
        }
        let material = materials.add(SkillRingMaterial { params });
        commands.entity(entity).try_insert(MaterialNode(material));
    }
}

/// Slides the specialisation rings' bands with the set edges'.
pub(super) fn slide_rings(
    shift: Res<BandShift>,
    rings: Query<(&SkillRing, &MaterialNode<SkillRingMaterial>)>,
    mut materials: ResMut<Assets<SkillRingMaterial>>,
) {
    if !shift.is_changed() {
        return;
    }
    for (ring, node) in &rings {
        if ring.specialised
            && let Some(mut material) = materials.get_mut(&node.0)
        {
            material.params.shift = shift.0;
        }
    }
}

#[cfg(test)]
mod tests;
