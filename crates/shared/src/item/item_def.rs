//! `ItemDef`, `ContainerDef` and `WearDef`: what a kind of item is, and the helpers the table in `def.rs` is written with.

use crate::creature::{Layer, Location};
use crate::item::Shape;

/// The grid inside a container, and what it will take. Anything put in must also
/// be `Shape::smaller_than` the container's outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerDef {
    pub width: u8,
    pub height: u8,
    /// The most all its contents may weigh, in grams.
    pub max_total: Option<u32>,
    /// The most any one thing in it may weigh, in grams, with its contents.
    pub max_each: Option<u32>,
}

/// What a kind of item is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemDef {
    pub name: &'static str,
    /// In grams, when empty.
    pub weight: u32,
    pub shape: Shape,
    pub container: Option<ContainerDef>,
    pub wear: Option<WearDef>,
    /// Whether how well it is made matters: not for a coin.
    pub has_quality: bool,
    /// How long it takes to make beside other things, as a percentage.
    pub make_percent: u64,
}

/// How something is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WearDef {
    pub layer: Layer,
    pub locations: &'static [Location],
    /// It takes only one place of its one location: a ring on a finger.
    pub one_place: bool,
    /// Of its locations, where it also takes the Middle layer: armour with its own
    /// padding, worn all as one.
    pub padded: &'static [Location],
    /// How many sizes smaller and larger than it was made for it can be worn.
    pub down: u8,
    pub up: u8,
    /// It adjusts, so it fits properly anywhere in that range. Otherwise anything
    /// but its own size is loose.
    pub adjustable: bool,
}

pub(super) const fn plain(name: &'static str, weight: u32, shape: Shape) -> ItemDef {
    ItemDef {
        name,
        weight,
        shape,
        container: None,
        wear: None,
        has_quality: true,
        make_percent: 100,
    }
}

pub(super) const fn bag(
    name: &'static str,
    weight: u32,
    shape: Shape,
    (width, height): (u8, u8),
    max_total: Option<u32>,
    max_each: Option<u32>,
) -> ItemDef {
    ItemDef {
        container: Some(ContainerDef {
            width,
            height,
            max_total,
            max_each,
        }),
        ..plain(name, weight, shape)
    }
}

pub(super) const fn wear(
    layer: Layer,
    locations: &'static [Location],
    (down, up): (u8, u8),
    adjustable: bool,
) -> WearDef {
    WearDef {
        layer,
        locations,
        one_place: false,
        padded: &[],
        down,
        up,
        adjustable,
    }
}

impl ItemDef {
    pub(super) const fn taking(self, make_percent: u64) -> Self {
        Self {
            make_percent,
            ..self
        }
    }

    pub(super) const fn without_quality(self) -> Self {
        Self {
            has_quality: false,
            ..self
        }
    }

    pub(super) const fn worn(self, wear: WearDef) -> Self {
        Self {
            wear: Some(wear),
            ..self
        }
    }
}
