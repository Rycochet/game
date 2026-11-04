//! What waist pieces may be made in and how they hang, and how often: the lists rolls
//! pick from and the grid checks against.

use super::{Length, WaistKind, WaistPiece};
use crate::armour::{Construction, Style};

/// Constructions waist armour is made in, by style. Each keeps to `Style::allows`
/// (checked by a test), narrowed to what was worn at the waist; mirrors never were.
pub(super) fn constructions(kind: WaistKind, style: Style) -> &'static [(Construction, u64)] {
    use Construction as C;
    if kind == WaistKind::Cuirassier {
        return &[(C::Plate, 1)];
    }
    match style {
        Style::Elven => &[
            (C::Plate, 45),
            (C::Leather, 20),
            (C::Scale, 15),
            (C::Mail, 15),
            (C::Lamellar, 5),
        ],
        Style::Dwarven => &[
            (C::Plate, 55),
            (C::Mail, 15),
            (C::Leather, 10),
            (C::Laminar, 10),
            (C::MailAndPlate, 10),
        ],
        // Kusazuri.
        Style::Japanese => &[
            (C::Lamellar, 50),
            (C::Laminar, 25),
            (C::Leather, 10),
            (C::PlatesInCloth, 10),
            (C::Mail, 5),
        ],
        Style::Chinese => &[
            (C::Lamellar, 45),
            (C::Scale, 25),
            (C::PlatesInCloth, 20),
            (C::Leather, 10),
        ],
        Style::Steppe => &[
            (C::Lamellar, 40),
            (C::Scale, 15),
            (C::Mail, 15),
            (C::Laminar, 15),
            (C::MailAndPlate, 10),
            (C::Leather, 5),
        ],
        // Pteruges of leather and linen, bronze girdles and hip plates, scale and mail.
        Style::Classical => &[
            (C::Leather, 40),
            (C::Padded, 15),
            (C::Plate, 15),
            (C::Scale, 15),
            (C::Mail, 15),
        ],
        // Kremasmata, quilted or lamellar.
        Style::Byzantine => &[
            (C::Lamellar, 30),
            (C::Padded, 25),
            (C::Scale, 15),
            (C::Mail, 15),
            (C::Leather, 15),
        ],
        Style::IndoPersian => &[
            (C::MailAndPlate, 40),
            (C::Mail, 30),
            (C::Lamellar, 15),
            (C::Plate, 10),
            (C::Scale, 5),
        ],
        // The European harnesses.
        _ => &[
            (C::Plate, 55),
            (C::Mail, 15),
            (C::Leather, 10),
            (C::PlatesInCloth, 8),
            (C::Splint, 4),
            (C::Scale, 4),
            (C::RingMail, 4),
        ],
    }
}

