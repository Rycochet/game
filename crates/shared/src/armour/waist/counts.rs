//! How many hoops, lames, rows, tassets or panels a waist piece has, and where a skirt
//! is split: the lists rolls pick from and the grid checks against.

use super::{Length, Opening, WaistKind, WaistPiece};
use crate::armour::{Construction, Style};

/// How many hoops, lames or rows a piece has, by what it is.
pub(super) fn lames(
    kind: WaistKind,
    piece: WaistPiece,
    construction: Construction,
    style: Style,
    length: Length,
) -> &'static [(u8, u64)] {
    use Construction as C;
    use WaistPiece as P;
    if kind == WaistKind::Cuirassier {
        return &[(10, 30), (11, 30), (12, 25), (13, 15)];
    }
    let laminated = matches!(construction, C::Plate | C::Laminar);
    match piece {
        P::Fauld if laminated => &[(2, 10), (3, 30), (4, 35), (5, 15), (6, 10)],
        // Leather faulds in one piece.
        P::Fauld => &[(1, 1)],
        // Single plates in the 15th century and the regional and fantasy styles; almain
        // rivets' lames later.
        P::Tassets if !laminated => &[(1, 1)],
        P::Tassets if length == Length::Knee => &[(8, 25), (9, 25), (10, 20), (11, 15), (12, 15)],
        P::Tassets => match style {
            Style::Gothic | Style::Milanese | Style::Classical | Style::Elven => &[(1, 1)],
            Style::Maximilian | Style::Greenwich => &[(3, 20), (4, 30), (5, 30), (6, 20)],
            _ => &[(1, 40), (3, 15), (4, 25), (5, 20)],
        },
        P::Panels if style == Style::Japanese => &[(5, 1)],
        P::Panels => &[(4, 30), (5, 40), (6, 30)],
        P::Pteruges if length == Length::Hip => &[(2, 60), (3, 40)],
        P::Pteruges => &[(1, 50), (2, 50)],
        P::Skirt if construction == C::Plate => &[(5, 20), (6, 35), (7, 30), (8, 15)],
        P::Skirt
            if matches!(
                construction,
                C::Scale | C::Lamellar | C::Laminar | C::MailAndPlate
            ) =>
        {
            if length == Length::Knee {
                &[(9, 30), (10, 40), (12, 30)]
            } else {
                &[(6, 30), (7, 40), (8, 30)]
            }
        }
        _ => &[(0, 1)],
    }
}

/// How many hoops the culet behind a fauld or tassets has, or none.
pub(super) fn culets(piece: WaistPiece, construction: Construction) -> &'static [(u8, u64)] {
    match (piece, construction) {
        (WaistPiece::Fauld | WaistPiece::Tassets, Construction::Plate | Construction::Laminar) => {
            &[(0, 40), (2, 25), (3, 25), (4, 10)]
        }
        _ => &[(0, 1)],
    }
}

/// How many tassets or panels hang from it.
pub(super) fn hangs(kind: WaistKind, piece: WaistPiece, style: Style) -> &'static [(u8, u64)] {
    match piece {
        _ if kind == WaistKind::Cuirassier => &[(2, 1)],
        // Small tassets at the sides too, on 15th-century harnesses.
        WaistPiece::Tassets
            if matches!(style, Style::Gothic | Style::White | Style::Kastenbrust) =>
        {
            &[(2, 65), (4, 35)]
        }
        // The Dendra panoply's three pairs.
        WaistPiece::Tassets if style == Style::Classical => &[(6, 1)],
        WaistPiece::Tassets => &[(2, 1)],
        // The ō-yoroi's four, the dō-maru's seven.
        WaistPiece::Panels if style == Style::Japanese => &[(4, 20), (5, 15), (7, 50), (8, 15)],
        // Two big flaps, split front and back, or four.
        WaistPiece::Panels => &[(2, 70), (4, 30)],
        _ => &[(0, 1)],
    }
}

/// Where a skirt is split.
pub(super) fn openings(piece: WaistPiece, construction: Construction) -> &'static [(Opening, u64)] {
    use Construction as C;
    use Opening as O;
    match (piece, construction) {
        (WaistPiece::Skirt, C::Mail | C::RingMail) => &[(O::Closed, 60), (O::FrontAndBack, 40)],
        (WaistPiece::Skirt, C::PlatesInCloth | C::Leather) => {
            &[(O::FrontAndBack, 70), (O::Closed, 30)]
        }
        (WaistPiece::Skirt, C::Scale | C::Lamellar | C::Laminar | C::MailAndPlate) => {
            &[(O::FrontAndBack, 50), (O::Sides, 30), (O::Closed, 20)]
        }
        (WaistPiece::Skirt, C::Padded) => &[(O::Sides, 50), (O::Closed, 50)],
        _ => &[(O::Closed, 1)],
    }
}
