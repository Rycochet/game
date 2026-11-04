//! What armour of many small plates rolls: scales, lamellae, lames, plates in cloth,
//! plates on mail, mirrors and slats.

use serde::{Deserialize, Serialize};

/// How big each scale, lamella or plate is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlateSize {
    /// About 2 cm: Roman scales, Japanese kozane, a jack's squares.
    Small,
    Medium,
    /// A hand across: a coat of plates' plates.
    Large,
}

impl PlateSize {
    /// About how tall each is, in millimetres.
    pub fn height(self) -> i32 {
        match self {
            Self::Small => 25,
            Self::Medium => 45,
            Self::Large => 80,
        }
    }
}

/// How a scale is cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScaleShape {
    Rounded,
    Pointed,
    Square,
    /// Leaf-shaped: Elven.
    Leaf,
    /// Coins sewn on in rows, as Tlingit coin armour.
    Coin,
}

/// What scales are fixed to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Backing {
    Cloth,
    Leather,
    /// Scales on mail: the Roman lorica plumata.
    Mail,
}

/// How a lamellar cuirass is cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LamellarCut {
    /// Round the body, closing at the side: dō-maru, klivanion.
    Wrap,
    /// Closing at the back: haramaki.
    OpenBack,
    /// Boxy, with a separate plate for the right side: ō-yoroi.
    Boxed,
    /// Small plates interlocking in a Y: Chinese mountain pattern.
    Mountain,
}

/// How closely lamellae are laced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lacing {
    /// In pairs, the plates showing between (sugake).
    Spaced,
    /// Lacing over the whole surface (kebiki).
    Full,
}

/// Rivet heads on plates in cloth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rivets {
    /// Plates stitched in, a grid of stitching showing: a jack.
    Hidden,
    Iron,
    Tinned,
    Gilt,
}

/// How the plates of plated mail are laid out, front and back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlateRows {
    /// A few wide plates in rows: yushman.
    Wide,
    /// Many narrow overlapping plates in columns: bakhterets.
    Narrow,
    /// Rectangles in a grid: zirah bagtar.
    Grid,
}

/// The polished plates of mirror armour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mirrors {
    /// A round plate on the breast and one on the back.
    Discs,
    /// Four plates, breast, back and both sides: char-aina.
    Four,
}
