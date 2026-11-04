//! What each construction of torso armour rolls of its own.

use serde::{Deserialize, Serialize};

use super::parts::{Breast, Fluting, Links, Plackart, Quilting, Rings, Weave};
use super::plates::{
    Backing, Lacing, LamellarCut, Mirrors, PlateRows, PlateSize, Rivets, ScaleShape,
};

/// Everything a construction rolls of its own. Saved by name, so new ones go at the end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TorsoBody {
    /// Shaped solid plates.
    Plate {
        breast: Breast,
        plackart: Plackart,
        /// A ridge down the middle.
        ridge: bool,
        fluting: Fluting,
        /// A hook on the right breast for a couched lance.
        lance_rest: bool,
    },
    /// Hardened, buff or raw leather; studded has rows of rivet heads.
    Leather {
        breast: Breast,
        studded: bool,
    },
    Mail {
        links: Links,
        weave: Weave,
        rings: Rings,
        /// Short sleeves, to the middle of the upper arm: still not Arms.
        sleeves: bool,
    },
    /// Rings sewn flat on leather.
    RingMail {
        sleeves: bool,
    },
    Padded {
        quilting: Quilting,
        sleeves: bool,
    },
    /// Small plates on a backing, each row over the one below.
    Scale {
        shape: ScaleShape,
        size: PlateSize,
        backing: Backing,
        sleeves: bool,
    },
    /// Small plates laced to each other in rows.
    Lamellar {
        cut: LamellarCut,
        size: PlateSize,
        lacing: Lacing,
    },
    /// Hoops round the body: riveted to leathers, or laced.
    Laminar {
        lames: u8,
        riveted: bool,
    },
    /// Plates riveted under a cover of cloth or leather.
    PlatesInCloth {
        size: PlateSize,
        rivets: Rivets,
        /// A leather cover instead of cloth.
        leather: bool,
    },
    /// A mail shirt with plates set in it, front and back.
    MailAndPlate {
        rows: PlateRows,
        sleeves: bool,
    },
    /// Large polished plates strapped over its own mail shirt.
    Mirror {
        mirrors: Mirrors,
    },
    /// Upright strips laced side by side; `rods` are round, as Tlingit rod armour.
    Splint {
        rods: bool,
    },
    /// Rattan woven over a frame and oiled: Chinese tengjia.
    Woven,
}
