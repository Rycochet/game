use super::*;
use shared::world::WorldSeed;

#[test]
fn bolts_strike_near_but_clear_of_players_and_pois() {
    let world = ServerWorld::new(WorldSeed::new(1));
    let mut struck = 0;
    for y in (-200..200).step_by(10) {
        for x in (0..400).step_by(10) {
            let near = (x * SCALE + SCALE / 2, y * SCALE + SCALE / 2);
            let players = [(near.0, near.1, SCALE)];
            let Some(strike) = world.strike_near(near, 6 * SCALE, 7, &players) else {
                continue;
            };
            struck += 1;
            let away = apart(strike.x - near.0, world.width * SCALE).max((strike.y - near.1).abs());
            assert!(away > SCALE && away <= 6 * SCALE, "{away}");
            assert!(!world.near_poi(strike.x, strike.y));
            // The same every time.
            assert_eq!(
                world.strike_near(near, 6 * SCALE, 7, &players),
                Some(strike)
            );
        }
    }
    assert!(struck > 800, "{struck}");
}

#[test]
fn apart_goes_the_short_way_round() {
    assert_eq!(apart(3, 100), 3);
    assert_eq!(apart(-3, 100), 3);
    assert_eq!(apart(98, 100), 2);
}
