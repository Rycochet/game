use super::*;
use crate::rng::Seed;
use crate::surface::SurfaceGenerator;

/// A world from a seed.
fn world(seed: u64) -> WorldParams {
    WorldParams::from_seed(Seed::new(seed))
}

fn gates(seed: u64, generator: SurfaceGenerator) -> (WorldParams, Surface, WorldGates) {
    let world = world(seed);
    let surface = Surface::with_generator(&world, generator);
    let gates = WorldGates::find(&world, &surface);
    (world, surface, gates)
}

#[test]
fn the_same_seed_has_the_same_gates() {
    let (_, _, first) = gates(42, SurfaceGenerator::Continents);
    let (_, _, again) = gates(42, SurfaceGenerator::Continents);
    assert_eq!(first, again);
}

#[test]
fn gates_stand_on_dry_land_apart_and_away_from_the_poles() {
    for generator in SurfaceGenerator::ALL {
        let (world, surface, found) = gates(7, generator);
        assert!(
            found.complete(),
            "{generator:?}: {} gates",
            found.gates.len()
        );
        let width = surface.width();
        let furthest = latitude_y(FURTHEST_LATITUDE, width) + CELL;
        let least_apart = (width as f64 * SPACING_SHARE).min(MOST_SPACING) * ROUNDS[3].spacing;
        for (number, gate) in found.gates.iter().enumerate() {
            assert!(surface.sample(gate.x, gate.y).height > world.high_water());
            assert!(gate.y.abs() <= furthest, "{generator:?}: {gate:?}");
            assert_eq!(gate.chunk, chunk_of(gate.x, gate.y, width));
            for other in &found.gates[..number] {
                assert!(distance(other, (gate.x, gate.y), width) >= least_apart);
            }
        }
    }
}

#[test]
fn a_planet_of_water_has_no_gates() {
    // Desert's heights don't follow the sea, so raising it drowns the land.
    let mut world = world(3);
    world.sea_level = 1000.0;
    let surface = Surface::with_generator(&world, SurfaceGenerator::Desert);
    let found = WorldGates::find(&world, &surface);
    assert!(found.gates.is_empty());
    assert_eq!(
        found.tries,
        TRIES_PER_ROUND * ROUNDS.len() as u32,
        "gives up"
    );
}

/// Many planets in a row, as seeds will be chosen: all must have every gate.
/// Prints which rounds they needed (`cargo test world_gate -- --nocapture`).
#[test]
fn every_planet_has_all_its_gates() {
    for generator in SurfaceGenerator::ALL {
        let mut last_rounds = [0; ROUNDS.len()];
        let mut tries = 0;
        for seed in 0..120 {
            let (_, _, found) = gates(seed, generator);
            assert!(
                found.complete(),
                "{generator:?} seed {seed}: {} gates",
                found.gates.len()
            );
            let last = found.gates.iter().map(|gate| gate.round).max().unwrap_or(0);
            last_rounds[last as usize] += 1;
            tries += found.tries;
        }
        println!("{generator:?}: planets finishing in each round {last_rounds:?}, {tries} tries");
    }
}
