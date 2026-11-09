//! Rolling a piece of underclothing, each choice from what the ones before allow, so
//! nothing is thrown away.

use super::cloth::{cloths, contrasts, decorations, dyes, materials};
use super::{Decoration, Garment, UnderForm, UnderPiece};
use crate::item::Quality;
use crate::rng::Rng;

/// Rolls one of this kind: its garment, material, cloth, decoration and colours. Gold
/// thread only comes at Rare and better.
pub fn roll(rng: &mut Rng, piece: UnderPiece, quality: Quality) -> UnderForm {
    let garment = Garment::roll(rng, piece);
    let material = rng.pick_weighted(&materials(&garment));
    let cloth = rng.pick_weighted(&cloths(&garment, material));
    let mut choices = decorations(&garment, material);
    if quality < Quality::Rare {
        for (decoration, weight) in &mut choices {
            if *decoration == Decoration::GoldThread {
                *weight = 0;
            }
        }
    }
    // Every list keeps Plain, so this is never empty.
    let decoration = rng.pick_weighted(&choices);
    let dye = rng.pick_weighted(&dyes(&garment, material));
    let contrast = two_coloured(&garment).then(|| rng.pick_weighted(&contrasts(dye)));
    UnderForm {
        garment,
        material,
        cloth,
        decoration,
        dye,
        contrast,
    }
}

/// Whether it has a second colour: striped and parti-coloured hose.
pub(super) fn two_coloured(garment: &Garment) -> bool {
    matches!(garment, Garment::Hose(hose) if hose.two_coloured())
}
