//! What waist pieces may be made of, by construction and style.

use crate::armour::{Construction, Style};
use crate::material::Material;

/// What the main part may be made of.
pub(super) fn materials(construction: Construction, style: Style) -> &'static [(Material, u64)] {
    use Construction as C;
    use Material as M;
    match (construction, style) {
        // Mitrai, Italic belts and the Dendra plates.
        (C::Plate, Style::Classical) => &[(M::Bronze, 1)],
        (C::Plate, Style::Elven) => &[(M::Steel, 50), (M::Mithril, 35), (M::Bronze, 15)],
        (C::Plate, Style::Dwarven) => &[(M::Steel, 50), (M::Iron, 25), (M::Adamantine, 25)],
        (C::Plate, Style::IndoPersian) => &[(M::Iron, 60), (M::Steel, 40)],
        (C::Plate, _) => &[
            (M::Steel, 55),
            (M::Iron, 35),
            (M::Mithril, 4),
            (M::Adamantine, 3),
            (M::Chitin, 3),
        ],
        // Pteruges hang soft; tassets and faulds are hardened.
        (C::Leather, Style::Classical | Style::Byzantine) => {
            &[(M::Leather, 60), (M::HardenedLeather, 40)]
        }
        (C::Leather, _) => &[(M::HardenedLeather, 70), (M::Leather, 30)],
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
        (C::Scale, Style::Classical) => &[(M::Bronze, 60), (M::Iron, 40)],
        (C::Scale, Style::Elven) => &[(M::Steel, 40), (M::Mithril, 30), (M::DragonScale, 30)],
        (C::Scale, _) => &[
            (M::Iron, 45),
            (M::Steel, 25),
            (M::HardenedLeather, 15),
            (M::Horn, 5),
            (M::DragonScale, 5),
            (M::Chitin, 5),
        ],
        (C::Lamellar, Style::Japanese) => &[(M::Iron, 60), (M::HardenedLeather, 40)],
        (C::Lamellar, _) => &[
            (M::Iron, 50),
            (M::Steel, 20),
            (M::HardenedLeather, 20),
            (M::Horn, 10),
        ],
        (C::Laminar, Style::Japanese) => &[(M::Iron, 70), (M::HardenedLeather, 30)],
        (C::Laminar, _) => &[(M::Iron, 60), (M::Steel, 25), (M::HardenedLeather, 15)],
        (C::PlatesInCloth | C::MailAndPlate, _) => &[(M::Iron, 60), (M::Steel, 40)],
        (C::Splint, _) => &[(M::Iron, 50), (M::Steel, 30), (M::HardenedLeather, 20)],
        _ => &[(M::Linen, 60), (M::Cotton, 25), (M::Wool, 15)],
    }
}
