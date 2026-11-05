//! What a body has beyond a person's: snouts, ears, horns, tails, the kind of feet and
//! wings. Shapes are chosen from a few kinds; sizes are numbers, in millimetres at a
//! person's scale as everything on a frame is, so one body can later turn into another
//! by changing the numbers.

/// The front of the face.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Snout {
    /// A person's face.
    Flat,
    /// Rounded and blunt, as a bull's or a dog's.
    Muzzle,
    /// Long and tapering, as a lizard's.
    Long,
    /// A bird's, hard and pointed.
    Beak,
}

/// Horns on the head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Horns {
    None,
    /// Short and swept back, as a goat's or a kobold's.
    Back,
    /// Out to the sides and curving up, as a bull's.
    Bull,
}

/// What a body stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Feet {
    /// Heels down, as a person's.
    Plantigrade,
    /// On its toes with the heel raised, as a dog's or a lizard's.
    Digitigrade,
    Hooves,
    /// A bird's toes, three forward and one back.
    Talons,
}

/// Wings, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Wings {
    None,
    /// The arms are wings: feathers along them, hands at their tips.
    Arms,
}

/// Everything a body has beyond a person's. Lengths are how far each reaches out from
/// the body, 0 for none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Parts {
    pub snout: Snout,
    pub snout_length: i32,
    /// Pointed ears reaching this far beyond a person's.
    pub ears: i32,
    pub horns: Horns,
    pub horn_length: i32,
    /// Tusks up from the lower jaw.
    pub tusks: i32,
    /// A tail this long, from the base of the back.
    pub tail: i32,
    pub feet: Feet,
    pub wings: Wings,
}

/// A person: nothing more.
pub const NONE: Parts = Parts {
    snout: Snout::Flat,
    snout_length: 0,
    ears: 0,
    horns: Horns::None,
    horn_length: 0,
    tusks: 0,
    tail: 0,
    feet: Feet::Plantigrade,
    wings: Wings::None,
};
