//! One item among words: room in the writing a line tall and square, with a
//! little gap either side. What fills it is its frame (`frame`).

use bevy::prelude::*;
use bevy::text::{InlineBox, InlineBoxKind};
use shared::id::Id;

use crate::common::KnownItems;

/// The room left either side of an item, so even items side by side stand
/// apart, in interface pixels.
pub(super) const GAP: f32 = 2.0;

/// How tall the items in some writing are, in interface pixels: on the writing.
#[derive(Component, Clone, Copy, Debug)]
#[require(crate::dom::Solid)]
pub(in crate::widgets) struct InlineHeight(pub f32);

/// An item in writing: a child of its text, in among its spans.
#[derive(Component, Clone, Copy, Debug)]
pub(in crate::widgets) struct InlineItem {
    pub id: Id,
}

/// An item to go among a text's spans: a square `side` across, and its gaps.
pub(super) fn inline_item(id: Id, side: f32) -> impl Bundle {
    (
        InlineItem { id },
        // So whether it is on show can be told.
        Visibility::Inherited,
        InlineBox {
            kind: InlineBoxKind::InFlow,
            size: Vec2::new(side + 2.0 * GAP, side),
        },
    )
}

/// Says which items are on show, to be asked after if need be.
pub(in crate::widgets) fn want_inline_items(
    mut known: ResMut<KnownItems>,
    time: Res<Time<Real>>,
    items: Query<(&InlineItem, &InheritedVisibility)>,
) {
    let now = time.elapsed_secs_f64();
    for (inline, visible) in &items {
        if visible.get() {
            // Not a change to what is known: only noted for asking.
            known.bypass_change_detection().want(inline.id, now);
        }
    }
}
