//! The World Gate whose ring the pointer is on: named beside the pointer
//! ("Gate 1", and why it can't be gone to, if it can't), and kept for
//! `take_input`, which goes there on a click.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::gates::RING;
use super::pick::{Sight, gate_spot, gate_under};
use super::showing::PlanetView;
use super::teleport::why_not;
use crate::common::{DebugTools, UiPointer};
use crate::widgets::{ItemTip, ItemTooltip, shown_at};
use shared::role::DebugTool;

/// Where the name goes from the pointer, as other tooltips have it.
const OFFSET: Vec2 = Vec2::new(16.0, 16.0);

/// A gate's ring under the pointer: its number (from 0), and whether a click
/// goes there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PointedGate {
    pub(super) number: u8,
    pub(super) can_go: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn point_at_gates(
    mut view: ResMut<PlanetView>,
    tools: Res<DebugTools>,
    pointer: Res<UiPointer>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    places: Query<&Transform>,
    mut tooltip: ResMut<ItemTooltip>,
) {
    let Some(showing) = view.showing.as_mut() else {
        return;
    };
    showing.pointed = None;
    let Some(at) = pointer.0 else {
        return;
    };
    // Not through the bar. (The whole view is a layer of the interface, so
    // `OverInterface` is always on here.)
    if let Ok((node, transform)) = nodes.get(showing.bar.root)
        && shown_at(at, node, transform, None)
    {
        return;
    }
    let (Ok((node, transform)), Ok(camera), Ok(globe)) = (
        nodes.get(showing.image),
        places.get(showing.globe.scene.camera),
        places.get(showing.globe.scene.globe),
    ) else {
        return;
    };
    let scale = node.inverse_scale_factor;
    let size = node.size() * scale;
    let corner = transform.translation * scale - size / 2.0;
    let on_picture = (at - corner) / size.max(Vec2::ONE);
    let planet = &showing.globe.planet;
    let width = planet.surface.width();
    let spots: Vec<Vec2> = (planet.gates.gates.iter())
        .map(|gate| gate_spot((gate.x, gate.y), width))
        .collect();
    let sight = Sight {
        camera,
        globe: globe.rotation,
        across: size.x,
        ring: RING / showing.globe.size.x as f32,
    };
    let Some(number) = gate_under(on_picture, &spots, &sight) else {
        return;
    };
    let why = why_not(showing, tools.may(DebugTool::TeleportToWorld));
    let words = match why {
        None => format!("Gate {}", number + 1),
        Some(why) => format!("Gate {} (can't go: {why})", number + 1),
    };
    showing.pointed = Some(PointedGate {
        number: number as u8,
        can_go: why.is_none(),
    });
    tooltip.0 = Some(ItemTip {
        words,
        quality: None,
        set: false,
        at: at + OFFSET,
    });
}
