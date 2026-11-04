//! What leg pieces may be made of, by construction and style.

use crate::armour::{Construction, Style};
use crate::material::Material;

/// What the main part may be made of.
pub(super) fn materials(construction: Construction, style: Style) -> &'static [(Material, u64)] {
    use Construction as C;
    use Material as M;
    match (construction, style) {
        (C::Plate, Style::Classical) => &[(M::Bronze, 1)],
        (C::Plate, Style::Elven) => &[(M::Steel, 50), (M::Mithril, 35), (M::Bronze, 15)],
        (C::Plate, Style::Dwarven) => &[(M::Steel, 50), (M::Iron, 25), (M::Adamantine, 25)],
        (C::Plate, Style::Japanese | Style::IndoPersian) => &[(M::Iron, 60), (M::Steel, 40)],
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
        (C::Laminar, Style::Classical) => &[(M::Iron, 70), (M::Bronze, 30)],
        (C::Laminar, _) => &[(M::Iron, 60), (M::HardenedLeather, 40)],
        (C::PlatesInCloth | C::MailAndPlate, _) => &[(M::Iron, 60), (M::Steel, 40)],
        // The Strategikon's iron or wooden greaves.
        (C::Splint, Style::Byzantine) => &[(M::Iron, 70), (M::Wood, 30)],
        (C::Splint, Style::Japanese) => &[(M::Iron, 80), (M::HardenedLeather, 20)],
        (C::Splint, _) => &[
            (M::Iron, 40),
            (M::Steel, 25),
            (M::HardenedLeather, 20),
            (M::Wood, 5),
            (M::Bone, 5),
            (M::Horn, 5),
        ],
        _ => &[(M::Linen, 60), (M::Cotton, 25), (M::Wool, 15)],
    }
}
