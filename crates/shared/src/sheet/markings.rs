//! Markings: a second colour over parts of an animal's coat, chosen when it is made as
//! its other looks are (`Trait::Markings`), so everyone sees the same animal. Kept
//! simple (owner, 2026-11-12): a paler belly, a face mask, a blaze or a white face,
//! and patches laid out by a number. Saved by place in the list, so new ones go at the
//! end; the first is what a creature has if nothing was chosen.

use crate::creature::CreatureKind;

use super::looks::word;
use super::{Choice, Pattern};

/// Where on the body a second colour is, and the colour: the client draws it by
/// region of the body (`item_art`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Marks {
    /// Red, green and blue, 0 to 255.
    pub colour: [u8; 3],
    /// Paler underneath: the belly, the throat and under the jaw.
    pub belly: bool,
    pub face: Face,
    /// Patches over the trunk, neck, head and the tops of the legs, laid out by this
    /// number, so animals with the same markings have different patches.
    pub patches: Option<u8>,
}

/// What of the face is marked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    Bare,
    /// The cheeks, the muzzle's sides and under the jaw, as a wolf's.
    Mask,
    /// A stripe down the front of the face, as a horse's.
    Blaze,
    /// The whole face, as a Hereford's.
    Whole,
}

/// Off-white, as a wolf's or a horse's markings.
const CREAM: [u8; 3] = [226, 218, 200];
const WHITE: [u8; 3] = [236, 234, 228];
const DARK: [u8; 3] = [40, 34, 32];

const fn marked(name: &'static str, marks: Marks) -> Choice {
    Choice {
        name,
        colour: None,
        pattern: Pattern::Marked(marks),
    }
}

const fn marks(colour: [u8; 3], belly: bool, face: Face) -> Marks {
    Marks {
        colour,
        belly,
        face,
        patches: None,
    }
}

const fn patched(name: &'static str, colour: [u8; 3], layout: u8) -> Choice {
    marked(
        name,
        Marks {
            colour,
            belly: false,
            face: Face::Bare,
            patches: Some(layout),
        },
    )
}

const WOLF: &[Choice] = &[
    marked("Pale belly and mask", marks(CREAM, true, Face::Mask)),
    marked("Pale belly", marks(CREAM, true, Face::Bare)),
];

/// Mostly patched (black and white, red and white), as farm cattle are.
const CATTLE: &[Choice] = &[
    word("Plain"),
    marked("White face", marks(WHITE, true, Face::Whole)),
    patched("White patches", WHITE, 0),
    patched("White patches", WHITE, 1),
    patched("White patches", WHITE, 2),
    patched("White patches", WHITE, 3),
    patched("White patches", WHITE, 4),
    patched("White patches", WHITE, 5),
    patched("Dark patches", DARK, 6),
    patched("Dark patches", DARK, 7),
];

const HORSE: &[Choice] = &[
    word("Plain"),
    marked("Blaze", marks(WHITE, false, Face::Blaze)),
    marked("Blaze", marks(WHITE, false, Face::Blaze)),
    patched("Pinto", WHITE, 8),
    patched("Pinto", WHITE, 9),
];

/// A minotaur's, to pick when making one.
const MINOTAUR: &[Choice] = &[
    word("Plain"),
    marked("White face", marks(WHITE, false, Face::Whole)),
    patched("Patched", WHITE, 10),
];

/// The markings a creature of this kind may have; none for a kind without.
pub(super) fn markings_choices(kind: CreatureKind) -> &'static [Choice] {
    match kind {
        CreatureKind::Wolf => WOLF,
        CreatureKind::Cattle => CATTLE,
        CreatureKind::Horse => HORSE,
        CreatureKind::Minotaur => MINOTAUR,
        _ => &[],
    }
}
