//! The tree as written: one file per top level, in the order they are shown.

mod athletics;
mod combat;
mod crafting;
mod gathering;
mod lore;
mod magic;
mod social;
mod stealth;
mod survival;

use super::draft::Draft;

/// The top levels, in order.
pub(super) const TOP: [&Draft; 9] = [
    &combat::COMBAT,
    &magic::MAGIC,
    &athletics::ATHLETICS,
    &stealth::STEALTH,
    &survival::SURVIVAL,
    &gathering::GATHERING,
    &crafting::CRAFTING,
    &social::SOCIAL,
    &lore::LORE,
];
