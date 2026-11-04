//! Rolling a neck piece, each choice from what the ones before allow, so nothing is
//! thrown away.

use super::materials::materials;
use super::tables::{
    capes, constructions, edgings, fronts, has_cloth, hems, highs, lames, may_spike, may_stud,
    plates, points,
};
use super::{NeckForm, NeckKind};
use crate::armour::{Construction, Dye, Finish, Style};
use crate::item::Quality;
use crate::material::{Category, Material};
use crate::rng::Rng;

/// One in so many.
fn one_in(rng: &mut Rng, chance: u64) -> bool {
    rng.range_u64(0, chance) == 0
}

/// Rolls a neck piece of this kind. A set passes its style; a piece on its own rolls
/// one.
pub fn roll(rng: &mut Rng, kind: NeckKind, style: Option<Style>, quality: Quality) -> NeckForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    let construction = rng.pick_weighted(constructions(kind, style));
    let plates = rng.pick_weighted(plates(kind, construction, style));
    let high = rng.pick_weighted(highs(kind, construction, plates));
    let (least, most) = lames(kind, construction, plates, high);
    let lames = rng.range_u64(least as u64, most as u64 + 1) as u8;
    let pointed = rng.pick_weighted(points(kind, construction, style));
    let front = rng.pick_weighted(fronts(kind, construction, style));
    let cape = rng.pick_weighted(capes(kind, construction));
    let hem = rng.pick_weighted(hems(kind, construction, style));
    let edging = rng.pick_weighted(edgings(construction, style));
    let spikes = may_spike(kind, construction, plates)
        && one_in(rng, if style == Style::Dwarven { 4 } else { 12 });
    let studded = may_stud(kind, construction) && one_in(rng, 3);
    let material = rng.pick_weighted(materials(construction, style));
    let finish = roll_finish(rng, construction, style, material, quality);
    let cloth = has_cloth(construction, style).then(|| Dye::roll(rng));
    let trim = (matches!(kind, NeckKind::Gorget | NeckKind::Bevor)
        && construction == Construction::Plate
        && metal(material)
        && one_in(rng, 5))
    .then_some(Material::Brass);
    NeckForm {
        kind,
        construction,
        style,
        plates,
        high,
        lames,
        pointed,
        front,
        cape,
        hem,
        edging,
        spikes,
        studded,
        material,
        finish,
        cloth,
        trim,
    }
}

/// A metal that takes finishes: bronze is left as it is.
pub(super) fn metal(material: Material) -> bool {
    material.category() == Category::Metal && material != Material::Bronze
}

/// Mail bright, blackened, tinned, or gilt when fine; Japanese lames and lamellae
/// lacquered; other metal as the style finishes it; hardened leather sometimes
/// lacquered; the rest as it is.
fn roll_finish(
    rng: &mut Rng,
    construction: Construction,
    style: Style,
    material: Material,
    quality: Quality,
) -> Finish {
    use Construction as C;
    match construction {
        C::Mail | C::RingMail | C::MailAndPlate if metal(material) => {
            let gilt = if quality >= Quality::Rare { 4 } else { 0 };
            rng.pick_weighted(&[
                (Finish::Bright, 70),
                (Finish::Blackened, 18),
                (Finish::Tinned, 8),
                (Finish::Gilt, gilt),
            ])
        }
        C::Laminar | C::Lamellar if style == Style::Japanese => {
            Finish::Lacquered(Dye::roll_paint(rng))
        }
        _ if metal(material) => Finish::roll(rng, style, quality),
        _ if material == Material::HardenedLeather && one_in(rng, 4) => {
            Finish::Lacquered(Dye::roll_paint(rng))
        }
        _ => Finish::Bright,
    }
}
