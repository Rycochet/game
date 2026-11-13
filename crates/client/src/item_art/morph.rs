//! A body changing into another, for a token while a creature changes its kind: a few
//! pictures of the blend between its old body and its new (`shared::body::Layout::
//! blend`), bare, each filling the token as the bodies at either end do.

use shared::body::{BLEND_WHOLE, Hair, Layout};
use shared::creature::CreatureKind;
use shared::sheet::Appearance;

use super::{Colours, Look};

/// How many steps a change of kind takes: pictures 1 to `MORPH_STEPS - 1` are between
/// the two bodies, 0 and `MORPH_STEPS` are the bodies themselves.
pub const MORPH_STEPS: u8 = 8;

/// One step of a body changing into another: both bodies at their real sizes, so the
/// blend between them is in proportion, how each looks, and how far along it is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MorphLook {
    pub(super) from: Layout,
    pub(super) to: Layout,
    pub(super) colours: [Colours; 2],
    pub(super) hair: [Hair; 2],
    pub(super) step: u8,
}

impl MorphLook {
    /// How far along the blend it is, in `BLEND_WHOLE`s.
    pub(super) fn along(&self) -> i32 {
        i32::from(self.step.min(MORPH_STEPS)) * BLEND_WHOLE / i32::from(MORPH_STEPS)
    }
}

/// A creature of kind `from`, looking like this, `step` of `MORPH_STEPS` of the way to
/// being one of kind `to`: a square picture of its bare body, as a token shows it. None
/// if either kind has no body drawn.
pub fn morph_look(
    from: (CreatureKind, &Appearance),
    to: (CreatureKind, &Appearance),
    step: u8,
) -> Option<Look> {
    let side = |(kind, appearance): (CreatureKind, &Appearance)| {
        Some((
            Layout::at_size(kind)?,
            Colours::of(kind, appearance),
            appearance.hair(kind),
        ))
    };
    let (from, from_colours, from_hair) = side(from)?;
    let (to, to_colours, to_hair) = side(to)?;
    Some(Look::Morph(Box::new(MorphLook {
        from,
        to,
        colours: [from_colours, to_colours],
        hair: [from_hair, to_hair],
        step,
    })))
}

/// Colours `t` thousandths of the way from one to the other.
pub(super) fn blend_colours(a: &Colours, b: &Colours, t: i32) -> Colours {
    let mix = |a: [u8; 3], b: [u8; 3]| {
        [0, 1, 2].map(|i| {
            let (a, b) = (i32::from(a[i]), i32::from(b[i]));
            (a + (b - a) * t / BLEND_WHOLE) as u8
        })
    };
    Colours {
        skin: mix(a.skin, b.skin),
        coat: mix(a.coat, b.coat),
        feathers: mix(a.feathers, b.feathers),
        hair: mix(a.hair, b.hair),
        mane: mix(a.mane, b.mane),
        // Markings change at halfway, as hair does.
        marks: if t < BLEND_WHOLE / 2 {
            a.marks
        } else {
            b.marks
        },
    }
}

#[cfg(test)]
mod tests;
