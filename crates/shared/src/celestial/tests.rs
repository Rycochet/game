use super::*;

fn world(seed: u64) -> Celestial {
    Celestial::from_seed(Seed::new(seed))
}

#[test]
fn suns_and_moons_are_about_as_common_as_they_should_be() {
    let worlds = 200_000;
    let mut suns = [0u32; 4];
    let mut moons = [0u32; 6];
    for seed in 0..worlds {
        let world = world(seed);
        suns[world.suns.len()] += 1;
        moons[world.moons.len()] += 1;
    }
    let share = |count: u32| f64::from(count) / worlds as f64;
    assert!((share(suns[2]) - 0.05).abs() < 0.005, "{suns:?}");
    assert!((share(suns[3]) - 0.001).abs() < 0.0005, "{suns:?}");
    for (count, &chance) in moons.iter().zip(&MOON_CHANCES) {
        assert!(
            (share(*count) - chance as f64 / 100.0).abs() < 0.01,
            "{moons:?}"
        );
    }
    // One moon the most common, none the least.
    assert!(moons.iter().all(|&count| count <= moons[1]));
    assert!(moons.iter().all(|&count| count >= moons[0]));
}

/// Hours above the horizon in a day, near enough, and noon is at 12.
#[test]
fn the_main_sun_keeps_the_day() {
    for seed in 0..200 {
        let world = world(seed);
        let main = |hour: f64| world.at(hour)[0];
        let up = (0..24 * 60)
            .filter(|&minute| main(f64::from(minute) / 60.0).is_up())
            .count() as f64
            / 60.0;
        assert!((LEAST_DAYLIGHT_HOURS - 0.1..=MOST_DAYLIGHT_HOURS + 0.1).contains(&up));
        assert!(main(12.0).is_up() && !main(0.0).is_up());
        // The same again a day later.
        let again = main(36.0).direction;
        for (a, b) in main(12.0).direction.iter().zip(again) {
            assert!((a - b).abs() < 1e-9);
        }
    }
}

#[test]
fn suns_cross_the_sky_the_way_they_rise() {
    for seed in 0..50 {
        let world = world(seed);
        let east = |hour: f64| world.at(hour)[0].direction[0];
        let (morning, evening) = (east(9.0), east(15.0));
        if world.rises_in_east {
            assert!(morning > 0.0 && evening < 0.0);
        } else {
            assert!(morning < 0.0 && evening > 0.0);
        }
    }
}

#[test]
fn noon_on_flat_ground_is_the_main_suns_colour() {
    for seed in 0..50 {
        let world = world(seed);
        let main = world.at(12.0)[0];
        let flat = flat_light(core::slice::from_ref(&main));
        for (light, colour) in flat.iter().zip(main.body.colour) {
            assert!((light - colour).abs() < 1e-9);
        }
    }
}

#[test]
fn light_fades_in_rather_than_coming_on() {
    let world = world(1);
    let light = |hour: f64| world.at(hour)[0].light[0];
    let mut last = light(0.0);
    assert_eq!(last, 0.0);
    for minute in 1..=24 * 60 {
        let now = light(f64::from(minute) / 60.0);
        assert!((now - last).abs() < 0.1, "at minute {minute}");
        last = now;
    }
}

/// Companions differ from every other sun and are weaker than the main one.
#[test]
fn companions_differ() {
    let mut found = 0;
    for seed in 0..50_000 {
        let world = world(seed);
        for (i, sun) in world.suns.iter().enumerate().skip(1) {
            found += 1;
            for other in &world.suns[..i] {
                assert!(
                    colour_distance(sun.body.colour, other.body.colour) >= 0.1,
                    "world {seed}"
                );
            }
            assert!(sun.strength < world.suns[0].strength);
        }
    }
    assert!(found > 1000);
}

#[test]
fn blue_suns_are_small_and_quick() {
    let mut blue_period = f64::MAX;
    let mut orange_period = 0.0_f64;
    for seed in 0..50_000 {
        let world = world(seed);
        for sun in world.suns.iter().skip(1) {
            let bluer = sun.body.colour[0] < sun.body.colour[2];
            if bluer {
                blue_period = blue_period.min(sun.period);
            } else {
                orange_period = orange_period.max(sun.period);
            }
        }
    }
    assert!(blue_period < DAY_HOURS && orange_period > DAY_HOURS);
}

