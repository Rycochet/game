use super::*;

#[test]
fn golden_world() {
    let world = WorldParams::from_seed(Seed::new(12345));
    assert_eq!(format!("{world:?}"), GOLDEN_WORLD);
    assert_eq!(
        world.water_level(&Celestial::from_seed(world.seed), 2.0),
        GOLDEN_WATER_AT_2
    );
}

// Golden values: if these change, every world gets a new terrain and id.
#[test]
fn golden_world_seed() {
    let seed = WorldSeed::new(12345);
    assert_eq!(seed.terrain().value(), GOLDEN_TERRAIN_SEED);
    assert_eq!(seed.id().0.to_string(), GOLDEN_WORLD_ID);
    // Each purpose gets its own seed, none of them the secret one.
    assert!(seed.terrain() != seed.0 && seed.pois() != seed.0);
    assert!(seed.terrain() != seed.pois());
}

#[test]
fn a_world_id_is_version_5_and_depends_only_on_the_seed() {
    let id = WorldSeed::new(12345).id();
    assert_eq!(id.0.get_version_num(), 5);
    assert_eq!(id, WorldSeed::new(12345).id());
    assert_ne!(id, WorldSeed::new(12346).id());
}

const GOLDEN_TERRAIN_SEED: u64 = 15274154617967521790;
const GOLDEN_WORLD_ID: &str = "61384f54-ba35-5b56-83dc-127e0e995d12";

const GOLDEN_WORLD: &str = "WorldParams { seed: Seed(12345), radius: 4760643, circumference: 29912002, year: 152, density: 1.2471431745178077, tilt: 0.38362863605768616, sea_level: 50.762015892761944, climate: 0.19909579997428872, warmth: -0.015898052726041195 }";
const GOLDEN_WATER_AT_2: f64 = 51.43738271905827;
