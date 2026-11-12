//! Tooltips over items and over what is held, and the inventory button.

use bevy::prelude::*;

use super::super::body::worn_notes;
use super::super::hold::Target;
use super::super::{Carried, Holding, item_cell_in_front};
use super::{HELD_NAME_ABOVE, InventoryButton, ItemCell, OpenGrids, TOOLTIP_OFFSET};
use crate::common::{GROUND, GameView, UiPointer};
use crate::dom::Hit;
use crate::widgets::{CELL, ItemTip, ItemTooltip};

/// The name of what is under the pointer, or of what is held.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::inventory) fn show_tooltip(
    carried: Res<Carried>,
    holding: Res<Holding>,
    pointer: Res<UiPointer>,
    open: Res<OpenGrids>,
    (hit, cells): (Res<Hit>, Query<&ItemCell>),
    mut tooltip: ResMut<ItemTooltip>,
    split: Res<super::super::split::Split>,
) {
    let shown = if let Some(held) = holding.held {
        Some(held.item.id)
    } else if open.open.is_empty() || split.0.is_some() {
        // The split box names the stack itself.
        None
    } else {
        item_cell_in_front(&hit.chain, &cells).map(|cell| cell.id)
    };
    let name = shown.and_then(|id| {
        let item = match holding.held {
            Some(_) => holding.item(&carried.0)?,
            None => carried.0.find(id)?,
        };
        // A stack is named by its top, which is what shows.
        let item = item.top();
        let words = format!("{}{}", item.name(), worn_notes(&carried.0, id));
        Some((words, item.quality, item.set))
    });
    let (Some((name, quality, set)), Some(at)) = (name, pointer.0.or(holding.at)) else {
        return;
    };
    let words = match holding.aim {
        Some(Target::Someone(GROUND)) | Some(Target::Drop { allowed: true }) => {
            format!("Drop {name}")
        }
        Some(Target::Drop { allowed: false }) => format!("Can't drop {name}"),
        Some(Target::Someone(_)) => format!("Give {name}"),
        Some(Target::Bin) => match carried.0.bin.as_ref() {
            Some(old) => format!("Throw away {name}, destroying {}", old.name()),
            None => format!("Throw away {name}"),
        },
        _ => name,
    };
    let top = match holding.held {
        // Above what is held, so as not to hide any of it.
        Some(held) => at.y - (held.item.grip.1 as f32 + 0.5) * CELL - HELD_NAME_ABOVE,
        None => at.y + TOOLTIP_OFFSET.y,
    };
    tooltip.0 = Some(ItemTip {
        words,
        quality,
        set,
        at: Vec2::new(at.x + TOOLTIP_OFFSET.x, top),
    });
}

/// The button is hidden while connecting, as the warning button is.
pub(in crate::inventory) fn show_button(
    view: Res<State<GameView>>,
    button: Option<Single<&mut Visibility, With<InventoryButton>>>,
) {
    // (Not there until its markup is built.)
    let Some(mut button) = button else {
        return;
    };
    button.set_if_neq(if *view.get() == GameView::Connecting {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
}