#[test]
fn suns_colours_never_turn_green() {
    for step in 0..=100 {
        let [red, green, blue] = sun_colour(f64::from(step) / 100.0, 0.0);
        assert!(green <= red.max(blue), "{step}");
    }
}

#[test]
fn colours_are_named_unless_near_white() {
    assert_eq!(ColourName::of([1.0, 1.0, 1.0]), None);
    assert_eq!(ColourName::of([0.95, 1.0, 0.97]), None);
    assert_eq!(ColourName::of([0.55, 0.7, 1.0]), Some(ColourName::Blue));
    assert_eq!(ColourName::of([1.0, 0.92, 0.6]), Some(ColourName::Yellow));
    assert_eq!(ColourName::of([1.0, 0.68, 0.4]), Some(ColourName::Orange));
    assert_eq!(ColourName::of([1.0, 0.5, 0.5]), Some(ColourName::Red));
    assert_eq!(ColourName::of([0.8, 1.0, 0.7]), Some(ColourName::Green));
    assert_eq!(ColourName::of([0.8, 0.6, 1.0]), Some(ColourName::Violet));
}

#[test]
fn moons_opposite_a_sun_are_full_and_beside_it_are_new() {
    let moon = |direction: [f64; 3]| Moon {
        body: Body {
            kind: BodyKind::Moon,
            colour: [1.0; 3],
            colour_name: None,
            size: 1.0,
        },
        across: [direction, [0.0, 0.0, 1.0]],
        period: 24.0,
        start: 0.0,
    };
    let sun = BodyNow {
        body: Body {
            kind: BodyKind::Sun,
            colour: [1.0; 3],
            colour_name: None,
            size: 1.0,
        },
        direction: [0.0, -1.0, 0.0],
        lit: 1.0,
        waxing: false,
        along: 0.0,
        light: [0.0; 3],
    };
    let full = moon_at(&moon([0.0, 1.0, 0.0]), &[sun], &[sun], 0.0);
    assert_eq!(full.lit, 1.0);
    assert_eq!(full.light, [MOONLIGHT; 3]);
    let new = moon_at(&moon([0.0, -1.0, 0.0]), &[sun], &[sun], 0.0);
    assert_eq!(new.lit, 0.0);
}

#[test]
fn moons_wax_as_their_lit_part_grows_and_have_phases() {
    let world = (0..)
        .map(world)
        .find(|world| !world.moons.is_empty())
        .unwrap();
    let moon = world.suns.len();
    let mut seen = std::collections::HashSet::new();
    for step in 0..5000 {
        let hours = f64::from(step) * 0.1;
        let now = world.at(hours)[moon];
        let later = world.at(hours + WAXING_LOOK_AHEAD)[moon];
        assert_eq!(now.waxing, later.lit > now.lit, "at {hours}");
        let phase = now.phase().unwrap();
        let waxing = matches!(
            phase,
            MoonPhase::WaxingCrescent | MoonPhase::FirstQuarter | MoonPhase::WaxingGibbous
        );
        let waning = matches!(
            phase,
            MoonPhase::WaningCrescent | MoonPhase::LastQuarter | MoonPhase::WaningGibbous
        );
        assert!(!waxing || now.waxing);
        assert!(!waning || !now.waxing);
        seen.insert(format!("{phase:?}"));
    }
    assert!(seen.len() >= 6, "{seen:?}");
    assert_eq!(world.at(0.0)[0].phase(), None);
}

