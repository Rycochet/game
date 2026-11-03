//! The parts a blade is described by: build, blade, hilt and decoration.

use serde::{Deserialize, Serialize};

/// How the blade meets the handle, which decides how its length is measured and which
/// size grid it uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Build {
    /// A blade carrying on along the handle's line, measured from where they meet.
    Along,
    /// A head on the side of the handle, measured along its cutting edge.
    Side,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BladeWidth {
    Narrow,
    Normal,
    Broad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Edges {
    PointOnly,
    Single,
    Double,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Curve {
    Straight,
    Slight,
    Strong,
    /// Bent forward, towards the edge, as on a kukri.
    Inward,
    Wavy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Point {
    Sharp,
    Rounded,
    None,
    Hooked,
    Clip,
    Notched,
    Flared,
}

/// A blade's outline, seen from the side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BladeShape {
    Parallel,
    Tapered,
    Leaf,
    Widening,
}

/// A blade's shape, cut across.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Section {
    Flat,
    Lenticular,
    Diamond,
    Midrib,
    Triangular,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hands {
    One,
    OneOrTwo,
    Two,
}

/// Follows from the build and handle, so it is worked out rather than rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Attachment {
    Tang,
    Socket,
    SideMounted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Guard {
    None,
    Cross,
    Disc,
    Cup,
    KnuckleBow,
    Basket,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pommel {
    None,
    Plain,
    Ring,
    Wheel,
    Disc,
    Hooked,
}

/// The unsharpened part of a blade above the guard.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ricasso {
    None,
    Plain,
    WithHooks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandleShape {
    Straight,
    Curved,
    SCurved,
    Angled,
}

/// What sticks out on one side of the handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SidePart {
    None,
    AxeBlade,
    Hook,
    Spike,
    Beak,
    Lug,
}

/// What is on the very end of the handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Top {
    None,
    Blade,
    Spike,
}

/// What is behind an axe's head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Back {
    None,
    Spike,
    Hammer,
    Pipe,
}

/// How a spear's point divides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tips {
    Single,
    Barbed,
    TwoTines,
    ThreeTines,
}

/// How much has been done to make it look good.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Decoration {
    Plain,
    Engraved,
    Inlaid,
    Tasselled,
}
