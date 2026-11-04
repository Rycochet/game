//! Rolling a pair of footwear, each choice from what the ones before allow, so nothing
//! is thrown away.

use super::finish::{contrast, contrasts, decorations, dyes, tops, unders};
use super::holding::fastenings;
use super::materials::{sole_of, uppers};
use super::tables::{builds, heels, heights, may_be_backless, toes};
use super::{BROAD, Decoration, FootwearForm, FootwearPiece, LONG_POINT, Toe};
use crate::item::Quality;
use crate::rng::Rng;

/// A number from `least` to `most`, both included.
fn between(rng: &mut Rng, (least, most): (u8, u8)) -> u8 {
    rng.range_u64(least as u64, most as u64 + 1) as u8
}

fn between_wide(rng: &mut Rng, (least, most): (u16, u16)) -> u16 {
    rng.range_u64(least as u64, most as u64 + 1) as u16
}

/// Rolls a pair of footwear of this kind. Jewels only come at Rare and better.
pub fn roll(rng: &mut Rng, piece: FootwearPiece, quality: Quality) -> FootwearForm {
    let build = rng.pick_weighted(builds(piece));
    let height = rng.pick_weighted(heights(piece, build));
    let shaft = between_wide(rng, height.shaft());
    let toe = rng.pick_weighted(toes(piece, build, height));
    let point = match toe {
        Toe::LongPoint => between(rng, LONG_POINT),
        _ => 0,
    };
    let width = match toe {
        Toe::Broad => between(rng, BROAD),
        _ => 100,
    };
    let heel = rng.pick_weighted(heels(piece, build, height, toe));
    let heel_mm = between(rng, heel.millimetres());
    let backless = may_be_backless(piece, build) && rng.range_u64(0, 2) == 0;
    let fastening = rng.pick_weighted(fastenings(piece, build, height, toe));
    let top = rng.pick_weighted(&tops(piece, build, height, toe, fastening));
    let upper = rng.pick_weighted(uppers(piece, build, toe, fastening));
    let mut choices = decorations(build, toe, fastening, top, upper);
    if quality < Quality::Rare {
        for (decoration, weight) in &mut choices {
            if *decoration == Decoration::Jewels {
                *weight = 0;
            }
        }
    }
    // Every list keeps something besides jewels, so this is never empty.
    let decoration = rng.pick_weighted(&choices);
    let under = rng.pick_weighted(unders(piece, build, height, toe));
    let dye = rng.pick_weighted(dyes(upper));
    let (may, must) = contrast(piece, fastening, top, decoration, upper);
    let contrast =
        (must || (may && rng.range_u64(0, 6) == 0)).then(|| rng.pick_weighted(&contrasts(dye)));
    FootwearForm {
        piece,
        build,
        height,
        shaft,
        toe,
        point,
        width,
        heel,
        heel_mm,
        backless,
        fastening,
        top,
        decoration,
        under,
        upper,
        sole: sole_of(build, upper),
        dye,
        contrast,
    }
}
