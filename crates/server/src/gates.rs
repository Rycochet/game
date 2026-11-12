//! Going through World Gates: a planet's into the Hall of Worlds, before the
//! gate it leads to, and a Hall gate onto its planet, at the World Gate it
//! leads to (`shared::hall`). The world a gate leads into may be another, so
//! that way goes through `connections`' world moves.

use bevy::prelude::*;
use shared::hall::{GateAddress, GatePlace};
use shared::protocol::{ENTER_REACH, GateInfo};
use shared::world::WorldSeed;

use crate::connections::Client;
use crate::{ServerWorld, hall};

impl ServerWorld {
    /// Its World Gates as clients are told them: where each stands, and the
    /// Hall gate it leads to. None unless it is one of the Hall's planets (its
    /// seed is under `hall::PLANETS`): other worlds are reached by other means.
    /// Where they would stand is still where new characters arrive.
    pub(crate) fn gates(&self) -> Vec<GateInfo> {
        (0..self.pois.world_gates().gates.len() as u8)
            .filter_map(|entrance| self.gate(entrance))
            .collect()
    }

    /// World Gate `entrance`, if it has one.
    pub(crate) fn gate(&self, entrance: u8) -> Option<GateInfo> {
        let address = GateAddress::for_world(self.seed, entrance)?;
        let gate = self.pois.world_gates().gates.get(usize::from(entrance))?;
        Some(GateInfo {
            entrance,
            x: gate.x,
            y: gate.y,
            place: address.place().number(),
        })
    }

    /// Where World Gate `entrance` stands on the world map, if it has one.
    pub(crate) fn gate_position(&self, entrance: u8) -> Option<(i64, i64)> {
        self.gate(entrance).map(|gate| (gate.x, gate.y))
    }
}

/// Through World Gate `entrance` of the player's world, if they are on the
/// world map within `ENTER_REACH` of it: into the Hall, before the gate it
/// leads to. `false` if the client has gone.
pub(crate) fn enter_gate(client: &mut Client, world: &mut ServerWorld, entrance: u8) -> bool {
    let Some(gate) = world.gate(entrance) else {
        warn!("A client tried to go through World Gate {entrance}, which isn't there");
        return true;
    };
    let (x, y) = client.world_position();
    // East-west, the nearer way round the planet.
    let dx = (gate.x - x).rem_euclid(world.width);
    let near = dx.min(world.width - dx) <= ENTER_REACH && (gate.y - y).abs() <= ENTER_REACH;
    if client.visit.is_some() || client.character.hall.is_some() || !near {
        warn!("A client tried to go through World Gate {entrance} from elsewhere");
        return true;
    }
    client.went_through(gate.place)
        && hall::enter(
            client,
            world,
            shared::spot::Spot::middle_of(GatePlace::from_number(gate.place).front()),
        )
}

/// Where the gate whose floor is at `at` in the Hall leads: the planet's seed
/// and which of its World Gates. `None` if no gate stands there.
pub(crate) fn hall_gate_leads(at: (i64, i64)) -> Option<(WorldSeed, u8)> {
    let address = GatePlace::at(at.0, at.1)?.address();
    Some((WorldSeed::new(address.seed().into()), address.entrance()))
}
