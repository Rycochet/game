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
    /// Hall gate it leads to.
    pub(crate) fn gates(&self) -> Vec<GateInfo> {
        let gates = &self.pois.world_gates().gates;
        gates
            .iter()
            .enumerate()
            .map(|(entrance, gate)| GateInfo {
                entrance: entrance as u8,
                x: gate.x,
                y: gate.y,
                place: GateAddress::for_world(self.seed, entrance as u8)
                    .map(|address| address.place().number()),
            })
            .collect()
    }

    /// Where World Gate `entrance` stands on the world map, if it has one.
    pub(crate) fn gate_position(&self, entrance: u8) -> Option<(i64, i64)> {
        let gates = &self.pois.world_gates().gates;
        gates
            .get(usize::from(entrance))
            .map(|gate| (gate.x, gate.y))
    }
}

/// Through World Gate `entrance` of the player's world, if they are on the
/// world map within `ENTER_REACH` of it: into the Hall, before the gate it
/// leads to. A gate of a world that isn't one of the Hall's leads nowhere.
/// `false` if the client has gone.
pub(crate) fn enter_gate(client: &mut Client, world: &mut ServerWorld, entrance: u8) -> bool {
    let Some(gate) = world.gate_position(entrance) else {
        warn!("A client tried to go through World Gate {entrance}, which isn't there");
        return true;
    };
    let (x, y) = client.world_position();
    // East-west, the nearer way round the planet.
    let dx = (gate.0 - x).rem_euclid(world.width);
    let near = dx.min(world.width - dx) <= ENTER_REACH && (gate.1 - y).abs() <= ENTER_REACH;
    if client.visit.is_some() || client.character.hall.is_some() || !near {
        warn!("A client tried to go through World Gate {entrance} from elsewhere");
        return true;
    }
    let Some(address) = GateAddress::for_world(world.seed, entrance) else {
        info!("World Gate {entrance} of this world leads nowhere: it isn't one of the Hall's");
        return true;
    };
    hall::enter(client, world, address.place().front())
}

/// Where the gate whose floor is at `at` in the Hall leads: the planet's seed
/// and which of its World Gates. `None` if no gate stands there.
pub(crate) fn hall_gate_leads(at: (i64, i64)) -> Option<(WorldSeed, u8)> {
    let address = GatePlace::at(at.0, at.1)?.address();
    Some((WorldSeed::new(address.seed().into()), address.entrance()))
}
