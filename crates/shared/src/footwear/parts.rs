//! What a pair of footwear rolls about its shape: the kind of item, how it is made,
//! how high it comes, its toe and its heel.

use serde::{Deserialize, Serialize};

use crate::item::ItemKind;

/// The kinds of footwear item, all worn in the Middle layer on the feet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FootwearPiece {
    Shoes,
    Boots,
    Sandals,
    Clogs,
    Slippers,
}

impl FootwearPiece {
    pub const ALL: [Self; 5] = [
        Self::Shoes,
        Self::Boots,
        Self::Sandals,
        Self::Clogs,
        Self::Slippers,
    ];

    /// The piece an item kind makes, for the kinds that are footwear.
    pub fn of(kind: ItemKind) -> Option<Self> {
        Some(match kind {
            ItemKind::Shoes => Self::Shoes,
            ItemKind::Boots => Self::Boots,
            ItemKind::Sandals => Self::Sandals,
            ItemKind::Clogs => Self::Clogs,
            ItemKind::Slippers => Self::Slippers,
            _ => return None,
        })
    }
}

/// How it is made, which decides almost everything else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Build {
    /// An upper sewn to a separate sole, which may have a heel.
    Sewn,
    /// Sole and sides from one piece wrapped up round the foot: moccasins, turnshoes,
    /// mukluks.
    Soft,
    /// One piece of hide gathered round the foot by thongs.
    Rawhide,
    /// Straw, bast, rope or grass, plaited or coiled.
    Woven,
    /// Felted wool in one piece.
    Felted,
    /// A sole held on by straps, loops or a thong.
    Strapped,
    /// Carved wood, whole or as the sole.
    Wooden,
}

impl Build {
    pub const ALL: [Self; 7] = [
        Self::Sewn,
        Self::Soft,
        Self::Rawhide,
        Self::Woven,
        Self::Felted,
        Self::Strapped,
        Self::Wooden,
    ];
}

/// Where the top stops.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Height {
    /// Below the ankle bone.
    Low,
    /// Over the ankle bone.
    Ankle,
    /// To the middle of the calf.
    Calf,
    /// Below or at the knee.
    Knee,
    /// Over the knee: the cavalier boot only (the owner's decision).
    Thigh,
}

impl Height {
    pub const ALL: [Self; 5] = [Self::Low, Self::Ankle, Self::Calf, Self::Knee, Self::Thigh];

    /// How high the top is above the ground, in hundredths of the foot's length:
    /// (least, most). From the research's estimates.
    pub fn shaft(self) -> (u16, u16) {
        match self {
            Self::Low => (20, 30),
            Self::Ankle => (45, 60),
            Self::Calf => (100, 120),
            Self::Knee => (170, 190),
            Self::Thigh => (260, 290),
        }
    }
}

/// The toe, seen from above.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Toe {
    /// The usual.
    Round,
    /// Narrowing to a soft point.
    Almond,
    /// A clear point at the toes.
    Pointed,
    /// A point reaching well past the toes: poulaines.
    LongPoint,
    /// Flat across: 17th century shoes.
    Square,
    /// Wider than the foot: duckbill shoes.
    Broad,
    /// The tip curls up: juttis, opanaks, gutals.
    Upturned,
    /// The toes show: sandals and wooden soles.
    Open,
}

impl Toe {
    pub const ALL: [Self; 8] = [
        Self::Round,
        Self::Almond,
        Self::Pointed,
        Self::LongPoint,
        Self::Square,
        Self::Broad,
        Self::Upturned,
        Self::Open,
    ];
}

/// A long point's reach past the toes, in hundredths of the foot's length (the
/// research: 0.1 to 0.5).
pub const LONG_POINT: (u8, u8) = (10, 50);
/// A broad toe's width, in hundredths of the foot's width (as broad sabatons).
pub const BROAD: (u8, u8) = (130, 220);

/// What is under the heel. No stilettos or modern high heels (the owner's rule).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Heel {
    Flat,
    /// Stacked leather.
    Low,
    /// Stacked and angled: cowboy, cavalier and engineer boots.
    Riding,
    /// Curved: 17th to 19th century mules, court shoes and button boots.
    Louis,
    /// Wooden teeth or blocks under the sole: geta, nalins, padukas.
    Teeth,
}

impl Heel {
    pub const ALL: [Self; 5] = [
        Self::Flat,
        Self::Low,
        Self::Riding,
        Self::Louis,
        Self::Teeth,
    ];

    /// How high it is, in millimetres at an ordinary size: (least, most).
    pub fn millimetres(self) -> (u8, u8) {
        match self {
            Self::Flat => (0, 0),
            Self::Low => (10, 25),
            Self::Riding => (25, 45),
            Self::Louis => (40, 65),
            Self::Teeth => (30, 90),
        }
    }
}
