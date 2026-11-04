//! What torso armour may be made of and shaped like, and how often: the lists rolls
//! pick from and the grid checks against.

use super::Breast;
use crate::armour::{Construction, Style, in_style};
use crate::material::Material;

/// Constructions torso armour is made in, before the style narrows them.
const CONSTRUCTIONS: [(Construction, u64); 13] = [
    (Construction::Plate, 30),
    (Construction::Mail, 18),
    (Construction::Leather, 12),
    (Construction::Padded, 9),
    (Construction::RingMail, 5),
    (Construction::Scale, 10),
    (Construction::Lamellar, 10),
    (Construction::Laminar, 6),
    (Construction::PlatesInCloth, 10),
    (Construction::MailAndPlate, 5),
    (Construction::Mirror, 3),
    (Construction::Splint, 3),
    (Construction::Woven, 2),
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

/// Leather that hangs rather than holding a shape: buff and raw hide.
pub(super) fn soft_leather(material: Material) -> bool {
    matches!(material, Material::BuffLeather | Material::Rawhide)
}

/// Styles a rigid piece may be worn over its own mail in: European, the fantasy
/// harnesses and Indo-Persian.
pub(super) fn may_be_over_mail(style: Style) -> bool {
    style.european() || matches!(style, Style::Elven | Style::Dwarven | Style::IndoPersian)
}
