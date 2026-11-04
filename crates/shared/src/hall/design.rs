//! What each gate is built like.

use super::GatePlace;
use crate::rng::Seed;

/// A gate's design, chosen by a hash of its place. Its code carries it too,
/// so a code with the wrong design is a mistyped one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GateDesign {
    RoundArch,
    SquareLintel,
    PointedArch,
    HorseshoeArch,
    Ogee,
    Trefoil,
    SteppedCorbel,
    Keyhole,
}

impl GateDesign {
    pub const ALL: [Self; 8] = [
        Self::RoundArch,
        Self::SquareLintel,
        Self::PointedArch,
        Self::HorseshoeArch,
        Self::Ogee,
        Self::Trefoil,
        Self::SteppedCorbel,
        Self::Keyhole,
    ];

    pub fn of(place: GatePlace) -> Self {
        let hash = Seed::new(0)
            .derive("hall gate design")
            .at(&[i64::from(place.number())]);
        Self::ALL[(hash % Self::ALL.len() as u64) as usize]
    }

    /// Its number, `0..8`, as codes carry it.
    pub fn index(self) -> u8 {
        self as u8
    }
}
