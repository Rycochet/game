//! Naming the item in writing under the pointer, as for items anywhere.

use bevy::prelude::*;
use bevy::text::TextLayoutInfo;
use bevy::ui::UiGlobalTransform;

use super::inline::InlineItem;
use crate::common::{KnownItem, KnownItems};
use crate::dom::Hit;
use crate::widgets::item::TOOLTIP_OFFSET;
use crate::widgets::{ItemTip, ItemTooltip, Pointed, PointedItem};

/// What is said of an item in writing not known yet.
fn tip(known: Option<&KnownItem>, at: Vec2) -> ItemTip {
    let words = match known {
        Some(KnownItem::Unknown(_)) => "Unknown item",
        _ => "Finding out what this is",
    };
    ItemTip {
        words: words.to_owned(),
        quality: None,
        set: false,
        at: at + TOOLTIP_OFFSET,
    }
}

/// Points at the item under the pointer in the writing in front there
/// (`dom::Hit`'s target), if any, for its popup; one not known yet is named
/// by a tooltip saying so.
pub(in crate::widgets) fn name_inline_items(
    hit: Res<Hit>,
    layouts: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform)>,
    items: Query<&InlineItem>,
    known: Res<KnownItems>,
    mut tooltip: ResMut<ItemTooltip>,
    mut pointed: ResMut<PointedItem>,
) {
    let free = tooltip.0.is_none() && pointed.0.is_none();
    let Some(at) = hit.at.filter(|_| free) else {
        return;
    };
    let Some((layout, node, transform)) = hit.target.and_then(|front| layouts.get(front).ok())
    else {
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
    let Some(item) = under else {
        return;
    };
    match known.get(item.id) {
        Some(KnownItem::Found(item, _)) => {
            let (item, notes) = ((**item).clone(), String::new());
            pointed.0 = Some(Pointed { item, notes, at });
        }
        other => tooltip.0 = Some(tip(other, at)),
    }
}
