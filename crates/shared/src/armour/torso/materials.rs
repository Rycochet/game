//! What the main part of torso armour may be made of, by construction and style.

use crate::armour::{Construction, Style};
use crate::material::Material;

/// What the main part may be made of: the plates, rings, scales, leather, padding,
/// slats or rattan.
pub(super) fn materials(construction: Construction, style: Style) -> &'static [(Material, u64)] {
    use Construction as C;
    use Material as M;
    match (construction, style) {
        (C::Plate, Style::Classical) => &[(M::Bronze, 80), (M::Iron, 20)],
        (C::Plate, Style::Japanese) => &[(M::Iron, 1)],
        (C::Plate, Style::Elven) => &[(M::Steel, 50), (M::Mithril, 35), (M::Bronze, 15)],
        (C::Plate, Style::Dwarven) => &[(M::Steel, 50), (M::Iron, 25), (M::Adamantine, 25)],
        (C::Plate, _) => &[
            (M::Steel, 55),
            (M::Iron, 35),
            (M::Mithril, 4),
            (M::Adamantine, 3),
            (M::Chitin, 3),
        ],
        (C::Leather, Style::Classical) => &[(M::HardenedLeather, 1)],
        (C::Leather, _) => &[
            (M::HardenedLeather, 50),
            (M::BuffLeather, 22),
            (M::Leather, 18),
            (M::Rawhide, 10),
        ],
        (C::Mail, Style::Elven) => &[(M::Mithril, 50), (M::Steel, 50)],
        (C::Mail, Style::Classical) => &[(M::Iron, 70), (M::Bronze, 30)],
        (C::Mail, _) => &[
            (M::Iron, 50),
            (M::Steel, 40),
            (M::Bronze, 5),
            (M::Mithril, 4),
            (M::Adamantine, 1),
        ],
        (C::RingMail, _) => &[(M::Iron, 50), (M::Steel, 30), (M::Brass, 20)],
        (C::Scale, Style::Classical) => &[(M::Bronze, 60), (M::Iron, 40)],
        (C::Scale, Style::Elven) => &[(M::Mithril, 40), (M::Bronze, 30), (M::Steel, 30)],
        (C::Scale, _) => &[
            (M::Iron, 30),
            (M::Steel, 20),
            (M::Bronze, 12),
            (M::Brass, 6),
            (M::Horn, 10),
            (M::HardenedLeather, 10),
            (M::DragonScale, 4),
            (M::Chitin, 3),
            (M::Bone, 3),
            (M::Mithril, 2),
        ],
        (C::Lamellar, _) => &[
            (M::Iron, 40),
            (M::Steel, 20),
            (M::HardenedLeather, 24),
            (M::Horn, 7),
            (M::Bone, 5),
            (M::Chitin, 4),
        ],
        (C::Laminar, Style::Classical) => &[(M::Iron, 1)],
        (C::Laminar, _) => &[
            (M::Iron, 50),
            (M::Steel, 30),
            (M::HardenedLeather, 15),
            (M::Mithril, 5),
        ],
        (C::PlatesInCloth, _) => &[
            (M::Steel, 50),
            (M::Iron, 40),
            (M::Horn, 5),
            (M::HardenedLeather, 5),
        ],
        (C::MailAndPlate, _) => &[(M::Steel, 55), (M::Iron, 45)],
        (C::Mirror, _) => &[(M::Steel, 70), (M::Iron, 25), (M::Mithril, 5)],
        (C::Splint, _) => &[(M::Wood, 40), (M::Bone, 15), (M::Steel, 25), (M::Iron, 20)],
        (C::Woven, _) => &[(M::Rattan, 1)],
        _ => &[
            (M::Linen, 55),
            (M::Cotton, 25),
            (M::Wool, 15),
            (M::Silk, 3),
            (M::Coir, 2),
        ],
    }
}
