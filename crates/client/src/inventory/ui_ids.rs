//! The inventory's widgets' ids (`widgets::UiId`), by whose grid they are for:
//! `inventory/...`, `bag#<id>/...` or `ground/...`.

use shared::item::Holder;

use crate::widgets::UiId;

/// A widget of this grid's panel: `None` for the ground.
pub(super) fn grid_id(holder: Option<Holder>, name: &'static str) -> UiId {
    match holder {
        None => UiId::new("ground", name),
        Some(Holder::Inventory) => UiId::new("inventory", name),
        Some(Holder::Inside(bag)) => UiId::of("bag", bag, name),
    }
}
