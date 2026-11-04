//! Rolling a pair of foot pieces, each choice from what the ones before allow, so
//! nothing is thrown away.

use super::materials::materials;
use super::tables::{constructions, has_cloth, lames, may_be_high, may_detach, pieces, toes};
use super::{DETACHABLE_POINT_MM, FeetForm, FeetPiece, POINT_MM, Toe};
use crate::armour::{Construction, Dye, Finish, Style};
use crate::item::Quality;
use crate::material::{Category, Material};
use crate::rng::Rng;

/// One in so many.
fn one_in(rng: &mut Rng, chance: u64) -> bool {
    rng.range_u64(0, chance) == 0
}

/// A number from `least` to `most`, both included.
fn between(rng: &mut Rng, (least, most): (u64, u64)) -> u64 {
    rng.range_u64(least, most + 1)
}

/// Rolls a pair of foot pieces. A set passes its style; a piece on its own rolls one.
pub fn roll(rng: &mut Rng, style: Option<Style>, quality: Quality) -> FeetForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    let construction = rng.pick_weighted(constructions(style));
    let piece = rng.pick_weighted(pieces(construction, style));
    let toe = rng.pick_weighted(&toes(piece, construction, style));
    let detachable = may_detach(construction, style, toe) && one_in(rng, 4);
    let point = match toe {
        Toe::Pointed if detachable => between(rng, widen(DETACHABLE_POINT_MM)),
        // Elven points are short, curving up.
        Toe::Pointed if style == Style::Elven => between(rng, (30, 60)),
        Toe::Pointed => between(rng, widen(POINT_MM)),
        _ => 0,
    } as u16;
    let (narrow, wide) = toe.widths();
    let width = between(rng, (narrow as u64, wide as u64)) as u8;
    let lames = if piece == FeetPiece::Sabatons {
        let (least, most) = lames(construction, style, toe);
        between(rng, (least as u64, most as u64)) as u8
    } else {
        0
    };
    let sabatons = piece == FeetPiece::Sabatons;
    let heel = sabatons
        && match construction {
            Construction::Plate | Construction::Leather => one_in(rng, 2),
            _ => one_in(rng, 3),
        };
    let high = may_be_high(piece, construction, toe)
        && one_in(rng, if style == Style::Dwarven { 2 } else { 5 });
    let fluted = sabatons && construction == Construction::Plate && style == Style::Maximilian;
    let toes = match piece {
        FeetPiece::FootGuards if style == Style::Classical => true,
        FeetPiece::FootGuards if style == Style::Japanese => !one_in(rng, 3),
        FeetPiece::FootGuards => one_in(rng, 2),
        _ => false,
    };
    let soled =
        piece == FeetPiece::MailShoes && (construction == Construction::RingMail || one_in(rng, 3));
    let studded = construction == Construction::Leather && one_in(rng, 3);
    let material = rng.pick_weighted(materials(construction, style));
    let finish = roll_finish(rng, construction, style, material, quality);
    let cloth = has_cloth(style).then(|| Dye::roll(rng));
    let trim = (sabatons
        && construction == Construction::Plate
        && material.category() == Category::Metal
        && material != Material::Bronze
        && one_in(rng, 5))
    .then_some(Material::Brass);
    FeetForm {
        piece,
        construction,
        style,
        toe,
        point,
        detachable,
        width,
        lames,
        heel,
        high,
        fluted,
        toes,
        soled,
        studded,
        material,
        finish,
        cloth,
        trim,
    }
}

fn widen((least, most): (u16, u16)) -> (u64, u64) {
    (least as u64, most as u64)
}

/// Metal as the style finishes it; mail bright, blackened, tinned, or gilt when fine;
/// hardened leather sometimes lacquered; the rest as it is (as leg pieces are).
fn roll_finish(
    rng: &mut Rng,
    construction: Construction,
    style: Style,
    material: Material,
    quality: Quality,
) -> Finish {
    let metal = material.category() == Category::Metal && material != Material::Bronze;
    match construction {
        Construction::Mail | Construction::RingMail if metal => {
            let gilt = if quality >= Quality::Rare { 4 } else { 0 };
            rng.pick_weighted(&[
                (Finish::Bright, 70),
                (Finish::Blackened, 18),
                (Finish::Tinned, 8),
                (Finish::Gilt, gilt),
            ])
        }
        _ if metal => Finish::roll(rng, style, quality),
        _ if material == Material::HardenedLeather && one_in(rng, 4) => {
            Finish::Lacquered(Dye::roll_paint(rng))
        }
        _ => Finish::Bright,
    }
}
