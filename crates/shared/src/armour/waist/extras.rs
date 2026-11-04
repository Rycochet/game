//! What a waist piece may have besides: the Roman apron's front-only strips and the
//! mitra's plate, mail beneath plate, a groin plate, dyed cloth or lacing.

use super::WaistPiece;
use crate::armour::{Construction, Style};

/// Whether pteruges may be over the front only: the Roman belt's studded apron.
pub(super) fn may_be_front(piece: WaistPiece, construction: Construction, style: Style) -> bool {
    (piece == WaistPiece::Pteruges && construction == Construction::Leather && style == Style::Classical)
        // The mitra hangs over the belly from a girdle.
        || (piece == WaistPiece::Girdle && construction == Construction::Plate)
}

/// Whether a plate fauld or tassets may have mail beneath, or a plate over the groin:
/// European and fantasy plate.
pub(super) fn may_add_mail(piece: WaistPiece, construction: Construction, style: Style) -> bool {
    matches!(piece, WaistPiece::Fauld | WaistPiece::Tassets)
        && matches!(construction, Construction::Plate | Construction::Laminar)
        && (style.european() || matches!(style, Style::Elven | Style::Dwarven))
}

/// Whether a piece may have a plain plate over the groin (the owner's decision).
pub(super) fn may_guard_groin(piece: WaistPiece, construction: Construction, style: Style) -> bool {
    may_add_mail(piece, construction, style)
        || (piece == WaistPiece::Skirt && construction == Construction::Plate)
}

/// Whether a piece is built on cloth or laced with cord that may be dyed: padding,
/// brigandine, and every laced row of plates.
pub(super) fn has_cloth(piece: WaistPiece, construction: Construction, style: Style) -> bool {
    matches!(
        construction,
        Construction::Padded | Construction::PlatesInCloth
    ) || piece == WaistPiece::Panels
        || (construction == Construction::Lamellar)
        || style == Style::Japanese
}
