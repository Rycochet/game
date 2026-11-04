//! What one item is physically like, rolled once when the server makes it and saved
//! on the item from then on. Changing the rolls never changes items already made.
//! See `docs/context/shared/item.md`.
//!
//! Each kind rolls its choices in order, each seeing those before it. A choice can
//! lean on an earlier one by changing weights, or limit it with a weight of 0.

use serde::{Deserialize, Serialize};

use crate::armour::arms::ArmsForm;
use crate::armour::hands::HandsForm;
use crate::armour::head::HeadForm;
use crate::armour::legs::LegsForm;
use crate::armour::shoulders::ShouldersForm;
use crate::armour::torso::TorsoForm;
use crate::armour::waist::WaistForm;
use crate::blade::BladeForm;
use crate::item::Item;

mod armour;
mod gem;
mod give;
mod limits;
mod measure;
mod ring;
mod ring_roll;
mod words;

pub use gem::{Cut, GemForm};
pub use give::{give_blade_within, give_commission, give_form};
pub use limits::Limits;
pub use ring::{BandWidth, Mount, Pattern, Profile, RingForm, Setting};

/// What an item is like, for kinds that have this.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemForm {
    Gem(GemForm),
    Ring(Box<RingForm>),
    Blade(Box<BladeForm>),
    /// Gauntlets and other hand pieces.
    Hands(HandsForm),
    /// A pair of pauldrons or other shoulder pieces.
    Shoulders(ShouldersForm),
    /// A cuirass or other torso armour, or an arming vest.
    Torso(TorsoForm),
    /// A helmet, coif or arming cap.
    Head(HeadForm),
    /// Vambraces, sleeves or other arm pieces, for one arm or both.
    Arms(ArmsForm),
    /// Greaves, chausses or other leg pieces, for both legs or one.
    Legs(LegsForm),
    /// A fauld, tassets, a skirt or another waist piece.
    Waist(WaistForm),
}

impl ItemForm {
    /// What something like this is called, without its size.
    pub fn name(&self) -> String {
        match self {
            Self::Gem(gem) => format!("{} {}", gem.cut.name(), gem.material.name()),
            Self::Ring(ring) => ring.name(),
            Self::Blade(blade) => blade.name().to_owned(),
            Self::Hands(hands) => hands.name().to_owned(),
            Self::Shoulders(shoulders) => shoulders.name().to_owned(),
            Self::Torso(torso) => torso.name().to_owned(),
            Self::Head(head) => head.name(),
            Self::Arms(arms) => arms.name(),
            Self::Legs(legs) => legs.name(),
            Self::Waist(waist) => waist.name(),
        }
    }

    /// The gems set in it.
    pub fn gems(&self) -> impl Iterator<Item = &Item> {
        let settings = match self {
            Self::Ring(ring) => ring.settings.as_slice(),
            Self::Gem(_)
            | Self::Blade(_)
            | Self::Hands(_)
            | Self::Shoulders(_)
            | Self::Torso(_)
            | Self::Head(_)
            | Self::Arms(_)
            | Self::Legs(_)
            | Self::Waist(_) => &[],
        };
        settings.iter().filter_map(|setting| setting.gem.as_ref())
    }
}

#[cfg(test)]
mod tests;
