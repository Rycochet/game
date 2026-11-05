//! What Give Item is set to, and the parts of its panel that say so.

use bevy::prelude::*;
use shared::creature::Size;
use shared::item::{Favour, ItemKind, Quality};
use shared::item_form::{Field, ItemOrder, RollRequest};

use super::OptionValue;

/// Whether new items are part of a set. Set and Unique do the same for now.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::debug_menu) enum SetChoice {
    #[default]
    None,
    Set,
    Unique,
}

impl SetChoice {
    pub(super) const ALL: [Self; 3] = [Self::None, Self::Set, Self::Unique];

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Set => "Set",
            Self::Unique => "Unique",
        }
    }
}

/// What Give Item makes. Put back to this whenever the panel opens.
#[derive(Resource)]
pub(in crate::debug_menu) struct GiveForm {
    pub(super) count: u32,
    /// `None` for the target's own (the character's, for now).
    pub(super) size: Option<Size>,
    /// All alike in one stack, rather than each rolled from its own seed.
    pub(super) identical: bool,
    pub(super) quality: Quality,
    pub(super) set: SetChoice,
    pub(super) favour: Option<Favour>,
    pub(super) magical: bool,
    pub(super) kind: ItemKind,
    /// What its form must be like: only options its kind takes.
    pub(super) request: RollRequest,
    /// `None` while the seed box says nothing that can be read.
    pub(super) seed: Option<u64>,
    /// How many seeds the Random button has made, to keep them apart.
    pub(super) seeds_made: i64,
}

impl Default for GiveForm {
    fn default() -> Self {
        Self {
            count: 1,
            size: None,
            identical: true,
            quality: Quality::default(),
            set: SetChoice::None,
            favour: None,
            magical: false,
            kind: give_kinds()[0],
            request: RollRequest::NONE,
            seed: None,
            seeds_made: 0,
        }
    }
}

impl GiveForm {
    pub(super) fn order(&self) -> ItemOrder {
        ItemOrder {
            kind: self.kind,
            size: self.size,
            magical: self.magical,
            quality: self.quality,
            set: self.set != SetChoice::None,
            favour: self.favour,
            request: self.request,
        }
    }
}

/// Every kind of item, by name, as the type dropdown lists them.
pub(super) fn give_kinds() -> Vec<ItemKind> {
    let mut kinds = ItemKind::ALL.to_vec();
    kinds.sort_by_key(|kind| kind.def().name);
    kinds
}

/// Which of Give Item's dropdowns this is.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum GiveChoice {
    Size,
    Identical,
    Quality,
    Set,
    Favour,
    Magic,
    Kind,
}

/// The box with how many to give, and the buttons beside it: -1 or +1.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveCount;
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct GiveCountStep(pub(super) i32);

#[derive(Component)]
pub(in crate::debug_menu) struct GiveSeed;
#[derive(Component)]
pub(in crate::debug_menu) struct GiveRandomSeed;

/// Where the item to be given is drawn.
#[derive(Component, Default)]
pub(in crate::debug_menu) struct GivePreview {
    /// The order and seed it was drawn for.
    pub(super) drawn: Option<(ItemOrder, u64)>,
}

/// The line of options for its form, shown for kinds that take any.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveOptionsLine;

/// A dropdown for one option of its form: "Any ..." then `values`.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveOption {
    pub(super) field: Field,
    pub(super) values: Vec<OptionValue>,
}

/// Why it can't all be given ("No space", or what can't be made), when it can't.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveNoSpace;

#[derive(Component)]
pub(in crate::debug_menu) struct GiveButton;
