//! What torso armour may be made of and shaped like, and how often: the lists rolls
//! pick from and the grid checks against.

use super::Breast;
use crate::armour::{Construction, Style, in_style};
use crate::material::Material;

/// Constructions torso armour is made in, before the style narrows them.
const CONSTRUCTIONS: [(Construction, u64); 5] = [
    (Construction::Plate, 40),
    (Construction::Mail, 25),
    (Construction::Leather, 15),
    (Construction::Padded, 12),
    (Construction::RingMail, 8),
];

pub(super) fn constructions(style: Style) -> Vec<(Construction, u64)> {
    in_style(&CONSTRUCTIONS, style)
}

/// The breasts plate is shaped in, in this style.
pub(super) fn plate_breasts(style: Style) -> &'static [(Breast, u64)] {
    use Breast as B;
    match style {
        Style::White => &[(B::Round, 70), (B::Munition, 30)],
        Style::Gothic => &[(B::Gothic, 1)],
        Style::Milanese => &[(B::Globose, 1)],
        Style::Greenwich => &[(B::Peascod, 1)],
        Style::Kastenbrust => &[(B::Box, 1)],
        Style::Elven => &[(B::Leaf, 1)],
        Style::Dwarven => &[(B::Panel, 1)],
        Style::Japanese => &[(B::Tanko, 1)],
        Style::Classical => &[(B::Muscle, 55), (B::Bell, 25), (B::Discs, 20)],
        _ => &[(B::Round, 1)],
    }
}

/// The breasts a leather cuirass is shaped in, in this style.
pub(super) fn leather_breasts(style: Style) -> &'static [(Breast, u64)] {
    match style {
        Style::Classical => &[(Breast::Muscle, 1)],
        Style::Elven => &[(Breast::Leaf, 1)],
        Style::Dwarven => &[(Breast::Panel, 1)],
        _ => &[(Breast::Round, 1)],
    }
}

/// What the main part may be made of.
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
            (M::HardenedLeather, 55),
            (M::BuffLeather, 25),
            (M::Leather, 20),
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
        _ => &[
            (M::Linen, 55),
            (M::Cotton, 25),
            (M::Wool, 15),
            (M::Silk, 3),
            (M::Coir, 2),
        ],
    }
}

/// What an arming vest is made of.
pub(super) const VEST_MATERIALS: [(Material, u64); 3] = [
    (Material::Linen, 70),
    (Material::Cotton, 20),
    (Material::Wool, 10),
];

/// Whether a list has this.
pub(super) fn has<T: PartialEq>(list: &[(T, u64)], wanted: T) -> bool {
    list.iter()
        .any(|(have, weight)| *have == wanted && *weight > 0)
}

/// Breasts that may be made without a back: plain and soldiers' breastplates, and
/// the single disc over the heart (kardiophylax).
pub(super) fn may_lack_back(breast: Breast) -> bool {
    matches!(
        breast,
        Breast::Round | Breast::Munition | Breast::Globose | Breast::Peascod | Breast::Discs
    )
}

/// Styles a rigid piece may be worn over its own mail in: European, the fantasy
/// harnesses and Indo-Persian.
pub(super) fn may_be_over_mail(style: Style) -> bool {
    style.european() || matches!(style, Style::Elven | Style::Dwarven | Style::IndoPersian)
}
