//! Rolling new shoulder armour.

use super::{
    AiletteShape, CapeCut, Cut, GuardShape, HautePiece, Heraldry, Lacing, Length, ShouldersForm,
    ShouldersKind, ShouldersPiece, Span, Wings, adornments, cloths, constructions_in, finishes,
    kinds_for, laced, materials, may_have_trim, needs_cloth, pauldron_cut,
};
use crate::armour::{Construction, Style};
use crate::material::Material;
use crate::rng::Rng;

pub(super) fn roll_piece(
    rng: &mut Rng,
    kind: ShouldersKind,
    construction: Construction,
    style: Style,
) -> ShouldersPiece {
    use Construction as C;
    let chance = |rng: &mut Rng, one_in: u64| rng.range_u64(0, one_in) == 0;
    match kind {
        ShouldersKind::Pauldrons => {
            let plate = construction == C::Plate;
            let hanging = !matches!(construction, C::Plate | C::Leather);
            let cut = if plate {
                rng.pick_weighted(pauldron_cut(style))
            } else {
                Cut::Matched
            };
            let wings = if hanging {
                Wings::Small
            } else if style == Style::Milanese || cut == Cut::GreatLeft {
                Wings::Large
            } else {
                rng.pick_weighted(&[(Wings::Small, 60), (Wings::Large, 40)])
            };
            let lames = if hanging {
                0
            } else {
                rng.range_u64(3, 7) as u8
            };
            let upper_lames = if plate { rng.range_u64(0, 3) as u8 } else { 0 };
            let haute_piece = if plate {
                rng.pick_weighted(&[
                    (HautePiece::None, 60),
                    (HautePiece::Low, 25),
                    (HautePiece::High, 15),
                ])
            } else {
                HautePiece::None
            };
            let gardbrace = plate && chance(rng, 5);
            let besagews = wings == Wings::Small && !hanging && chance(rng, 3);
            ShouldersPiece::Pauldrons {
                cut,
                wings,
                lames,
                upper_lames,
                haute_piece,
                gardbrace,
                besagews,
            }
        }
        ShouldersKind::Spaulders => {
            let dendra = construction == C::Plate && style == Style::Classical;
            let lames = if dendra { 0 } else { rng.range_u64(2, 6) as u8 };
            let besagews = matches!(construction, C::Plate | C::Leather)
                && !dendra
                && (style == Style::Gothic || style == Style::Kastenbrust || chance(rng, 3));
            let pteruges = if matches!(construction, C::Laminar | C::Leather) {
                rng.pick_weighted(&[(0, 50), (1, 35), (2, 15)])
            } else {
                0
            };
            ShouldersPiece::Spaulders {
                lames,
                besagews,
                pteruges,
            }
        }
        ShouldersKind::Guards => {
            let size =
                rng.pick_weighted(&[(Span::Small, 30), (Span::Medium, 40), (Span::Large, 30)]);
            let (fewest, most) = size.rows();
            let rows = rng.range_u64(fewest as u64, most as u64 + 1) as u8;
            let shape = rng.pick_weighted(&[
                (GuardShape::Straight, 60),
                (GuardShape::Tapered, 25),
                (GuardShape::Flared, 15),
            ]);
            let lacing = if laced(construction) {
                rng.pick_weighted(&[(Lacing::Dense, 70), (Lacing::Spaced, 30)])
            } else {
                Lacing::Dense
            };
            ShouldersPiece::Guards {
                size,
                shape,
                rows,
                lacing,
            }
        }
        ShouldersKind::Cape => ShouldersPiece::Cape {
            cut: match construction {
                C::Padded => CapeCut::Yoke,
                C::Leather if style == Style::Classical => {
                    rng.pick_weighted(&[(CapeCut::Round, 40), (CapeCut::Yoke, 60)])
                }
                _ => CapeCut::Round,
            },
            length: rng.pick_weighted(&[(Length::Short, 45), (Length::Long, 55)]),
        },
        ShouldersKind::Discs => ShouldersPiece::Discs {
            diameter: 110 + 10 * rng.range_u64(0, 6) as u16,
            boss: chance(rng, 2),
        },
        ShouldersKind::Ailettes => ShouldersPiece::Ailettes {
            shape: rng.pick_weighted(&[
                (AiletteShape::Rectangle, 55),
                (AiletteShape::Rounded, 15),
                (AiletteShape::Shield, 20),
                (AiletteShape::Lozenge, 10),
            ]),
            size: rng.pick_weighted(&[(Span::Small, 30), (Span::Medium, 45), (Span::Large, 25)]),
            heraldry: Heraldry::roll(rng),
        },
        ShouldersKind::Wing => ShouldersPiece::Wing,
    }
}

/// Rolls a pair of a kind, or any kind. A set passes its style; a pair on its own
/// rolls one. Each choice comes from what the ones before allow, so nothing is thrown
/// away.
pub fn roll(rng: &mut Rng, kind: Option<ShouldersKind>, style: Option<Style>) -> ShouldersForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    let kind = kind.unwrap_or_else(|| rng.pick_weighted(&kinds_for(style)));
    let construction = rng.pick_weighted(&constructions_in(kind, style));
    let material = rng.pick_weighted(materials(kind, construction, style));
    let piece = roll_piece(rng, kind, construction, style);
    let finish = rng.pick_weighted(finishes(kind, construction, &piece));
    let adornment = rng.pick_weighted(adornments(kind, construction));
    let cloth = needs_cloth(construction).then(|| rng.pick_weighted(cloths(construction)));
    let trim = (may_have_trim(kind, construction, material) && rng.range_u64(0, 5) == 0)
        .then_some(Material::Brass);
    ShouldersForm {
        piece,
        construction,
        adornment,
        finish,
        material,
        cloth,
        trim,
        style,
    }
}
