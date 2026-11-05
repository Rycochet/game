//! Keeping a scroll area's place. Every area keeps it as a share of how far it
//! can go (`ScrollShare`), so it stays at much the same place in its contents
//! when the window, the interface size or its contents change size. One with a
//! `UiId` is also remembered for the rest of the session, so an area built
//! again (a page rebuilt after a change, a panel opened again) comes back there.

use std::collections::HashMap;

use bevy::prelude::*;

use super::{ScrollArea, furthest};
use crate::widgets::UiId;

/// Closer than this, in interface pixels, counts as the same place.
const SAME_PLACE: f32 = 0.5;

/// Where an area is: `share` from 0 at the top to 1 at the bottom, and how far
/// it could go when that was last seen (`None` until it has been laid out).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::widgets) struct ScrollShare {
    pub(super) share: f32,
    pub(super) furthest: Option<f32>,
}

/// The places of areas with keys, for the session.
#[derive(Resource, Default)]
pub(in crate::widgets) struct ScrollMemory(HashMap<UiId, ScrollShare>);

/// Where a share is, in an area that can go this far.
pub(super) fn place_of(share: f32, furthest: f32) -> f32 {
    share.clamp(0.0, 1.0) * furthest
}

/// What share a place is: an area with nowhere to go keeps the one it had.
pub(super) fn share_of(place: f32, furthest: f32, had: f32) -> f32 {
    if furthest > 0.0 {
        (place / furthest).clamp(0.0, 1.0)
    } else {
        had
    }
}

/// Before layout: a new area with a key starts where it was last, in pixels as
/// far as was known then, so it doesn't show its top for a frame.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn bring_back_places(
    memory: Res<ScrollMemory>,
    mut areas: Query<
        (&UiId, &mut ScrollShare, &mut ScrollPosition),
        (Added<UiId>, With<ScrollArea>),
    >,
) {
    for (key, mut share, mut position) in &mut areas {
        let Some(kept) = memory.0.get(key) else {
            continue;
        };
        share.share = kept.share;
        share.furthest = None;
        position.y = place_of(kept.share, kept.furthest.unwrap_or(0.0));
    }
}

/// After layout: when an area's room to scroll changes, it goes back to its
/// share of the new room; otherwise wherever it was moved to (by hand or by
/// `AutoScroll`) becomes its share. Areas with keys are remembered.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn keep_places(
    mut memory: ResMut<ScrollMemory>,
    mut areas: Query<
        (
            &ComputedNode,
            &mut ScrollShare,
            &mut ScrollPosition,
            Option<&UiId>,
        ),
        With<ScrollArea>,
    >,
) {
    for (node, mut share, mut position, key) in &mut areas {
        // Not laid out, or hidden (a Settings page under another tab): keep the
        // share, and put it back once it shows.
        if node.size == Vec2::ZERO {
            if share.furthest.is_some() {
                share.furthest = None;
            }
            continue;
        }
        let most = furthest(node);
        let wanted = place_of(share.share, most);
        if share.furthest != Some(most) {
            share.furthest = Some(most);
            position.y = wanted;
        } else if (position.y - wanted).abs() > SAME_PLACE {
            share.share = share_of(position.y, most, share.share);
        } else {
            continue;
        }
        if let Some(key) = key {
            memory.0.insert(*key, *share);
        }
    }
}

#[cfg(test)]
mod tests;
