//! Items, and carrying them in grids: the rules both sides check moves by. See
//! `docs/context/shared/item.md`.
//!
//! What a kind of item is (`ItemKind::def`) is a table here. An item itself is
//! its id and kind, and what is inside it. Everything a character carries is a
//! tree starting from their pockets: a grid of placed items, some of which have
//! grids of their own.

mod category;
mod def;
mod drop;
mod inventory;
// Named after the type it holds, as every file here is.
mod feet_def;
#[allow(clippy::module_inception)]
mod item;
mod item_def;
mod kind;
mod made;
mod neck_def;
mod placed;
mod quality;
mod refused;
mod secrets;
mod set_def;
mod shape;
mod stack;

pub use category::ItemCategory;
pub use drop::{Drop, ODDS_OUT_OF, roll_drop};
pub use inventory::Inventory;
pub use item::{Item, MadeAt};
pub use item_def::{ContainerDef, ItemDef, WearDef};
pub use kind::ItemKind;
pub use made::Made;
pub use placed::{Holder, Placed};
pub use quality::{Quality, artifact_name};
pub use refused::MoveRefused;
pub use secrets::{Favour, Known};
pub use shape::{Rotation, Shape};
pub use stack::StackLimits;

/// The widest and tallest a shape can be.
pub const MAX_SHAPE_SIDE: u8 = 8;

/// How many grids deep things can go below the pockets: backpack, pouch, box.
pub const MAX_DEPTH: u32 = 3;

/// The pockets of a character saved before creature kinds, and of a new
/// `Inventory`. Otherwise they come from the kind of creature.
pub const POCKETS: (u8, u8) = (6, 4);

#[cfg(test)]
mod add_tests;

#[cfg(test)]
mod stack_tests;

#[cfg(test)]
mod tests;
