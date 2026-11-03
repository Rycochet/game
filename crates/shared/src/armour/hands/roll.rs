//! Rolling a new piece for the hands.

use super::{
    HandsForm, HandsPiece, cloths_for, constructions, cuffs_for, fingers_for, knuckles_for,
    materials_for, may_have_claws, may_have_trim, may_lock, needs_cloth,
};
use crate::armour::{Style, in_style};
use crate::material::Material;
use crate::rng::Rng;

/// Rolls a piece of this kind. A set passes its style; a piece on its own rolls one.
/// Each choice comes from what the ones before allow, so nothing is thrown away.
pub fn roll(rng: &mut Rng, piece: HandsPiece, style: Option<Style>) -> HandsForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    let construction = rng.pick_weighted(&in_style(constructions(piece), style));
    let fingers = rng.pick_weighted(fingers_for(construction, piece, style));
    let cuff = rng.pick_weighted(cuffs_for(construction, fingers, style));
    let knuckles = rng.pick_weighted(knuckles_for(construction));
    let claws = may_have_claws(construction, fingers) && rng.range_u64(0, 12) == 0;
    let locking = may_lock(construction, fingers) && rng.range_u64(0, 8) == 0;
    let material = rng.pick_weighted(materials_for(construction));
    let cloth =
        needs_cloth(construction, fingers).then(|| rng.pick_weighted(cloths_for(construction)));
    let trim = (may_have_trim(construction, material) && rng.range_u64(0, 6) == 0)
        .then_some(Material::Brass);
    HandsForm {
        construction,
        fingers,
        cuff,
        knuckles,
        claws,
        locking,
        material,
        cloth,
        trim,
        style,
    }
}
