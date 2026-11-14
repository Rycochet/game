//! Opening an item's popup once the pointer rests on it, keeping it while the
//! pointer is on the item, and replacing it with another item's.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::Item;

use super::show::{ITEM_POPUP, fill};
use crate::common::Wording;
use crate::dom::{DocumentOf, Dom, HeldObject, KeptOpen, Popup, UiEvent, UiValue, document};
use crate::widgets::{PointedItem, UiId};

/// How long the pointer rests on an item before its popup opens, by the real
/// clock: long enough not to flash one open for every item passed over.
pub(super) const REST: f32 = 0.15;
/// How far from the pointer the popup's corner goes, so as not to cover it.
pub(super) const OFFSET: Vec2 = Vec2::new(16.0, 16.0);

/// The form every item popup's ids are in, as `item_popup#<id>/…`.
pub(super) const FORM: &str = "item_popup";

/// The popups open, and the item the pointer has rested on and since when.
#[derive(Resource, Default)]
pub(super) struct ItemPopups {
    pub(super) open: Vec<OpenPopup>,
    resting: Option<(Id, f32)>,
}

pub(super) struct OpenPopup {
    /// The item's id, which tells its ids apart.
    pub(super) of: Id,
    pub(super) document: Entity,
    pub(super) item: Item,
}

/// A popup's box, by the id of the item it is about.
pub(super) fn box_id(of: Id) -> UiId {
    UiId::of(FORM, of, "box")
}

/// Despawns each popup that says it closed.
pub(super) fn forget_closed(
    mut events: MessageReader<UiEvent>,
    mut popups: ResMut<ItemPopups>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.value != UiValue::Closed || event.id.parts().0 != FORM {
            continue;
        }
        let Some(of) = event.id.which() else {
            continue;
        };
        popups.open.retain(|open| {
            let closed = open.of == of;
            if closed {
                commands.entity(open.document).try_despawn();
            }
            !closed
        });
    }
}

/// Follows what the pointer is on (`PointedItem`): keeps the popup about it
/// open, or opens one once it has rested there, in place of any not locked.
#[allow(clippy::too_many_arguments)]
pub(super) fn follow_pointed(
    mut pointed: ResMut<PointedItem>,
    held: Res<HeldObject>,
    time: Res<Time<Real>>,
    mut popups: ResMut<ItemPopups>,
    locks: Query<(&Popup, &UiId)>,
    mut kept: ResMut<KeptOpen>,
    (mut dom, wording): (Dom, Res<Wording>),
    mut commands: Commands,
) {
    let pointed = pointed.0.take().filter(|_| held.thing().is_none());
    let Some(pointed) = pointed else {
        popups.resting = None;
        return;
    };
    let of = pointed.item.id;
    if popups.open.iter().any(|open| open.of == of) {
        kept.0.push(box_id(of));
        popups.resting = None;
        return;
    }
    let now = time.elapsed_secs();
    let since = match popups.resting {
        Some((was, since)) if was == of => since,
        _ => {
            popups.resting = Some((of, now));
            now
        }
    };
    if now - since < REST {
        return;
    }
    popups.resting = None;
    // Only one about what the pointer rests on: locked ones stay.
    let locked = |of: Id| {
        let id = box_id(of);
        locks.iter().any(|(popup, at)| *at == id && popup.locked)
    };
    popups.open.retain(|open| {
        let stays = locked(open.of);
        if !stays {
            commands.entity(open.document).try_despawn();
        }
        stays
    });
    let document = commands.spawn((document(&ITEM_POPUP), DocumentOf(of))).id();
    fill(
        &mut dom,
        of,
        &pointed,
        (pointed.at + OFFSET).round(),
        &wording.0,
    );
    kept.0.push(box_id(of));
    popups.open.push(OpenPopup {
        of,
        document,
        item: pointed.item,
    });
}
