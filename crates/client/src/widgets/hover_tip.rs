//! Words about something bigger than a button, such as a whole row of
//! Settings: shown after the pointer (or a finger held down) stays anywhere on
//! it for a moment, even over a part that is disabled.

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};

use super::item::{Floating, ItemTip, ItemTooltip, shown_at};
use crate::common::UiPointer;

/// How long the pointer stays before the words show, in seconds.
const DELAY: f32 = 0.5;
const OFFSET: Vec2 = Vec2::new(16.0, 16.0);

/// What this node and everything in it is, shown on hovering (nothing while
/// it is empty).
#[derive(Component, Clone, Debug)]
pub struct HoverTip(pub String);

/// Asks for the words of the `HoverTip` holding what is in front at the
/// pointer, once it has stayed there, unless something else has been named.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn show_hover_tips(
    time: Res<Time<Real>>,
    pointer: Res<UiPointer>,
    nodes: Query<
        (
            Entity,
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
        ),
        (With<Node>, Without<Floating>),
    >,
    parents: Query<&ChildOf>,
    tips: Query<&HoverTip>,
    mut tooltip: ResMut<ItemTooltip>,
    mut since: Local<Option<(Entity, f32)>>,
) {
    let Some(at) = pointer.0 else {
        *since = None;
        return;
    };
    let front = nodes
        .iter()
        .filter(|(_, _, node, transform, visible, clip)| {
            visible.get() && shown_at(at, node, transform, *clip)
        })
        .max_by_key(|(_, stack, ..)| stack.0)
        .map(|(entity, ..)| entity);
    let holder = front.and_then(|front| {
        std::iter::once(front)
            .chain(parents.iter_ancestors(front))
            .find(|&entity| tips.contains(entity))
    });
    let Some(holder) = holder else {
        *since = None;
        return;
    };
    let now = time.elapsed_secs();
    let start = match *since {
        Some((was, start)) if was == holder => start,
        _ => {
            *since = Some((holder, now));
            now
        }
    };
    if now - start < DELAY || tooltip.0.is_some() {
        return;
    }
    // An empty one has nothing to say for now.
    if let Ok(tip) = tips.get(holder)
        && !tip.0.is_empty()
    {
        tooltip.0 = Some(ItemTip {
            words: tip.0.clone(),
            quality: None,
            set: false,
            at: at + OFFSET,
        });
    }
}
