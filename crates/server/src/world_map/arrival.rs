//! Arriving on a planet: through one of its World Gates (`shared::world_gate`).

use shared::protocol::CharacterId;

use crate::ServerWorld;

/// Where a world with no gates (only a test's planet of water) is arrived at.
const NO_GATE: (i64, i64) = (0, 0);

/// Where a character new to this world arrives: one of its World Gates,
/// picked from their id, so it is the same one each time they start again.
pub(crate) fn arrival(world: &ServerWorld, character: &CharacterId) -> (i64, i64) {
    let gates = &world.pois.world_gates().gates;
    if gates.is_empty() {
        return NO_GATE;
    }
    let mut rng = character.0.seed().derive("arrival gate").rng();
    let gate = &gates[rng.range_u64(0, gates.len() as u64) as usize];
    (gate.x, gate.y)
}

/// The World Gate nearest to a place, the near way round the planet.
pub(crate) fn nearest_gate(world: &ServerWorld, (x, y): (i64, i64)) -> (i64, i64) {
    let width = world.pois.width();
    world
        .pois
        .world_gates()
        .gates
        .iter()
        .map(|gate| (gate.x, gate.y))
        .min_by_key(|&(gate_x, gate_y)| {
            let across = (gate_x - x).rem_euclid(width);
            let dx = across.min(width - across);
            let dy = gate_y - y;
            dx * dx + dy * dy
        })
        .unwrap_or(NO_GATE)
}
