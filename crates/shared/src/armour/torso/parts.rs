//! What each construction of torso armour rolls about itself.

use serde::{Deserialize, Serialize};

/// The shape of a breastplate, or of a leather cuirass: mostly the style's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Breast {
    /// Smooth and rounded.
    Round,
    /// Plain, flatter, ridged down the middle: 17th-century soldiers'.
    Munition,
    /// Almost round below the chest: Milanese.
    Globose,
    /// Waisted, a pointed plackart and ridges fanning up.
    Gothic,
    /// Flat and sloping, breaking forward at the chest: Kastenbrust.
    Box,
    /// A ridge running down to a point over the belt.
    Peascod,
    /// Chest and belly shaped in the metal.
    Muscle,
    /// A plain shell flaring out at the bottom.
    Bell,
    /// Three discs on breast and back, joined by straps: Samnite.
    Discs,
    /// Iron bands riveted into a closed shell: Kofun Japan.
    Tanko,
    /// Thin, with leaf lines rising from a pointed hem.
    Leaf,
    /// Broad and heavy, in flat angled panels with bands across.
    Panel,
}

/// A lower plate over the breast's belly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Plackart {
    None,
    /// Up to the middle of the belly, rounded.
    Half,
    /// A point up to the chest, as on Gothic breasts.
    Pointed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Fluting {
    None,
    /// Ridges fanning up from the waist: Gothic.
    Ridges,
    /// Parallel flutes all over: Maximilian.
    All,
}

/// How mail's rings are closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Links {
    /// Ends only pressed together: cheapest and weakest.
    Butted,
    Riveted,
    /// Rows of riveted rings and solid punched ones.
    RivetedSolid,
    Welded,
}

/// How mail's rings go through each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weave {
    /// Each ring through four others: the usual.
    FourInOne,
    /// Each through six: twice as dense and heavy.
    SixInOne,
    /// Square, light, sewn on cloth: Japanese kusari.
    Japanese,
}

/// How big mail's rings are across the outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rings {
    Fine,
    Medium,
    Coarse,
}

impl Rings {
    /// Outside diameter in millimetres.
    pub fn outside(self) -> i32 {
        match self {
            Self::Fine => 6,
            Self::Medium => 9,
            Self::Coarse => 12,
        }
    }
}

/// How padding is stitched.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quilting {
    /// Upright channels.
    Channels,
    Diamonds,
    Squares,
    /// Layers glued together, not stitched: linothorax.
    Laminated,
}
