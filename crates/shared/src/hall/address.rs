//! Where a gate leads: a planet's seed and which of its World Gates.

use super::shuffle::{place_of, seed_at};
use super::{GatePlace, PLANET_BITS, PLANETS};
use crate::world::WorldSeed;
use crate::world_gate::WORLD_GATES;

/// A gate's address: the planet's seed (below `PLANETS`) and its entrance,
/// one of the planet's `WORLD_GATES`. As bits, `entrance << 28 | seed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GateAddress {
    seed: u32,
    entrance: u8,
}

impl GateAddress {
    /// `None` unless the seed is a Hall planet's and the entrance one of 16.
    pub fn new(seed: u32, entrance: u8) -> Option<Self> {
        (seed < PLANETS && usize::from(entrance) < WORLD_GATES).then_some(Self { seed, entrance })
    }

    /// The address of a gate of this world, if it is one of the Hall's.
    pub fn for_world(world: WorldSeed, entrance: u8) -> Option<Self> {
        let seed = u32::try_from(world.value()).ok()?;
        Self::new(seed, entrance)
    }

    pub fn from_bits(bits: u32) -> Self {
        Self {
            seed: bits & (PLANETS - 1),
            entrance: (bits >> PLANET_BITS) as u8,
        }
    }

    pub fn bits(self) -> u32 {
        (u32::from(self.entrance) << PLANET_BITS) | self.seed
    }

    pub fn seed(self) -> u32 {
        self.seed
    }

    /// Which of the planet's World Gates, in the order `WorldGates::find`
    /// gives them.
    pub fn entrance(self) -> u8 {
        self.entrance
    }

    pub fn world(self) -> WorldSeed {
        WorldSeed::new(u64::from(self.seed))
    }

    /// The gate in the Hall that leads here.
    pub fn place(self) -> GatePlace {
        let within = place_of(usize::from(self.entrance), self.seed);
        GatePlace::from_number((u32::from(self.entrance) << PLANET_BITS) | within)
    }

    /// Where the gate at this place leads.
    pub(super) fn of(place: GatePlace) -> Self {
        let number = place.number();
        let entrance = (number >> PLANET_BITS) as u8;
        let seed = seed_at(usize::from(entrance), number & (PLANETS - 1));
        Self { seed, entrance }
    }
}
