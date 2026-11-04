//! Rolling a piece of waist armour, each choice from what the ones before allow, so
//! nothing is thrown away.

use super::counts::{culets, hangs, lames, openings};
use super::extras::{has_cloth, may_add_mail, may_be_front, may_guard_groin};
use super::materials::materials;
use super::tables::{constructions, lengths, pieces};

use super::{WaistForm, WaistKind};
use crate::armour::{Construction, Dye, Finish, Style};
use crate::item::Quality;
use crate::material::{Category, Material};
use crate::rng::Rng;

/// One in so many.
fn one_in(rng: &mut Rng, chance: u64) -> bool {
    rng.range_u64(0, chance) == 0
}

/// Rolls a piece of waist armour of this kind. A set passes its style; a piece on its
/// own rolls one.
pub fn roll(rng: &mut Rng, kind: WaistKind, style: Option<Style>, quality: Quality) -> WaistForm {
    let style = match kind {
        // Cuirassier tassets were 16th- and 17th-century European, or fantasy.
        WaistKind::Cuirassier => style
            .filter(|style| style.allows(Construction::Plate))
            .unwrap_or_else(|| {
                rng.pick_weighted(&[
                    (Style::White, 40),
                    (Style::Greenwich, 25),
                    (Style::Maximilian, 15),
                    (Style::Elven, 10),
                    (Style::Dwarven, 10),
                ])
            }),
        WaistKind::Armour => style.unwrap_or_else(|| Style::roll(rng)),
    };
    let construction = rng.pick_weighted(constructions(kind, style));
    let piece = rng.pick_weighted(pieces(kind, construction, style));
    let length = rng.pick_weighted(lengths(kind, piece, construction, style));
    let lames = rng.pick_weighted(lames(kind, piece, construction, style, length));
    let culet = rng.pick_weighted(culets(piece, construction));
    let hangs = rng.pick_weighted(hangs(kind, piece, style));
    let front = may_be_front(piece, construction, style) && one_in(rng, 3);
    let opening = rng.pick_weighted(openings(piece, construction));
    let mail_beneath = may_add_mail(piece, construction, style) && one_in(rng, 3);
    let groin = may_guard_groin(piece, construction, style) && one_in(rng, 6);
    let material = rng.pick_weighted(materials(construction, style));
    let studded =
        construction == Construction::Leather && style != Style::Japanese && one_in(rng, 3);
    let finish = roll_finish(rng, construction, style, material, quality);
    let cloth = has_cloth(piece, construction, style).then(|| Dye::roll(rng));
    let trim = (matches!(construction, Construction::Plate | Construction::Mail)
        && material.category() == Category::Metal
        && material != Material::Bronze
        && style != Style::Japanese
        && one_in(rng, 5))
    .then_some(Material::Brass);
    WaistForm {
        kind,
        construction,
        style,
        piece,
        length,
        lames,
        culet,
        hangs,
        front,
        opening,
        mail_beneath,
        groin,
        studded,
        material,
        finish,
        cloth,
        trim,
    }
}

/// Metal as the style finishes it; mail bright, blackened, tinned, or gilt when fine;
/// hardened leather sometimes lacquered, always in Japan; the rest as it is.
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
        _ if material == Material::HardenedLeather
            && (style == Style::Japanese || one_in(rng, 4)) =>
        {
            Finish::Lacquered(Dye::roll_paint(rng))
        }
        _ => Finish::Bright,
    }
}
