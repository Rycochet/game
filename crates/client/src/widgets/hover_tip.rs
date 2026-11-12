//! Words about something bigger than a button, such as a whole row of
//! Settings: shown after the pointer (or a finger held down) stays anywhere on
//! it for a moment, even over a part that is disabled.

use bevy::prelude::*;

use super::item::{ItemTip, ItemTooltip};
use crate::dom::{Hit, Solid};

/// How long the pointer stays before the words show, in seconds.
const DELAY: f32 = 0.5;
const OFFSET: Vec2 = Vec2::new(16.0, 16.0);

/// What this node and everything in it is, shown on hovering (nothing while
/// it is empty).
#[derive(Component, Clone, Debug)]
#[require(Solid)]
pub struct HoverTip(pub String);

/// Asks for the words of the `HoverTip` holding what is in front at the
/// pointer (`dom::Hit`), once it has stayed there, unless something else has
/// been named.
pub(in crate::widgets) fn show_hover_tips(
    time: Res<Time<Real>>,
    hit: Res<Hit>,
    tips: Query<&HoverTip>,
    mut tooltip: ResMut<ItemTooltip>,
    mut since: Local<Option<(Entity, f32)>>,
) {
    let Some(at) = hit.at else {
        *since = None;
        return;
    };
    let holder = hit.first(|entity| tips.contains(entity).then_some(entity));
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
