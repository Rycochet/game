//! Rolling the forms of armour, by item kind.

use super::ItemForm;
use crate::armour::arms;
use crate::armour::feet;
use crate::armour::hands::{self, HandsPiece};
use crate::armour::head::{self, HeadKind};
use crate::armour::legs::{self, LegsKind};
use crate::armour::shoulders;
use crate::armour::torso::{self, TorsoKind};
use crate::armour::waist::{self, WaistKind};
use crate::item::{ItemKind, Quality};
use crate::rng::Rng;

/// The form of a new piece of armour of this kind, or `None` for other kinds.
pub(super) fn armour_form(kind: ItemKind, rng: &mut Rng, quality: Quality) -> Option<ItemForm> {
    let hands = |rng: &mut Rng, piece| ItemForm::Hands(hands::roll(rng, piece, None));
    let torso = |rng: &mut Rng, kind| ItemForm::Torso(torso::roll(rng, kind, None, quality));
    let head = |rng: &mut Rng, kind| ItemForm::Head(head::roll(rng, kind, None, quality));
    let legs = |rng: &mut Rng, kind| ItemForm::Legs(legs::roll(rng, kind, None, quality));
    let waist = |rng: &mut Rng, kind| ItemForm::Waist(waist::roll(rng, kind, None, quality));
    Some(match kind {
        ItemKind::ArmArmour => ItemForm::Arms(arms::roll(rng, None, quality)),
        ItemKind::Gauntlets => hands(rng, HandsPiece::Gauntlets),
        ItemKind::DemiGauntlets => hands(rng, HandsPiece::Demi),
        ItemKind::PaddedGloves => hands(rng, HandsPiece::Padded),
        ItemKind::ShoulderArmour => ItemForm::Shoulders(shoulders::roll(rng, None, None)),
        ItemKind::TorsoArmour => torso(rng, TorsoKind::Armour),
        ItemKind::ArmingVest => torso(rng, TorsoKind::ArmingVest),
        ItemKind::ArmingCap => head(rng, HeadKind::Cap),
        ItemKind::Coif => head(rng, HeadKind::Coif),
        ItemKind::Helmet => head(rng, HeadKind::Helmet),
        ItemKind::BevoredHelm => head(rng, HeadKind::BevoredHelm),
        ItemKind::LegArmour => legs(rng, LegsKind::Armour),
        ItemKind::Chausses => legs(rng, LegsKind::Chausses),
        ItemKind::PaddedChausses => legs(rng, LegsKind::Padded),
        ItemKind::WaistArmour => waist(rng, WaistKind::Armour),
        ItemKind::CuirassierTassets => waist(rng, WaistKind::Cuirassier),
        ItemKind::FootArmour => ItemForm::Feet(feet::roll(rng, None, quality)),
        _ => return None,
    })
}
