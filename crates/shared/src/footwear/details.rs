//! What a pair of footwear rolls about how it is held on and finished: fastening, the
//! top edge of a boot, decoration and what is under the sole.

use serde::{Deserialize, Serialize};

/// How it is held on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Fastening {
    /// Nothing: slip-on shoes, pull-on boots.
    PullOn,
    /// Facings under the vamp: Oxfords.
    ClosedLaces,
    /// Facings on top of the vamp: Derbies, and laced boots.
    OpenLaces,
    /// Laces through loops with no tongue, wound round the ankle: ghillies, carbatinas,
    /// crepidae.
    Loops,
    /// One or two straps with buckles.
    Buckle,
    /// A row up the outer side.
    Buttons,
    /// Leather toggles through loops: turnshoes.
    Toggles,
    /// Chelsea boots only.
    ElasticSides,
    /// Thongs or cords wound round the foot and shin.
    Thongs,
    /// Between the first two toes, or a knob held by them.
    ToeThong,
    /// Round the top of a soft boot.
    Drawstring,
    /// One band over the foot, without a buckle.
    Strap,
}

impl Fastening {
    pub const ALL: [Self; 12] = [
        Self::PullOn,
        Self::ClosedLaces,
        Self::OpenLaces,
        Self::Loops,
        Self::Buckle,
        Self::Buttons,
        Self::Toggles,
        Self::ElasticSides,
        Self::Thongs,
        Self::ToeThong,
        Self::Drawstring,
        Self::Strap,
    ];

    /// Laced up the front.
    pub fn laced(self) -> bool {
        matches!(self, Self::ClosedLaces | Self::OpenLaces)
    }
}

/// The top edge of a boot's shaft.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Top {
    /// Straight round: everything that isn't a boot has this.
    Plain,
    /// Turned down, in another colour: top boots.
    Cuff,
    /// Wide and flared: jackboots and cavalier boots.
    Bucket,
    /// A V at the front with a tassel: Hessians.
    Notch,
    /// A wavy edge: cowboy and button boots.
    Scalloped,
    /// Fur round the top.
    Fur,
    /// Higher at the front than the back: gutals.
    Uneven,
}

impl Top {
    pub const ALL: [Self; 7] = [
        Self::Plain,
        Self::Cuff,
        Self::Bucket,
        Self::Notch,
        Self::Scalloped,
        Self::Fur,
        Self::Uneven,
    ];
}

/// How it is decorated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Decoration {
    None,
    /// Punched holes and serrated edges along the seams: laced leather only.
    Broguing,
    /// Thread, gold or silver.
    Embroidery,
    /// Beads or quillwork.
    Beading,
    /// Cuts showing the lining, in another colour.
    Slashing,
    /// Rows of stitching or tooling.
    Stitching,
    /// A wool pompom on an upturned toe.
    Pompom,
    /// On the vamp or the top.
    Tassels,
    /// Leather fringe.
    Fringe,
    /// Carved or painted wood.
    Carved,
    /// Pearls or stones, at Rare and better only.
    Jewels,
    /// Straps plaited from strips: huaraches, kolhapuris.
    Plaited,
}

impl Decoration {
    pub const ALL: [Self; 12] = [
        Self::None,
        Self::Broguing,
        Self::Embroidery,
        Self::Beading,
        Self::Slashing,
        Self::Stitching,
        Self::Pompom,
        Self::Tassels,
        Self::Fringe,
        Self::Carved,
        Self::Jewels,
        Self::Plaited,
    ];
}

/// What is under the sole.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Under {
    Nothing,
    /// Iron studs in rows.
    Hobnails,
    /// Plates at the heel and toe; clog irons on clogs.
    Irons,
}

impl Under {
    pub const ALL: [Self; 3] = [Self::Nothing, Self::Hobnails, Self::Irons];
}
