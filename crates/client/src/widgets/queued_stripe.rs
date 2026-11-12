//! Something waiting its turn, such as an ability queued on the action bar:
//! yellow and black hazard stripes round the inside of a node, chasing round
//! it (`assets/shaders/queued_stripe.wesl`). One material for all, so every
//! stripe moves together; it stands still with Reduce Motion.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use crate::common::Preferences;

/// How fast the stripes chase round, in pixels a second: quickly, to catch
/// the eye (owner's wish).
const SPEED: f32 = 60.0;
/// One yellow and one black stripe, as `STRIPE_PIXELS` in the shader: moving
/// this far looks the same, so the shift is kept below it.
const STRIPE_PIXELS: f32 = 12.0;

/// Order matches `struct Stripe` in the shader; `x` is the shift. A whole
/// `vec4`, as WebGL 2 wants uniforms in multiples of 16 bytes.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct StripeParams {
    params: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct QueuedStripeMaterial {
    #[uniform(0)]
    params: StripeParams,
}

impl UiMaterial for QueuedStripeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/queued_stripe.wesl".into()
    }
}

/// On a node drawn as moving hazard stripes in its border.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct QueuedStripe;

/// The one material.
#[derive(Resource)]
pub(super) struct QueuedStripes(Handle<QueuedStripeMaterial>);

/// Hazard stripes `width` wide round the inside of the parent, standing `inset`
/// in from its edges (inside another edge, say). Not what the pointer is on.
pub fn queued_stripe(width: f32, inset: f32) -> impl Bundle {
    (
        QueuedStripe,
        Node {
            position_type: PositionType::Absolute,
            left: px(inset),
            top: px(inset),
            right: px(inset),
            bottom: px(inset),
            border: UiRect::all(px(width)),
            ..default()
        },
    )
}

pub(super) fn make_queued_stripes(
    mut materials: ResMut<Assets<QueuedStripeMaterial>>,
    mut commands: Commands,
) {
    let material = QueuedStripeMaterial {
        params: StripeParams::default(),
    };
    commands.insert_resource(QueuedStripes(materials.add(material)));
}

pub(super) fn give_queued_stripes(
    stripes: Res<QueuedStripes>,
    new: Query<Entity, Added<QueuedStripe>>,
    mut commands: Commands,
) {
    for entity in &new {
        // It may be gone before this lands: skipped, not a crash.
        commands
            .entity(entity)
            .try_insert(MaterialNode(stripes.0.clone()));
    }
}

/// Moves the stripes on, by real time so pausing doesn't stop them.
pub(super) fn chase_stripes(
    time: Res<Time<Real>>,
    preferences: Res<Preferences>,
    stripes: Res<QueuedStripes>,
    mut materials: ResMut<Assets<QueuedStripeMaterial>>,
) {
    if preferences.graphics.reduce_motion {
        return;
    }
    if let Some(mut material) = materials.get_mut(&stripes.0) {
        let shift = &mut material.params.params.x;
        *shift = chased(*shift, time.delta_secs());
    }
}

/// The shift after `seconds`, kept within one pair of stripes.
fn chased(shift: f32, seconds: f32) -> f32 {
    (shift + SPEED * seconds).rem_euclid(STRIPE_PIXELS)
}

#[cfg(test)]
mod tests;
