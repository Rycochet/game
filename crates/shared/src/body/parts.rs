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

/// The shape of the ears, however long they are (`Parts::ears`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ears {
    /// Pointed, swept out and up beside the head, as an elf's.
    Swept,
    /// Pointed and upright on top of the head, as a wolf's or a horse's.
    Upright,
    /// Broad, out to the sides and a little down, under any horns, as a cow's.
    Sideways,
    /// Hanging down beside the head, as a hound's.
    Dropped,
}

/// The shape of a tail, however long it is (`Parts::tail`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tail {
    /// Tapering evenly, as a horse's dock or a lizard's.
    Plain,
    /// Thick with fur its whole length and hanging, as a wolf's.
    Bushy,
    /// Thin its whole length, as a cow's (its switch is tail hair).
    Thin,
    /// Thin, with a tuft at its end, as a lion's.
    Tufted,
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
    /// A bird's wings, folded along its sides.
    Folded,
}

/// Everything a body has beyond a person's. Lengths are how far each reaches out from
/// the body, 0 for none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Parts {
    pub snout: Snout,
    pub snout_length: i32,
    /// Ears reaching this far beyond a person's, in this shape.
    pub ears: i32,
    pub ear_shape: Ears,
    pub horns: Horns,
    pub horn_length: i32,
    /// Tusks up from the lower jaw.
    pub tusks: i32,
    /// A tail this long, from the base of the back (or the end of the abdomen).
    pub tail: i32,
    pub tail_shape: Tail,
    /// Whether the tail curls up over the back to a sting, as a scorpion's.
    pub sting: bool,
    /// A separate rear body this long behind the hips, as a spider's or a scorpion's.
    pub abdomen: i32,
    pub feet: Feet,
    pub wings: Wings,
    /// A mane along the top of the neck, hanging this far down its sides, as a
    /// horse's.
    pub mane: i32,
    /// Long hair hanging this far from the tail: a horse's whole tail, a cow's switch.
    pub tail_hair: i32,
    /// A fold of skin hanging this far below the throat, as a cow's.
    pub dewlap: i32,
    /// An udder this big under the belly, between the hind legs.
    pub udder: i32,
    /// A comb this tall on top of the head and wattles under the beak, as a hen's.
    pub comb: i32,
}

/// A person: nothing more.
pub const NONE: Parts = Parts {
    snout: Snout::Flat,
    snout_length: 0,
    ears: 0,
    ear_shape: Ears::Swept,
    horns: Horns::None,
    horn_length: 0,
    tusks: 0,
    tail: 0,
    tail_shape: Tail::Plain,
    sting: false,
    abdomen: 0,
    feet: Feet::Plantigrade,
    wings: Wings::None,
    mane: 0,
    tail_hair: 0,
    dewlap: 0,
    udder: 0,
    comb: 0,
};