/// While above the horizon, everything goes one way along its path, from
/// before its top to after it, in worlds turning either way.
#[test]
fn bodies_keep_going_along_their_way_over_the_sky() {
    let mut ways = [false; 2];
    for seed in 0..300 {
        let world = world(seed);
        ways[usize::from(world.rises_in_east)] = true;
        let count = world.bodies().len();
        for number in 0..count {
            let mut last: Option<BodyNow> = None;
            for step in 0..3000 {
                let now = world.at(f64::from(step) * 0.05)[number];
                if let Some(last) = last.filter(|last| last.is_up() && now.is_up()) {
                    assert!(
                        now.along > last.along,
                        "world {seed} body {number} step {step}"
                    );
                }
                last = Some(now);
            }
        }
        // The main sun at noon is at the top of its way.
        assert!(world.at(12.0)[0].along.abs() < 1e-9);
        assert!(world.at(9.0)[0].along < 0.0 && world.at(15.0)[0].along > 0.0);
    }
    assert_eq!(ways, [true; 2]);
}

#[test]
fn colours_and_strengths_can_be_changed() {
    let mut world = world(12345);
    let moon = world.suns.len();
    world.set_colour(moon, colour_along(BodyKind::Moon, 4.0 / 6.0));
    assert_eq!(world.bodies()[moon].colour_name, Some(ColourName::Blue));
    world.set_colour(0, colour_along(BodyKind::Sun, 1.0));
    assert_eq!(world.at(12.0)[0].body.colour_name, Some(ColourName::Orange));
    world.set_sun_strength(0, 0.0);
    assert_eq!(world.at(12.0)[0].light, [0.0; 3]);
    // Nothing there: nothing happens.
    world.set_colour(99, [1.0; 3]);
    world.set_sun_strength(moon, 5.0);
    assert_eq!(world.sun_strength(moon), 0.0);
}

#[test]
fn moons_climb_the_sky() {
    for seed in 0..500 {
        let world = world(seed);
        for number in 0..world.moons.len() {
            let highest = (0..2000)
                .map(|i| world.at(f64::from(i) / 10.0)[world.suns.len() + number].direction[1])
                .fold(f64::MIN, f64::max);
            assert!(highest > 0.3, "world {seed}: {highest}");
        }
    }
}

#[test]
fn bodies_and_their_places_match() {
    let world = world(7);
    let bodies = world.bodies();
    let now = world.at(100.5);
    assert_eq!(bodies.len(), world.suns.len() + world.moons.len());
    assert_eq!(now.iter().map(|n| n.body).collect::<Vec<_>>(), bodies);
    assert_eq!(world.at(100.5), now);
}

// Golden values: if these change, every world gets a new sky.
#[test]
fn golden_sky() {
    let world = world(12345);
    assert_eq!(format!("{:?}", world.at(9.5)), GOLDEN_SKY);
}

const GOLDEN_SKY: &str = "[BodyNow { body: Body { kind: Sun, colour: [0.7919773309346745, 0.8613182206231162, 1.0], colour_name: Some(Blue), size: 0.8387194116696959 }, direction: [-0.4142466559940582, 0.7847245973775617, 0.46109328152594464], lit: 1.0, waxing: false, along: -0.5057405771498451, light: [0.8848544385542069, 0.9623271030074982, 1.1172724319140812] }, BodyNow { body: Body { kind: Moon, colour: [1.0, 0.6229742931451838, 0.9630333796082404], colour_name: Some(Violet), size: 0.38467303853895163 }, direction: [-0.5682175123423905, 0.7765099767964616, 0.27232538369198017], lit: 0.014851955962146002, waxing: false, along: -0.4487949123033693, light: [0.00034810470657831426, 0.00023584729793441146, 0.0004232904641828701] }, BodyNow { body: Body { kind: Moon, colour: [1.0, 0.7892538947659659, 0.6835815888213319], colour_name: Some(Orange), size: 0.975111596324884 }, direction: [0.3830264755924152, 0.863143928178815, 0.32904905142439983], lit: 0.16480742912403212, waxing: false, along: 0.5016809540028195, light: [0.024821508610915713, 0.02130569919414452, 0.021424257526623178] }, BodyNow { body: Body { kind: Moon, colour: [1.0, 0.9869641784394392, 0.9994370279804488], colour_name: None, size: 0.981384194948326 }, direction: [-0.20199164293217609, 0.4677803293887748, -0.8604539148742888], lit: 0.47299821297354117, waxing: true, along: 0.8323854186958036, light: [0.07215731511971289, 0.07745200046741861, 0.09105903635547749] }]";
