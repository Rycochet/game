//! Socks and foot wraps, inside the footwear: sewn, nålbound, felt, wrapped or knitted.

use serde::{Deserialize, Serialize};

use super::table::{has, when};
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SockMake {
    /// Sewn from cloth (udones, tabi).
    Sewn,
    /// Nålbound: thick and elastic (Coptic Egypt, Viking York).
    Nalbound,
    /// Felt, worn inside boots (piloi).
    Felt,
    /// A square or oblong cloth wrapped round the foot.
    Wrapped,
    /// Knitted (16th century on; jorabs, çorape).
    Knitted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Toe {
    Whole,
    /// The big toe apart, for a thong sandal.
    Split,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SockHeight {
    /// The foot alone: wraps only.
    Foot,
    Ankle,
    Calf,
    Knee,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SockTop {
    Plain,
    Banded,
    /// Turned down.
    Cuff,
    Tied,
    /// Hooks down the back (kohaze): tabi only.
    Hooks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Socks {
    pub make: SockMake,
    pub toe: Toe,
    pub height: SockHeight,
    pub top: SockTop,
}

const MAKES: [(SockMake, u64); 5] = [
    (SockMake::Sewn, 30),
    (SockMake::Nalbound, 20),
    (SockMake::Felt, 10),
    (SockMake::Wrapped, 25),
    (SockMake::Knitted, 15),
];

fn toes(make: SockMake) -> [(Toe, u64); 2] {
    let may_split = matches!(make, SockMake::Sewn | SockMake::Nalbound);
    [(Toe::Whole, 85), (Toe::Split, when(may_split, 15))]
}

fn heights(make: SockMake, toe: Toe) -> [(SockHeight, u64); 4] {
    use SockHeight::{Ankle, Calf, Foot, Knee};
    match (make, toe) {
        (SockMake::Wrapped, _) => [(Foot, 60), (Ankle, 40), (Calf, 0), (Knee, 0)],
        (SockMake::Felt, _) => [(Foot, 0), (Ankle, 50), (Calf, 50), (Knee, 0)],
        (_, Toe::Split) => [(Foot, 0), (Ankle, 70), (Calf, 30), (Knee, 0)],
        _ => [(Foot, 0), (Ankle, 30), (Calf, 45), (Knee, 25)],
    }
}

fn tops(make: SockMake, toe: Toe) -> &'static [(SockTop, u64)] {
    use SockTop::{Banded, Cuff, Hooks, Plain, Tied};
    match (make, toe) {
        (SockMake::Wrapped, _) => &[(Plain, 1)],
        (SockMake::Sewn, Toe::Split) => &[(Hooks, 70), (Tied, 30)],
        (SockMake::Sewn, _) => &[(Plain, 40), (Banded, 30), (Cuff, 15), (Tied, 15)],
        (SockMake::Nalbound, _) => &[(Plain, 60), (Banded, 20), (Tied, 20)],
        (SockMake::Felt, _) => &[(Plain, 70), (Cuff, 30)],
        (SockMake::Knitted, _) => &[(Plain, 40), (Banded, 30), (Cuff, 30)],
    }
}

impl Socks {
    pub(super) fn roll(rng: &mut Rng) -> Self {
        let make = rng.pick_weighted(&MAKES);
        let toe = rng.pick_weighted(&toes(make));
        Self {
            make,
            toe,
            height: rng.pick_weighted(&heights(make, toe)),
            top: rng.pick_weighted(tops(make, toe)),
        }
    }

    pub(super) fn allowed(&self) -> bool {
        has(&MAKES, self.make)
            && has(&toes(self.make), self.toe)
            && has(&heights(self.make, self.toe), self.height)
            && has(tops(self.make, self.toe), self.top)
    }

    pub(super) fn name(&self) -> &'static str {
        match (self.make, self.toe) {
            (SockMake::Wrapped, _) => "Foot Wraps",
            (SockMake::Sewn, Toe::Split) => "Tabi",
            (SockMake::Nalbound, Toe::Split) => "Coptic Socks",
            (SockMake::Nalbound, _) => "Nålbound Socks",
            (SockMake::Felt, _) => "Piloi",
            (SockMake::Sewn, _) if self.height == SockHeight::Knee => "Udones",
            _ => "Socks",
        }
    }

    /// Grams for a pair in plain linen or wool at an ordinary size.
    pub(super) fn base_grams(&self) -> u32 {
        let pair = match self.height {
            SockHeight::Foot => 80,
            SockHeight::Ankle => 60,
            SockHeight::Calf => 90,
            SockHeight::Knee => 130,
        };
        match self.make {
            // Thick looped wool, twice a sewn pair's.
            SockMake::Nalbound => pair * 2,
            _ => pair,
        }
    }
}
