//! Rolling what armour of many small plates has of its own: scale, lamellar, laminar,
//! plates in cloth, plated mail, mirrors, slats and rattan. The lists here are what the
//! grid checks against too.

use super::roll::one_in;
use super::{
    Backing, Lacing, LamellarCut, Mirrors, PlateRows, PlateSize, Rivets, ScaleShape, TorsoBody,
};
use crate::armour::{Construction, Style};
use crate::material::{Category, Material};
use crate::rng::Rng;

pub(super) fn scale_shapes(style: Style, material: Material) -> &'static [(ScaleShape, u64)] {
    use ScaleShape as S;
    match (style, material) {
        (Style::Elven, _) => &[(S::Leaf, 50), (S::Rounded, 50)],
        // Brass coins sewn on in rows, as Tlingit coin armour.
        (_, Material::Brass) => &[(S::Coin, 60), (S::Rounded, 40)],
        (_, Material::DragonScale) => &[(S::Pointed, 50), (S::Rounded, 50)],
        _ => &[(S::Rounded, 45), (S::Pointed, 30), (S::Square, 25)],
    }
}

pub(super) fn scale_sizes(style: Style) -> &'static [(PlateSize, u64)] {
    match style {
        // Roman scales were small.
        Style::Classical => &[(PlateSize::Small, 1)],
        _ => &[
            (PlateSize::Small, 30),
            (PlateSize::Medium, 50),
            (PlateSize::Large, 20),
        ],
    }
}

pub(super) fn backings(style: Style, material: Material) -> &'static [(Backing, u64)] {
    match (style, material.category()) {
        (Style::Classical, _) => &[(Backing::Cloth, 70), (Backing::Mail, 30)],
        (_, Category::Metal) => &[(Backing::Cloth, 60), (Backing::Leather, 40)],
        _ => &[(Backing::Leather, 1)],
    }
}

pub(super) fn lamellar_cuts(style: Style) -> &'static [(LamellarCut, u64)] {
    use LamellarCut as L;
    match style {
        Style::Japanese => &[(L::Wrap, 45), (L::OpenBack, 25), (L::Boxed, 30)],
        Style::Chinese => &[(L::Mountain, 40), (L::Wrap, 60)],
        _ => &[(L::Wrap, 1)],
    }
}

pub(super) fn lamellar_sizes(style: Style) -> &'static [(PlateSize, u64)] {
    match style {
        // Kozane.
        Style::Japanese => &[(PlateSize::Small, 1)],
        Style::Steppe => &[(PlateSize::Medium, 50), (PlateSize::Large, 50)],
        _ => &[(PlateSize::Small, 30), (PlateSize::Medium, 70)],
    }
}

pub(super) fn lacings(style: Style) -> &'static [(Lacing, u64)] {
    match style {
        Style::Japanese => &[(Lacing::Full, 70), (Lacing::Spaced, 30)],
        _ => &[(Lacing::Full, 1)],
    }
}

/// Riveted lames (Roman, Japanese okegawa) or laced ones.
pub(super) fn laminar_riveting(style: Style) -> &'static [(bool, u64)] {
    match style {
        Style::Classical => &[(true, 1)],
        Style::Japanese => &[(true, 70), (false, 30)],
        _ => &[(true, 50), (false, 50)],
    }
}

/// How many lames go round the body.
pub(super) const LAMES: std::ops::RangeInclusive<u8> = 5..=8;

pub(super) fn cloth_sizes(style: Style) -> &'static [(PlateSize, u64)] {
    match style {
        // Karuta, in tatami armour.
        Style::Japanese => &[(PlateSize::Small, 1)],
        Style::Chinese => &[(PlateSize::Medium, 1)],
        _ if style.european() => &[
            (PlateSize::Small, 50),
            (PlateSize::Medium, 25),
            (PlateSize::Large, 25),
        ],
        _ => &[(PlateSize::Small, 60), (PlateSize::Medium, 40)],
    }
}

/// Stitched-in plates (a jack) only small and European; big studs gilt on Chinese work.
pub(super) fn rivets(style: Style, size: PlateSize) -> &'static [(Rivets, u64)] {
    use Rivets as R;
    match (style, size) {
        (Style::Chinese, _) => &[(R::Gilt, 40), (R::Iron, 60)],
        (_, PlateSize::Large) => &[(R::Iron, 70), (R::Tinned, 30)],
        (_, PlateSize::Small) if style.european() => &[
            (R::Hidden, 30),
            (R::Iron, 30),
            (R::Tinned, 25),
            (R::Gilt, 15),
        ],
        _ => &[(R::Iron, 50), (R::Tinned, 35), (R::Gilt, 15)],
    }
}

/// A leather cover, studded leather's real ancestor: European, not on a jack.
pub(super) fn may_have_leather_cover(style: Style, rivets: Rivets) -> bool {
    style.european() && rivets != Rivets::Hidden
}

pub(super) fn plate_rows(style: Style) -> &'static [(PlateRows, u64)] {
    use PlateRows as P;
    match style {
        Style::Steppe => &[(P::Wide, 50), (P::Narrow, 50)],
        Style::IndoPersian => &[(P::Grid, 60), (P::Wide, 40)],
        _ => &[(P::Wide, 40), (P::Narrow, 30), (P::Grid, 30)],
    }
}

pub(super) fn mirrors(style: Style) -> &'static [(Mirrors, u64)] {
    match style {
        Style::IndoPersian => &[(Mirrors::Four, 80), (Mirrors::Discs, 20)],
        _ => &[(Mirrors::Discs, 1)],
    }
}

pub(super) fn roll_plates(
    rng: &mut Rng,
    construction: Construction,
    style: Style,
    material: Material,
) -> TorsoBody {
    match construction {
        Construction::Scale => TorsoBody::Scale {
            shape: rng.pick_weighted(scale_shapes(style, material)),
            size: rng.pick_weighted(scale_sizes(style)),
            backing: rng.pick_weighted(backings(style, material)),
            sleeves: one_in(rng, 2),
        },
        Construction::Lamellar => TorsoBody::Lamellar {
            cut: rng.pick_weighted(lamellar_cuts(style)),
            size: rng.pick_weighted(lamellar_sizes(style)),
            lacing: rng.pick_weighted(lacings(style)),
        },
        Construction::Laminar => TorsoBody::Laminar {
            lames: rng.range_u64(*LAMES.start() as u64, *LAMES.end() as u64 + 1) as u8,
            riveted: rng.pick_weighted(laminar_riveting(style)),
        },
        Construction::PlatesInCloth => {
            let size = rng.pick_weighted(cloth_sizes(style));
            let rivets = rng.pick_weighted(rivets(style, size));
            let leather = may_have_leather_cover(style, rivets) && one_in(rng, 5);
            TorsoBody::PlatesInCloth {
                size,
                rivets,
                leather,
            }
        }
        Construction::MailAndPlate => TorsoBody::MailAndPlate {
            rows: rng.pick_weighted(plate_rows(style)),
            sleeves: !one_in(rng, 3),
        },
        Construction::Mirror => TorsoBody::Mirror {
            mirrors: rng.pick_weighted(mirrors(style)),
        },
        Construction::Splint => TorsoBody::Splint {
            rods: material.category() != Category::Metal && one_in(rng, 3),
        },
        _ => TorsoBody::Woven,
    }
}
