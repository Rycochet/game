//! What head pieces may be made of, by construction and style.

use crate::armour::{Construction, Style};
use crate::material::Material;

/// What the main part may be made of.
pub(super) fn materials(construction: Construction, style: Style) -> &'static [(Material, u64)] {
    use Construction as C;
    use Material as M;
    match (construction, style) {
        (C::Plate, Style::Classical) => &[(M::Bronze, 80), (M::Iron, 20)],
        (C::Plate, Style::Japanese) | (C::Laminar, Style::Japanese) => &[(M::Iron, 1)],
        (C::Plate, Style::Elven) => &[(M::Steel, 50), (M::Mithril, 35), (M::Bronze, 15)],
        (C::Plate, Style::Dwarven) => &[(M::Steel, 50), (M::Iron, 25), (M::Adamantine, 25)],
        (C::Plate, Style::IndoPersian) => &[(M::Steel, 60), (M::Iron, 40)],
        (C::Plate, _) => &[
            (M::Steel, 55),
            (M::Iron, 35),
            (M::Mithril, 4),
            (M::Adamantine, 3),
            (M::Copper, 3),
        ],
        (C::Laminar, _) => &[(M::Iron, 70), (M::Steel, 20), (M::HardenedLeather, 10)],
        (C::Lamellar, _) => &[
            (M::Iron, 50),
            (M::Steel, 20),
            (M::HardenedLeather, 20),
            (M::Horn, 10),
        ],
        (C::Leather, _) => &[(M::HardenedLeather, 70), (M::Leather, 30)],
        (C::Woven, _) => &[(M::Rattan, 1)],
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
        _ => &[(M::Linen, 60), (M::Cotton, 20), (M::Wool, 20)],
    }
}

/// Coifs, by how often each is made before the style narrows them.
pub(super) const COIFS: [(Construction, u64); 3] = [
    (Construction::Mail, 60),
    (Construction::Padded, 30),
    (Construction::RingMail, 10),
];

/// Whether a list has this.
pub(super) fn has<T: PartialEq>(list: &[(T, u64)], wanted: T) -> bool {
    list.iter()
        .any(|(have, weight)| *have == wanted && *weight > 0)
}
