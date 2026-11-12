//! What dev tools' Give Item asks for (`ClientMessage::DevGiveItem`).

use serde::{Deserialize, Serialize};

use crate::creature::Size;
use crate::id::Id;
use crate::item::{Favour, ItemKind, Quality};
use crate::item_form::RollRequest;

/// New items of this kind for the character or `to`, made for this size (or the
/// one getting them's own) and their fingers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevGive {
    pub kind: ItemKind,
    pub size: Option<Size>,
    pub magical: bool,
    /// Ignored for kinds without quality.
    pub quality: Quality,
    /// Part of a set, if it has quality.
    pub set: bool,
    /// Blessed or cursed. It and `magical` are as secret as on anything else.
    pub favour: Option<Favour>,
    /// What it is rolled from (`item_form::make_item`), so it is what Give Item
    /// showed; `None` for a random one.
    pub seed: Option<u64>,
    /// How many: kept to what the server allows (`StackLimits`).
    pub count: u32,
    /// All alike, in one stack; otherwise each rolled from its own seed
    /// (`item_form::batch_seed`). What doesn't fit goes on the ground in a site,
    /// and on the world map nothing is given unless all of it fits.
    pub identical: bool,
    /// What their form must be like (`item_form::RollRequest`); refused, giving
    /// nothing, if any of them can't keep to it (`try_make_item`).
    pub request: RollRequest,
    /// Who gets them: a mob or another player in sight (the target), or
    /// `None` for the character. Nothing is given if `to` isn't in sight.
    pub to: Option<Id>,
}
