//! What foot pieces may be made of and shaped like, and how often: the lists rolls
//! pick from and the grid checks against.

use super::{FeetPiece, Toe};
use crate::armour::{Construction, Style};

/// Constructions foot armour is made in, by style. Each keeps to `Style::allows`
/// (checked by a test). The regions with no foot armour of their own (Chinese, steppe,
/// Byzantine, Indo-Persian) have mail shoes and leather foot guards.
pub(super) fn constructions(style: Style) -> &'static [(Construction, u64)] {
    use Construction as C;
    match style {
        Style::Elven => &[
            (C::Plate, 50),
            (C::Leather, 20),
            (C::Scale, 15),
            (C::Mail, 15),
        ],
        Style::Dwarven => &[
            (C::Plate, 60),
            (C::Splint, 15),
            (C::Mail, 15),
            (C::Leather, 10),
        ],
        // Kogake and kusari tabi.
        Style::Japanese => &[
            (C::PlatesInCloth, 40),
            (C::MailAndPlate, 30),
            (C::Mail, 20),
            (C::Plate, 10),
        ],
        Style::Chinese | Style::Steppe => &[(C::Leather, 60), (C::Mail, 40)],
        Style::Byzantine => &[(C::Mail, 60), (C::Leather, 40)],
        Style::IndoPersian => &[(C::Mail, 70), (C::MailAndPlate, 30)],
        // The bronze feet.
        Style::Classical => &[(C::Plate, 1)],
        // The European harnesses.
        _ => &[
            (C::Plate, 60),
            (C::Mail, 14),
            (C::Leather, 10),
            (C::Scale, 6),
            (C::Splint, 6),
            (C::RingMail, 4),
        ],
    }
}

/// Which piece a construction makes, in a style.
pub(super) fn pieces(construction: Construction, style: Style) -> &'static [(FeetPiece, u64)] {
    use Construction as C;
    use FeetPiece as P;
    let shaped = style.european() || matches!(style, Style::Elven | Style::Dwarven);
    match construction {
        C::Plate if matches!(style, Style::Classical | Style::Japanese) => &[(P::FootGuards, 1)],
        C::Plate | C::Scale | C::Splint => &[(P::Sabatons, 1)],
        C::Leather if shaped => &[(P::Sabatons, 75), (P::FootGuards, 25)],
        C::Mail | C::RingMail => &[(P::MailShoes, 1)],
        _ => &[(P::FootGuards, 1)],
    }
}

/// The toes a style gives its sabatons, before the construction narrows them.
fn style_toes(style: Style) -> &'static [(Toe, u64)] {
    match style {
        Style::Gothic => &[(Toe::Pointed, 1)],
        Style::Milanese => &[(Toe::Round, 70), (Toe::Pointed, 30)],
        Style::Maximilian => &[(Toe::BearPaw, 50), (Toe::CowMouth, 50)],
        Style::Greenwich => &[(Toe::DuckBill, 50), (Toe::CowMouth, 50)],
        Style::Kastenbrust => &[(Toe::Pointed, 60), (Toe::Round, 40)],
        Style::Elven => &[(Toe::Pointed, 1)],
        Style::Dwarven => &[(Toe::CowMouth, 70), (Toe::Claw, 30)],
        _ => &[
            (Toe::Round, 40),
            (Toe::Pointed, 25),
            (Toe::CowMouth, 15),
            (Toe::DuckBill, 10),
            (Toe::BearPaw, 10),
        ],
    }
}

/// Whether a construction can be shaped to this toe: plate to any, leather to the
/// plain ones and a square front, scales and splints only round or pointed.
fn shapes(construction: Construction, toe: Toe) -> bool {
    match construction {
        Construction::Plate => true,
        Construction::Leather => matches!(toe, Toe::Round | Toe::Pointed | Toe::CowMouth),
        _ => matches!(toe, Toe::Round | Toe::Pointed),
    }
}

/// The toes a piece may have: the style's that its construction can be shaped to, or
/// round if none; mail shoes and foot guards follow the foot.
pub(super) fn toes(piece: FeetPiece, construction: Construction, style: Style) -> Vec<(Toe, u64)> {
    if piece != FeetPiece::Sabatons {
        return vec![(Toe::Round, 1)];
    }
    let toes: Vec<_> = style_toes(style)
        .iter()
        .copied()
        .filter(|&(toe, _)| shapes(construction, toe))
        .collect();
    if toes.is_empty() {
        vec![(Toe::Round, 1)]
    } else {
        toes
    }
}

/// Lames a sabaton may have (splints, for splinted ones): (least, most).
pub(super) fn lames(construction: Construction, style: Style, toe: Toe) -> (u8, u8) {
    match construction {
        Construction::Scale => (0, 0),
        Construction::Splint => (5, 8),
        Construction::Leather => (3, 5),
        _ if style == Style::Dwarven => (3, 5),
        _ if style == Style::Gothic => (6, 10),
        _ if toe == Toe::Pointed => (5, 10),
        _ => (4, 7),
    }
}

/// Plate and leather sabatons may rise over the ankle, all but pointed ones.
pub(super) fn may_be_high(piece: FeetPiece, construction: Construction, toe: Toe) -> bool {
    piece == FeetPiece::Sabatons
        && matches!(construction, Construction::Plate | Construction::Leather)
        && toe != Toe::Pointed
}

/// Long plate points may come off; elven points are short.
pub(super) fn may_detach(construction: Construction, style: Style, toe: Toe) -> bool {
    construction == Construction::Plate && toe == Toe::Pointed && style != Style::Elven
}

/// Cloth under it: Japanese pieces are built on tabi.
pub(super) fn has_cloth(style: Style) -> bool {
    style == Style::Japanese
}

/// Whether a list offers this, at a weight above nothing.
pub(super) fn has<T: PartialEq>(list: &[(T, u64)], wanted: T) -> bool {
    list.iter()
        .any(|(have, weight)| *have == wanted && *weight > 0)
}
