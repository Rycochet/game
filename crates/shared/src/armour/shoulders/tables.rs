//! How often each choice is rolled, by kind, construction and style.

use super::{Adornment, CapeCut, Cut, Finish, ShouldersKind, ShouldersPiece};
use crate::armour::{Construction, Style, in_style};
use crate::material::Material;

/// How often each kind is made.
pub(super) const KINDS: [(ShouldersKind, u64); 7] = [
    (ShouldersKind::Pauldrons, 30),
    (ShouldersKind::Spaulders, 22),
    (ShouldersKind::Guards, 20),
    (ShouldersKind::Cape, 12),
    (ShouldersKind::Discs, 6),
    (ShouldersKind::Ailettes, 7),
    (ShouldersKind::Wing, 3),
];

/// Which constructions a kind is made in, and how often.
pub(super) fn constructions(kind: ShouldersKind) -> &'static [(Construction, u64)] {
    use Construction as C;
    match kind {
        ShouldersKind::Pauldrons => &[
            (C::Plate, 60),
            (C::Leather, 20),
            (C::Scale, 10),
            (C::MailAndPlate, 10),
        ],
        ShouldersKind::Spaulders => &[(C::Plate, 55), (C::Leather, 25), (C::Laminar, 20)],
        ShouldersKind::Guards => &[
            (C::Lamellar, 35),
            (C::Scale, 20),
            (C::Laminar, 15),
            (C::PlatesInCloth, 15),
            (C::Leather, 10),
            (C::Mail, 5),
        ],
        ShouldersKind::Cape => &[
            (C::Mail, 45),
            (C::Scale, 25),
            (C::Padded, 20),
            (C::Leather, 10),
        ],
        ShouldersKind::Discs => &[(C::MailAndPlate, 40), (C::Plate, 30), (C::Mirror, 30)],
        ShouldersKind::Ailettes => &[(C::Leather, 70), (C::Plate, 30)],
        ShouldersKind::Wing => &[(C::Laminar, 1)],
    }
}

/// Lamellae and lames laced together.
pub(super) fn laced(construction: Construction) -> bool {
    matches!(construction, Construction::Lamellar | Construction::Laminar)
}

/// The constructions a kind is made in this style. Padded and leather yokes are only
/// Greek.
pub(super) fn constructions_in(kind: ShouldersKind, style: Style) -> Vec<(Construction, u64)> {
    let list: Vec<_> = constructions(kind)
        .iter()
        .copied()
        .filter(|&(construction, _)| {
            kind != ShouldersKind::Cape
                || construction != Construction::Padded
                || style == Style::Classical
        })
        .collect();
    in_style(&list, style)
}

/// Fantastic things go on rigid tops; a beast's head on eastern-looking pieces and
/// plate pauldrons; nothing on capes, ailettes and the wing.
pub(super) fn adornments(
    kind: ShouldersKind,
    construction: Construction,
) -> &'static [(Adornment, u64)] {
    use Adornment::{BeastHead, Horns, None, Skull, Spikes};
    match kind {
        ShouldersKind::Pauldrons if construction == Construction::Plate => &[
            (None, 76),
            (Spikes(1), 3),
            (Spikes(3), 5),
            (Spikes(5), 3),
            (Horns, 4),
            (BeastHead, 3),
            (Skull, 6),
        ],
        ShouldersKind::Pauldrons | ShouldersKind::Spaulders => &[
            (None, 82),
            (Spikes(1), 3),
            (Spikes(3), 5),
            (Spikes(5), 2),
            (Horns, 3),
            (Skull, 5),
        ],
        ShouldersKind::Guards => &[(None, 90), (BeastHead, 6), (Skull, 4)],
        ShouldersKind::Discs => &[(None, 85), (Spikes(1), 10), (Skull, 5)],
        _ => &[(None, 1)],
    }
}

/// Leather is plain, tooled or studded; ring mail on guards and round capes.
pub(super) fn finishes(
    kind: ShouldersKind,
    construction: Construction,
    piece: &ShouldersPiece,
) -> &'static [(Finish, u64)] {
    use Finish::{Plain, RingMail, Studded, Tooled};
    if construction != Construction::Leather || kind == ShouldersKind::Ailettes {
        return &[(Plain, 1)];
    }
    match piece {
        ShouldersPiece::Cape {
            cut: CapeCut::Round,
            ..
        } => &[(RingMail, 1)],
        ShouldersPiece::Guards { .. } => {
            &[(Plain, 30), (Tooled, 15), (Studded, 35), (RingMail, 20)]
        }
        _ => &[(Plain, 40), (Tooled, 25), (Studded, 35)],
    }
}

