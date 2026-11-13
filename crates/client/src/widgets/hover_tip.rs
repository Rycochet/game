//! Words about something bigger than a button, such as a whole row of
//! Settings: shown after the pointer (or a finger held down) stays anywhere on
//! it for a moment, even over a part that is disabled. A tip can also be
//! pinned beside something (a "?" tapped), showing until it is let go.

use bevy::prelude::*;

use super::item::{ItemTip, ItemTooltip, logical_rect};
use crate::dom::{Hit, Solid};

/// How long the pointer stays before the words show, in seconds.
const DELAY: f32 = 0.5;
const OFFSET: Vec2 = Vec2::new(16.0, 16.0);

/// What this node and everything in it is, shown on hovering (nothing while
/// it is empty).
#[derive(Component, Clone, Debug)]
#[require(Solid)]
pub struct HoverTip(pub String);

/// A tip shown without hovering: the `HoverTip` of `tip_of`, beside
/// `beside` while that is on show. Whoever pins it lets it go (`dom`'s "?"
/// marks, on the next press or key).
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinnedTip(pub Option<Pinned>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pinned {
    pub tip_of: Entity,
    pub beside: Entity,
}

/// Where a pinned tip shows: past the right end of what it is beside, below
/// its top, if that is on show.
fn pinned_at(
    pinned: &PinnedTip,
    places: &Query<(&ComputedNode, &UiGlobalTransform)>,
) -> Option<(Entity, Vec2)> {
    let Pinned { tip_of, beside } = pinned.0?;
    let (node, transform) = places.get(beside).ok()?;
    let (corner, size) = logical_rect(node, transform);
    (size.x > 0.0).then(|| (tip_of, corner + Vec2::new(size.x, 0.0)))
}

/// Asks for the words of the `HoverTip` holding what is in front at the
/// pointer (`dom::Hit`), once it has stayed there, unless something else has
/// been named.
pub(in crate::widgets) fn show_hover_tips(
    time: Res<Time<Real>>,
    hit: Res<Hit>,
    tips: Query<&HoverTip>,
    pinned: Res<PinnedTip>,
    places: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut tooltip: ResMut<ItemTooltip>,
    mut since: Local<Option<(Entity, f32)>>,
) {
    if tooltip.0.is_none()
        && let Some((holder, at)) = pinned_at(&pinned, &places)
        && let Ok(tip) = tips.get(holder)
        && !tip.0.is_empty()
    {
        tooltip.0 = Some(ItemTip {
            words: tip.0.clone(),
            quality: None,
            set: false,
            at: at + Vec2::new(OFFSET.x / 2.0, 0.0),
        });
        return;
    }
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
