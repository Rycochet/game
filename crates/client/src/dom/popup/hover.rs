//! A `hover` popup: about something the pointer rests on (an item), it stays
//! while the pointer is on that or on the popup, and closes a moment after it
//! has left both, with no click. Its owner says each frame that the pointer is
//! on what it is about (`KeptOpen`). Locked by its padlock, it stays as any
//! locked popup does. With no pointer at all (a finger lifted), it stays
//! until a press elsewhere, so a finger held on an item opens one to read.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::Popup;
use crate::common::UiPointer;
use crate::dom::controls::{UiEvent, UiValue};
use crate::widgets::{UiId, logical_rect};

/// How long a hover popup waits, after the pointer has left it and what it is
/// about, before closing: time to cross the gap between them.
pub(super) const GRACE: f32 = 0.3;

/// The `hover` popups whose subject the pointer is on: their owners add their
/// ids each frame (in `Update` or `PostUpdate`), and the popups read and
/// empty it (`PreUpdate`).
#[derive(Resource, Default, Debug)]
pub struct KeptOpen(pub Vec<UiId>);

/// On a hover popup's box: when the pointer was last on it or its subject, by
/// the real clock.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct LastKept(pub(super) f32);

type HoverBox<'a> = (
    Entity,
    &'a Popup,
    &'a UiId,
    &'a Node,
    &'a ComputedNode,
    &'a UiGlobalTransform,
    Option<&'a LastKept>,
);

/// Closes each unlocked hover popup once the pointer has been off it and its
/// subject for `GRACE`.
pub(super) fn close_hovers(
    popups: Query<HoverBox>,
    mut kept: ResMut<KeptOpen>,
    pointer: Res<UiPointer>,
    time: Res<Time<Real>>,
    mut events: MessageWriter<UiEvent>,
    mut commands: Commands,
) {
    let kept = std::mem::take(&mut kept.0);
    let now = time.elapsed_secs();
    for (entity, popup, id, node, computed, transform, last) in &popups {
        if !popup.hover || popup.locked || node.display == Display::None {
            continue;
        }
        let (corner, size) = logical_rect(computed, transform);
        let on_it = pointer
            .0
            .is_some_and(|at| Rect::from_corners(corner, corner + size).contains(at));
        // No pointer (a finger lifted): it waits for a press elsewhere.
        let stays = pointer.0.is_none() || on_it || kept.contains(id);
        match last {
            Some(LastKept(since)) if !stays && now - since > GRACE => {
                events.write(UiEvent {
                    id: *id,
                    value: UiValue::Closed,
                });
            }
            Some(_) if !stays => {}
            // (Newly shown counts as kept.)
            _ => {
                commands.entity(entity).try_insert(LastKept(now));
            }
        }
    }
}
