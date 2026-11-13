//! One worn thing built in place on a frame, coloured as its own picture is: what a
//! character's picture is made of.

use shared::body::Frame;

use super::set::Coloured;
use super::{Model, Stuff, arms, feet, hands, head, legs, neck, set, shoulders, torso, waist};
use crate::item_art::{Look, footwear, under};

/// A worn thing's parts on this frame, for the looks that are worn on a body; none
/// for others. An aventail hangs from `rim`, the helmet worn with it, if any.
pub(in crate::item_art) fn worn(look: &Look, frame: &Frame, rim: Option<&[[i32; 3]]>) -> Coloured {
    let mut all = Coloured::new();
    let mut add = |model: Model, colour: &dyn Fn(Stuff) -> bevy::prelude::StandardMaterial| {
        for (stuff, triangles) in model.parts {
            all.push((triangles, colour(stuff)));
        }
    };
    match look {
        Look::Hands { hands: l, quality } => {
            add(hands::build_pair(&l.form, frame, l.digits), &|s| {
                hands::material_of(s, &l.form, *quality)
            })
        }
        Look::Shoulders {
            shoulders: l,
            quality,
        } => add(shoulders::build_pair(&l.form, frame), &|s| {
            shoulders::material_of(s, &l.form, *quality)
        }),
        Look::Torso { torso: l, quality } => add(torso::build(&l.form, frame), &|s| {
            torso::material_of(s, &l.form, *quality)
        }),
        Look::Head { head: l, quality } => add(head::build(&l.form, frame), &|s| {
            head::material_of(s, &l.form, *quality)
        }),
        Look::Arms { arms: l, quality } => add(arms::build(&l.form, frame), &|s| {
            arms::material_of(s, &l.form, *quality)
        }),
        Look::Legs { legs: l, quality } => add(legs::build(&l.form, frame), &|s| {
            legs::material_of(s, &l.form, *quality)
        }),
        Look::Waist { waist: l, quality } => add(waist::build(&l.form, frame), &|s| {
            waist::material_of(s, &l.form, *quality)
        }),
        Look::Feet { feet: l, quality } => add(feet::build(&l.form, frame), &|s| {
            feet::material_of(s, &l.form, *quality)
        }),
        Look::Neck { neck: l, quality } => {
            let model = match rim {
                Some(rim) => neck::build_under(&l.form, frame, rim),
                None => neck::build(&l.form, frame),
            };
            add(model, &|s| neck::material_of(s, &l.form, *quality));
        }
        Look::Footwear {
            footwear: l,
            quality,
        } => add(footwear::build(&l.form, frame), &|s| {
            footwear::material_of(s, &l.form, *quality)
        }),
        Look::Under { under: l, quality } => {
            add(under::build(&l.form, frame, under::Fit::Tucked), &|s| {
                under::material_of(s, &l.form, *quality)
            })
        }
        Look::Set { set: l, quality } => return set::build(&l.form, frame, l.digits, *quality),
        _ => {}
    }
    all
}

#[cfg(test)]
mod tests;
