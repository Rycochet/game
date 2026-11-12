//! `ItemDef`, `ContainerDef` and `WearDef`: what a kind of item is, and the helpers the table in `def.rs` is written with.

use crate::creature::{Layer, Location};
use crate::item::Shape;

/// The grid inside a container, and what it will take. Anything put in must also
/// be `Shape::smaller_than` the container's outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerDef {
    pub width: u8,
    /// Its rows; for one that grows, the most it can ever have.
    pub height: u8,
    /// So big inside that only the rows in use are shown, with room below, growing
    /// as it fills (a bag of holding): its weight is what really limits it. The
    /// rules are the same as for any grid of `width` by `height`.
    pub grows: bool,
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
    /// Every one is like every other, so a stack of them is a number, not items
    /// with ids (`Item::count`): coins. Such a kind has no quality, form, wear,
    /// grid, name of its own or secrets.
    pub counted: bool,
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
        counted: false,
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
            grows: false,
            max_total,
            max_each,
        }),
        ..plain(name, weight, shape)
    }
}

/// Held in a hand to fight with (`Location::Held`), by anyone a size either way
/// of whom it was made for. A two-handed one takes both hands (`Item::takes_one_place`).
pub(super) const HELD: WearDef = WearDef {
    layer: Layer::Outer,
    locations: &[Location::Held],
    one_place: true,
    padded: &[],
    down: 1,
    up: 1,
    adjustable: true,
};

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

    /// A bag whose grid grows as it fills: see `ContainerDef::grows`.
    pub(super) const fn growing(self) -> Self {
        let Some(container) = self.container else {
            panic!("only a bag can grow");
        };
        Self {
            container: Some(ContainerDef {
                grows: true,
                ..container
            }),
            ..self
        }
    }

    pub(super) const fn without_quality(self) -> Self {
        Self {
            has_quality: false,
            ..self
        }
    }

    /// Held as a number in a stack: see `ItemDef::counted`.
    pub(super) const fn counted(self) -> Self {
        Self {
            counted: true,
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
