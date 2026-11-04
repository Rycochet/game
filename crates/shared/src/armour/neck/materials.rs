//! What neck pieces may be made of, by construction and style.

use crate::armour::{Construction, Style};
use crate::material::Material;

/// What the main part may be made of: the plates, rings, scales, leather or padding.
pub(super) fn materials(construction: Construction, style: Style) -> &'static [(Material, u64)] {
    use Construction as C;
    use Material as M;
    match (construction, style) {
        (C::Plate, Style::Classical) => &[(M::Bronze, 1)],
        (C::Plate, Style::Elven) => &[(M::Steel, 50), (M::Mithril, 35), (M::Bronze, 15)],
        (C::Plate, Style::Dwarven) => &[(M::Steel, 50), (M::Iron, 25), (M::Adamantine, 25)],
        (C::Plate, Style::Japanese) => &[(M::Iron, 60), (M::Steel, 40)],
        (C::Plate, _) => &[
            (M::Steel, 55),
            (M::Iron, 35),
            (M::Mithril, 4),
            (M::Adamantine, 3),
            (M::Chitin, 3),
        ],
        (C::Leather, _) => &[(M::HardenedLeather, 80), (M::Leather, 20)],
        (C::Mail, Style::Elven) => &[(M::Mithril, 50), (M::Steel, 50)],
        (C::Mail, Style::Japanese) => &[(M::Iron, 1)],
        (C::Mail, _) => &[
            (M::Iron, 50),
            (M::Steel, 40),
            (M::Bronze, 5),
            (M::Mithril, 4),
            (M::Adamantine, 1),
        ],
        (C::RingMail, _) => &[(M::Iron, 50), (M::Steel, 30), (M::Brass, 20)],
        (C::Scale, Style::Classical) => &[(M::Bronze, 70), (M::Iron, 30)],
        (C::Scale, Style::Elven) => &[(M::Steel, 40), (M::Mithril, 30), (M::DragonScale, 30)],
        (C::Scale, _) => &[
            (M::Iron, 45),
            (M::Steel, 25),
            (M::HardenedLeather, 15),
            (M::Horn, 5),
            (M::DragonScale, 5),
            (M::Chitin, 5),
        ],
        (C::Lamellar, _) => &[
            (M::Iron, 45),
            (M::HardenedLeather, 30),
            (M::Steel, 20),
            (M::Horn, 5),
        ],
        // The nodowa's lames.
        (C::Laminar, _) => &[(M::Iron, 50), (M::HardenedLeather, 35), (M::Steel, 15)],
        (C::Padded, _) => &[(M::Linen, 50), (M::Cotton, 30), (M::Wool, 20)],
        // Kikko's small plates.
        (C::PlatesInCloth, _) => &[(M::Iron, 60), (M::HardenedLeather, 40)],
        // Mail and plate.
        _ => &[(M::Iron, 60), (M::Steel, 40)],
    }
}
