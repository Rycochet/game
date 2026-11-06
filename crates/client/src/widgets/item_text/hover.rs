//! Naming the item in writing under the pointer, as for items anywhere.

use bevy::prelude::*;
use bevy::text::TextLayoutInfo;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};

use super::inline::InlineItem;
use crate::common::{KnownItem, KnownItems, UiPointer};
use crate::widgets::item::TOOLTIP_OFFSET;
use crate::widgets::{Floating, ItemTip, ItemTooltip, shown_at};

/// What an item in writing is called, as far as it is known.
fn tip(known: Option<&KnownItem>, at: Vec2) -> ItemTip {
    let (words, quality, set) = match known {
        Some(KnownItem::Found(item, _)) => (item.name(), item.quality, item.set),
        Some(KnownItem::Unknown(_)) => ("Unknown item".to_owned(), None, false),
        Some(KnownItem::Asked(_)) | None => ("Finding out what this is".to_owned(), None, false),
    };
    ItemTip {
        words,
        quality,
        set,
        at: at + TOOLTIP_OFFSET,
    }
}

/// Names the item under the pointer in the writing in front there, if any.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn name_inline_items(
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
    layouts: Query<&TextLayoutInfo>,
    items: Query<&InlineItem>,
    known: Res<KnownItems>,
    mut tooltip: ResMut<ItemTooltip>,
) {
    let Some(at) = pointer.0.filter(|_| tooltip.0.is_none()) else {
        return;
    };
    let front = nodes
        .iter()
        .filter(|(_, _, node, transform, visible, clip)| {
            visible.get() && shown_at(at, node, transform, *clip)
        })
        .max_by_key(|(_, stack, ..)| stack.0);
    let Some((entity, _, node, transform, ..)) = front else {
        return;
    };
    let Ok(layout) = layouts.get(entity) else {
        return;
    };
    // In the layout's own pixels, from the node's top left.
    let corner = transform.translation - node.size() / 2.0;
    let inside = at / node.inverse_scale_factor - corner;
    let under = layout
        .inline_boxes
        .iter()
        .find(|(_, _, rect)| rect.contains(inside))
        .and_then(|(item, ..)| items.get(*item).ok());
    if let Some(item) = under {
        tooltip.0 = Some(tip(known.get(item.id), at));
    }
}
