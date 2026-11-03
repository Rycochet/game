//! Rolling a piece of arm armour, each choice from what the ones before allow, so
//! nothing is thrown away.

use super::tables::{
    always_closed, constructions, has_closure, has_couter, has_cover, materials, may_be_one_arm,
    reaches,
};
use super::{ArmsForm, Couter, Sides, Spikes};
use crate::armour::{Construction, Dye, Finish, Style};
use crate::item::Quality;
use crate::material::{Category, Material};
use crate::rng::Rng;

/// One in so many.
fn one_in(rng: &mut Rng, chance: u64) -> bool {
    rng.range_u64(0, chance) == 0
}

/// Rolls a piece of arm armour. A set passes its style; a piece on its own rolls one.
pub fn roll(rng: &mut Rng, style: Option<Style>, quality: Quality) -> ArmsForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    let construction = rng.pick_weighted(constructions(style));
    let reach = rng.pick_weighted(reaches(construction, style));
    let material = rng.pick_weighted(materials(construction, style, reach));
    let sides = if may_be_one_arm(construction, style, reach) && one_in(rng, 3) {
        // The bow arm for a bracer, the sword arm for a manica, as most were handed.
        let usual = if construction == Construction::Leather {
            Sides::Left
        } else {
            Sides::Right
        };
        let other = if usual == Sides::Left {
            Sides::Right
        } else {
            Sides::Left
        };
        rng.pick_weighted(&[(usual, 3), (other, 1)])
    } else {
        Sides::Both
    };
    let couter = if has_couter(construction, style, reach) {
        let winged = match (construction, style) {
            (Construction::Plate, Style::Gothic | Style::Milanese) => 80,
            (Construction::Plate, _) => 50,
            _ => 0,
        };
        rng.pick_weighted(&[(Couter::Winged, winged), (Couter::Cup, 100 - winged)])
    } else {
        Couter::None
    };
    let closed = has_closure(construction, reach) && (always_closed(style) || !one_in(rng, 4));
    let spikes =
        if matches!(construction, Construction::Plate | Construction::Leather) && one_in(rng, 10) {
            let elbow = if couter == Couter::None { 0 } else { 1 };
            let ridge = if reach.forearm() { 1 } else { 0 };
            if elbow + ridge == 0 {
                Spikes::None
            } else {
                rng.pick_weighted(&[(Spikes::Elbow, elbow), (Spikes::Ridge, ridge)])
            }
        } else {
            Spikes::None
        };
    let studded = construction == Construction::Leather && one_in(rng, 3);
    let finish = roll_finish(rng, construction, style, material, quality);
    let cover = has_cover(construction, style).then(|| Dye::roll(rng));
    let trim = (matches!(construction, Construction::Plate | Construction::Mail)
        && material.category() == Category::Metal
        && material != Material::Bronze
        && one_in(rng, 5))
    .then_some(Material::Brass);
    ArmsForm {
        construction,
        style,
        reach,
        sides,
        couter,
        closed,
        spikes,
        studded,
        material,
        finish,
        cover,
        trim,
    }
}

/// Metal as the style finishes it; mail bright, blackened, tinned, or gilt when fine;
/// hardened leather sometimes lacquered; everything else as it is.
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