/// What shapes of piece a construction makes, in a style.
pub(super) fn pieces(
    kind: WaistKind,
    construction: Construction,
    style: Style,
) -> &'static [(WaistPiece, u64)] {
    use Construction as C;
    use WaistPiece as P;
    if kind == WaistKind::Cuirassier {
        return &[(P::Tassets, 1)];
    }
    let european = style.european();
    match (construction, style) {
        // The mitra and Italic belts, and the Dendra panoply's hip plates.
        (C::Plate, Style::Classical) => &[(P::Girdle, 60), (P::Tassets, 40)],
        // The tonlet.
        (C::Plate, Style::Kastenbrust) => &[(P::Skirt, 50), (P::Tassets, 30), (P::Fauld, 20)],
        // Steel bases.
        (C::Plate, Style::Maximilian | Style::Greenwich) => {
            &[(P::Tassets, 60), (P::Skirt, 20), (P::Fauld, 20)]
        }
        (C::Plate, Style::Dwarven) => &[(P::Tassets, 50), (P::Fauld, 30), (P::Girdle, 20)],
        (C::Plate, Style::Elven) => &[(P::Tassets, 70), (P::Fauld, 30)],
        (C::Plate, _) => &[(P::Tassets, 60), (P::Fauld, 30), (P::Skirt, 10)],
        (C::Laminar, Style::Japanese) => &[(P::Panels, 1)],
        (C::Laminar, Style::Steppe) => &[(P::Panels, 50), (P::Skirt, 50)],
        (C::Laminar, _) => &[(P::Fauld, 50), (P::Skirt, 50)],
        (C::Leather, Style::Classical) => &[(P::Pteruges, 90), (P::Girdle, 10)],
        (C::Leather, Style::Japanese | Style::Chinese) => &[(P::Panels, 1)],
        (C::Leather, Style::Steppe) => &[(P::Panels, 60), (P::Skirt, 40)],
        (C::Leather, Style::Byzantine) => &[(P::Pteruges, 60), (P::Skirt, 40)],
        // War kilts among them.
        (C::Leather, Style::Elven | Style::Dwarven) => &[
            (P::Tassets, 40),
            (P::Pteruges, 30),
            (P::Girdle, 15),
            (P::Fauld, 15),
        ],
        (C::Leather, _) => &[
            (P::Tassets, 40),
            (P::Fauld, 20),
            (P::Skirt, 20),
            (P::Pteruges, 20),
        ],
        // Linen pteruges, quilted kremasmata.
        (C::Padded, Style::Classical) => &[(P::Pteruges, 1)],
        (C::Padded, _) => &[(P::Skirt, 1)],
        (C::Mail, _) if european || matches!(style, Style::Elven | Style::Dwarven) => {
            &[(P::Skirt, 75), (P::Breeches, 25)]
        }
        (C::Mail | C::RingMail, _) => &[(P::Skirt, 1)],
        (C::Scale | C::Lamellar, Style::Chinese | Style::Steppe) => {
            &[(P::Panels, 60), (P::Skirt, 40)]
        }
        (C::Lamellar, Style::Japanese) => &[(P::Panels, 1)],
        (C::Lamellar, Style::Byzantine) | (C::Scale, _) => &[(P::Skirt, 1)],
        (C::Lamellar, _) => &[(P::Panels, 50), (P::Skirt, 50)],
        (C::PlatesInCloth, Style::Japanese) => &[(P::Panels, 1)],
        (C::PlatesInCloth, Style::Chinese) => &[(P::Panels, 60), (P::Skirt, 40)],
        (C::PlatesInCloth, _) => &[(P::Skirt, 50), (P::Tassets, 50)],
        (C::MailAndPlate, Style::IndoPersian) => {
            &[(P::Skirt, 60), (P::Tassets, 30), (P::Girdle, 10)]
        }
        (C::MailAndPlate, Style::Dwarven) => &[(P::Skirt, 60), (P::Girdle, 40)],
        (C::MailAndPlate, _) => &[(P::Skirt, 1)],
        (C::Splint, _) => &[(P::Tassets, 1)],
        _ => &[(P::Skirt, 1)],
    }
}

/// How far down a piece hangs.
pub(super) fn lengths(
    kind: WaistKind,
    piece: WaistPiece,
    construction: Construction,
    style: Style,
) -> &'static [(Length, u64)] {
    use Construction as C;
    use Length as L;
    use WaistPiece as P;
    if kind == WaistKind::Cuirassier {
        return &[(L::Knee, 1)];
    }
    match (piece, construction) {
        (P::Girdle, _) => &[(L::Band, 1)],
        (P::Fauld | P::Breeches, _) => &[(L::Hip, 1)],
        // Gothic tassets are small, and the Dendra plates and the rest go to mid-thigh.
        (P::Tassets, C::Plate | C::Laminar | C::PlatesInCloth)
            if !matches!(style, Style::Gothic | Style::Classical) =>
        {
            &[(L::Thigh, 65), (L::Knee, 35)]
        }
        (P::Tassets, _) => &[(L::Thigh, 1)],
        (P::Panels, _) if style == Style::Japanese => &[(L::Thigh, 1)],
        (P::Panels, _) => &[(L::Thigh, 40), (L::Knee, 60)],
        (P::Pteruges, _) => &[(L::Hip, 50), (L::Thigh, 50)],
        // Tonlets and steel bases, and quilted kremasmata.
        (P::Skirt, C::Plate | C::Padded) => &[(L::Knee, 1)],
        (P::Skirt, C::Mail | C::RingMail) => &[(L::Thigh, 60), (L::Knee, 40)],
        (P::Skirt, _) => &[(L::Thigh, 50), (L::Knee, 50)],
    }
}

/// Whether a list has this.
pub(super) fn has<T: PartialEq>(list: &[(T, u64)], wanted: T) -> bool {
    list.iter()
        .any(|(have, weight)| *have == wanted && *weight > 0)
}
