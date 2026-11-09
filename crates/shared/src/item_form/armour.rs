//! Rolling the forms of armour, footwear and underclothes, by item kind.

use super::ItemForm;
use crate::armour::hands::{self, HandsPiece};
use crate::armour::head::{self, HeadKind};
use crate::armour::legs::{self, LegsKind};
use crate::armour::neck::{self, NeckKind};
use crate::armour::set::{self, SetKind};
use crate::armour::torso::{self, TorsoKind};
use crate::armour::waist::{self, WaistKind};
use crate::armour::{Style, arms, feet, shoulders};
use crate::footwear::{self, FootwearPiece};
use crate::item::{ItemKind, Quality};
use crate::rng::Rng;
use crate::underclothes::{self, UnderPiece};

/// What is rolled for a kind of armour or footwear.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Piece {
    Footwear(FootwearPiece),
    Under(UnderPiece),
    Arms,
    Hands(HandsPiece),
    Shoulders,
    Torso(TorsoKind),
    Head(HeadKind),
    Legs(LegsKind),
    Waist(WaistKind),
    Feet,
    Neck(NeckKind),
    Set(SetKind),
}

impl Piece {
    /// The piece an item kind is, or `None` for other kinds.
    pub(super) fn of(kind: ItemKind) -> Option<Self> {
        if let Some(piece) = FootwearPiece::of(kind) {
            return Some(Self::Footwear(piece));
        }
        if let Some(piece) = UnderPiece::of(kind) {
            return Some(Self::Under(piece));
        }
        Some(match kind {
            ItemKind::ArmArmour => Self::Arms,
            ItemKind::Gauntlets => Self::Hands(HandsPiece::Gauntlets),
            ItemKind::DemiGauntlets => Self::Hands(HandsPiece::Demi),
            ItemKind::PaddedGloves => Self::Hands(HandsPiece::Padded),
            ItemKind::ShoulderArmour => Self::Shoulders,
            ItemKind::TorsoArmour => Self::Torso(TorsoKind::Armour),
            ItemKind::ArmingVest => Self::Torso(TorsoKind::ArmingVest),
            ItemKind::ArmingCap => Self::Head(HeadKind::Cap),
            ItemKind::Coif => Self::Head(HeadKind::Coif),
            ItemKind::Helmet => Self::Head(HeadKind::Helmet),
            ItemKind::BevoredHelm => Self::Head(HeadKind::BevoredHelm),
            ItemKind::LegArmour => Self::Legs(LegsKind::Armour),
            ItemKind::Chausses => Self::Legs(LegsKind::Chausses),
            ItemKind::PaddedChausses => Self::Legs(LegsKind::Padded),
            ItemKind::WaistArmour => Self::Waist(WaistKind::Armour),
            ItemKind::CuirassierTassets => Self::Waist(WaistKind::Cuirassier),
            ItemKind::FootArmour => Self::Feet,
            ItemKind::Collar => Self::Neck(NeckKind::Collar),
            ItemKind::Gorget => Self::Neck(NeckKind::Gorget),
            ItemKind::Bevor => Self::Neck(NeckKind::Bevor),
            ItemKind::Aventail => Self::Neck(NeckKind::Aventail),
            ItemKind::Mantle => Self::Neck(NeckKind::Mantle),
            ItemKind::ArmourCoat => Self::Set(SetKind::Coat),
            ItemKind::Corslet => Self::Set(SetKind::Corslet),
            ItemKind::HalfArmour => Self::Set(SetKind::Half),
            ItemKind::FullArmour => Self::Set(SetKind::Full),
            _ => return None,
        })
    }

    /// A new one, in `style` if given (footwear has none), or one of its own.
    pub(super) fn roll(self, rng: &mut Rng, quality: Quality, style: Option<Style>) -> ItemForm {
        match self {
            Self::Footwear(piece) => ItemForm::Footwear(footwear::roll(rng, piece, quality)),
            Self::Under(piece) => ItemForm::Under(underclothes::roll(rng, piece, quality)),
            Self::Arms => ItemForm::Arms(arms::roll(rng, style, quality)),
            Self::Hands(piece) => ItemForm::Hands(hands::roll(rng, piece, style)),
            Self::Shoulders => ItemForm::Shoulders(shoulders::roll(rng, None, style)),
            Self::Torso(kind) => ItemForm::Torso(torso::roll(rng, kind, style, quality)),
            Self::Head(kind) => ItemForm::Head(head::roll(rng, kind, style, quality)),
            Self::Legs(kind) => ItemForm::Legs(legs::roll(rng, kind, style, quality)),
            Self::Waist(kind) => ItemForm::Waist(waist::roll(rng, kind, style, quality)),
            Self::Feet => ItemForm::Feet(feet::roll(rng, style, quality)),
            Self::Neck(kind) => ItemForm::Neck(neck::roll(rng, kind, style, quality)),
            Self::Set(kind) => ItemForm::Set(Box::new(set::roll(rng, kind, style, quality))),
        }
    }
}

/// Whether a kind is armour with a form (not footwear or underclothes, which have
/// their own options).
pub(super) fn is_armour(kind: ItemKind) -> bool {
    Piece::of(kind).is_some_and(|piece| !matches!(piece, Piece::Footwear(_) | Piece::Under(_)))
}
