//! Rolling a piece of leg armour, each choice from what the ones before allow, so
//! nothing is thrown away.

use super::materials::materials;
use super::tables::{
    Wrap, constructions, covers_for, has_cloth, knees, may_be_one_leg, may_ornament, may_reinforce,
    wrap,
};
use super::{Knee, LegsForm, LegsKind, Ornament, Reinforce};
use crate::armour::arms::Sides;
use crate::armour::{Construction, Dye, Finish, Style};
use crate::item::Quality;
use crate::material::{Category, Material};
use crate::rng::Rng;

/// One in so many.
fn one_in(rng: &mut Rng, chance: u64) -> bool {
    rng.range_u64(0, chance) == 0
}

/// Rolls a piece of leg armour of this kind. A set passes its style; a piece on its
/// own rolls one.
pub fn roll(rng: &mut Rng, kind: LegsKind, style: Option<Style>, quality: Quality) -> LegsForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    let construction = rng.pick_weighted(constructions(kind, style));
    let cover = rng.pick_weighted(covers_for(kind, construction, style));
    let material = rng.pick_weighted(materials(construction, style));
    let closed = match wrap(construction, style, cover) {
        Wrap::Closed => true,
        Wrap::Front => false,
        // Open-backed cuisses, schynbalds and demi-chausses were common; ring greaves
        // and ocreae half and half.
        Wrap::Either if style == Style::Classical => one_in(rng, 2),
        Wrap::Either => !one_in(rng, 3),
    };
    let knee = rng.pick_weighted(knees(kind, construction, style, cover));
    let reinforce = if may_reinforce(construction, style, cover) && !one_in(rng, 3) {
        rng.pick_weighted(&[
            (Reinforce::Poleyns, 40),
            (Reinforce::PoleynsAndSchynbalds, 35),
            (Reinforce::QuiltedThighs, 25),
        ])
    } else {
        Reinforce::None
    };
    // One greave in twenty, mostly on the left leg, the one forward behind a shield.
    let sides = if may_be_one_leg(construction, style, cover) && one_in(rng, 20) {
        rng.pick_weighted(&[(Sides::Left, 3), (Sides::Right, 1)])
    } else {
        Sides::Both
    };
    let ornament = if may_ornament(construction, style) && one_in(rng, 12) {
        let spikes = if matches!(knee, Knee::Cop | Knee::Winged) {
            1
        } else {
            0
        };
        let fins = if cover.shin() { 1 } else { 0 };
        if spikes + fins == 0 {
            Ornament::None
        } else {
            rng.pick_weighted(&[(Ornament::KneeSpikes, spikes), (Ornament::ShinFins, fins)])
        }
    } else {
        Ornament::None
    };
    let studded = construction == Construction::Leather && one_in(rng, 3);
    let finish = roll_finish(rng, construction, style, material, quality);
    let cloth = (has_cloth(construction, style) || reinforce == Reinforce::QuiltedThighs)
        .then(|| Dye::roll(rng));
    let trim = (matches!(construction, Construction::Plate | Construction::Mail)
        && material.category() == Category::Metal
        && material != Material::Bronze
        && style != Style::Japanese
        && one_in(rng, 5))
    .then_some(Material::Brass);
    LegsForm {
        kind,
        construction,
        style,
        cover,
        closed,
        knee,
        reinforce,
        sides,
        ornament,
        studded,
        material,
        finish,
        cloth,
        trim,
    }
}

/// Metal as the style finishes it; mail bright, blackened, tinned, or gilt when fine;
/// hardened leather sometimes lacquered; the rest as it is (as arm pieces are).
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