pub(super) fn materials(
    kind: ShouldersKind,
    construction: Construction,
    style: Style,
) -> &'static [(Material, u64)] {
    use Construction as C;
    use Material as M;
    match (kind, construction) {
        // Greek plate is bronze.
        (_, C::Plate) if style == Style::Classical => &[(M::Bronze, 1)],
        (ShouldersKind::Wing, _) => &[(M::Wood, 60), (M::HardenedLeather, 40)],
        (ShouldersKind::Ailettes, C::Leather) => &[(M::HardenedLeather, 1)],
        (ShouldersKind::Ailettes, _) => &[(M::Steel, 40), (M::Iron, 30), (M::Wood, 30)],
        (_, C::Plate) => &[
            (M::Steel, 55),
            (M::Iron, 35),
            (M::Chitin, 4),
            (M::Mithril, 3),
            (M::Adamantine, 2),
        ],
        (_, C::Mail) => &[
            (M::Iron, 50),
            (M::Steel, 40),
            (M::Bronze, 5),
            (M::Mithril, 4),
            (M::Adamantine, 1),
        ],
        (_, C::Scale) => &[
            (M::Iron, 30),
            (M::Steel, 25),
            (M::Bronze, 15),
            (M::Horn, 12),
            (M::HardenedLeather, 10),
            (M::Chitin, 3),
            (M::DragonScale, 3),
            (M::Mithril, 2),
        ],
        (_, C::Lamellar) => &[
            (M::Iron, 40),
            (M::Steel, 20),
            (M::HardenedLeather, 25),
            (M::Horn, 8),
            (M::DragonScale, 4),
            (M::Chitin, 3),
        ],
        (_, C::Laminar) => &[
            (M::Iron, 50),
            (M::Steel, 30),
            (M::HardenedLeather, 15),
            (M::Mithril, 5),
        ],
        (_, C::PlatesInCloth | C::MailAndPlate) => &[(M::Steel, 55), (M::Iron, 45)],
        (_, C::Mirror) => &[(M::Steel, 70), (M::Iron, 25), (M::Mithril, 5)],
        (_, C::Leather) => &[
            (M::HardenedLeather, 55),
            (M::BuffLeather, 25),
            (M::Leather, 20),
        ],
        _ => &[(M::Linen, 75), (M::Cotton, 15), (M::Wool, 10)],
    }
}

/// Cloth over plates in cloth, and lacing through lamellae and lames.
pub(super) fn needs_cloth(construction: Construction) -> bool {
    matches!(
        construction,
        Construction::PlatesInCloth | Construction::Lamellar | Construction::Laminar
    )
}

pub(super) fn cloths(construction: Construction) -> &'static [(Material, u64)] {
    use Material as M;
    match construction {
        Construction::PlatesInCloth => &[(M::Linen, 40), (M::Wool, 35), (M::Silk, 25)],
        // Lacing.
        _ => &[(M::Silk, 50), (M::Leather, 50)],
    }
}

pub(super) fn may_have_trim(
    kind: ShouldersKind,
    construction: Construction,
    material: Material,
) -> bool {
    let metal = matches!(
        material,
        Material::Steel | Material::Iron | Material::Mithril
    );
    match kind {
        ShouldersKind::Pauldrons | ShouldersKind::Spaulders | ShouldersKind::Discs => {
            metal && matches!(construction, Construction::Plate | Construction::Mirror)
        }
        ShouldersKind::Cape => construction == Construction::Mail,
        _ => false,
    }
}

/// How the cop and lames of plate look by style: Gothic and Kastenbrust pairs are
/// spaulders more often, Milanese pauldrons are large and often uneven.
pub(super) fn pauldron_cut(style: Style) -> &'static [(Cut, u64)] {
    match style {
        Style::Milanese => &[
            (Cut::Matched, 40),
            (Cut::GreatLeft, 40),
            (Cut::LanceCut, 20),
        ],
        _ => &[(Cut::Matched, 80), (Cut::GreatLeft, 8), (Cut::LanceCut, 12)],
    }
}

pub(super) fn kinds_for(style: Style) -> Vec<(ShouldersKind, u64)> {
    let fantasy = matches!(style, Style::Elven | Style::Dwarven);
    KINDS
        .iter()
        .map(|&(kind, weight)| {
            let weight = match (kind, style) {
                // Ailettes are European, the wing from the far north-east of the steppe.
                (ShouldersKind::Ailettes, _) if !style.european() && !fantasy => 0,
                (ShouldersKind::Wing, _) if style != Style::Steppe => 0,
                (ShouldersKind::Spaulders, Style::Gothic | Style::Kastenbrust) => weight * 2,
                (
                    ShouldersKind::Pauldrons,
                    Style::Milanese | Style::Maximilian | Style::Greenwich,
                ) => weight * 2,
                (ShouldersKind::Guards, _) if !style.european() && !fantasy => weight * 3,
                (ShouldersKind::Cape | ShouldersKind::Spaulders, Style::Classical) => weight * 3,
                (ShouldersKind::Discs, Style::Steppe | Style::IndoPersian) => weight * 4,
                _ => weight,
            };
            (kind, weight)
        })
        .filter(|&(_, weight)| weight > 0)
        .collect()
}
