use super::shuffle::{MULTIPLIERS, STEPS, place_of, seed_at};
use super::*;
use crate::rng::Seed;
use crate::world::WorldSeed;
use crate::world_gate::WORLD_GATES;

const MASK: u64 = PLANETS as u64 - 1;
const COLUMNS: u64 = (HALL_LENGTH / GATE_SPACING) as u64;

/// Gate numbers to try: both ends, the sixteenths' edges, and many at random.
fn numbers() -> Vec<u32> {
    let mut out = vec![0, 1, u32::MAX, u32::MAX - 1];
    for k in 0..16u32 {
        out.extend([k << PLANET_BITS, (k << PLANET_BITS).wrapping_sub(1)]);
    }
    let mut rng = Seed::new(7).rng();
    out.extend((0..20_000).map(|_| rng.next_u64() as u32));
    out
}

#[test]
fn walkway_gates_and_void() {
    for x in [-8, -1, 0, 1, 2, 3, 4, 5, HALL_LENGTH - 4, HALL_LENGTH] {
        for y in 0..WALKWAY_ROWS {
            assert_eq!(tile(x, y), Tile::Walkway);
        }
        let gate = wrap_x(x) % GATE_SPACING == 0;
        for (y, side) in [(-1, Side::North), (WALKWAY_ROWS, Side::South)] {
            let expected = if gate { Tile::Gate(side) } else { Tile::Void };
            assert_eq!(tile(x, y), expected, "{x}, {y}");
        }
        assert_eq!(tile(x, -2), Tile::Void);
        assert_eq!(tile(x, WALKWAY_ROWS + 1), Tile::Void);
    }
    assert_eq!(tile(HALL_LENGTH + 5, 0), tile(5, 0));
    assert_eq!(wrap_x(-1), HALL_LENGTH - 1);
}

#[test]
fn hue_goes_round_once() {
    assert_eq!(hue(0), 0.0);
    assert_eq!(hue(HALL_LENGTH / 2), 180.0);
    assert_eq!(hue(HALL_LENGTH), 0.0);
    assert!(hue(-1) > 359.9 && hue(-1) < 360.0);
}

#[test]
fn every_gate_number_is_one_place() {
    assert_eq!(COLUMNS * 2, 1 << 32, "the Hall holds every u32");
    for number in numbers() {
        let place = GatePlace::from_number(number);
        assert_eq!(place.number(), number);
        assert_eq!(GatePlace::at(place.x(), place.y()), Some(place));
        assert_eq!(
            GatePlace::at(place.x() + HALL_LENGTH, place.y()),
            Some(place)
        );
        let (x, y) = place.front();
        assert_eq!(tile(x, y), Tile::Walkway);
        assert_eq!((x, (y - place.y()).abs()), (place.x(), 1));
    }
    assert_eq!(GatePlace::at(1, -1), None);
    assert_eq!(GatePlace::at(0, 0), None);
}

#[test]
fn multipliers_undo_steps() {
    for k in 0..WORLD_GATES {
        assert_eq!(STEPS[k] % 2, 1);
        assert_eq!((STEPS[k] * MULTIPLIERS[k]) & MASK, 1, "sixteenth {k}");
    }
}

#[test]
fn addresses_and_places_go_both_ways() {
    for number in numbers() {
        let place = GatePlace::from_number(number);
        let address = place.address();
        assert_eq!(address.place(), place);
        assert_eq!(GateAddress::from_bits(address.bits()), address);
        assert_eq!(
            GateAddress::new(address.seed(), address.entrance()),
            Some(address)
        );
    }
    let mut rng = Seed::new(8).rng();
    for _ in 0..5_000 {
        let seed = rng.range_u64(0, u64::from(PLANETS)) as u32;
        for entrance in 0..16 {
            assert_eq!(seed_at(entrance, place_of(entrance, seed)), seed);
            let address = GateAddress::new(seed, entrance as u8).unwrap();
            assert_eq!(address.place().address(), address);
        }
    }
}

#[test]
fn a_planets_gates_are_a_sixteenth_apart() {
    for seed in [0, 1, 12345, PLANETS - 1] {
        for entrance in 0..16u8 {
            let number = GateAddress::new(seed, entrance).unwrap().place().number();
            assert_eq!(number >> PLANET_BITS, u32::from(entrance));
        }
    }
}

#[test]
fn only_hall_planets_have_addresses() {
    assert!(GateAddress::new(PLANETS, 0).is_none());
    assert!(GateAddress::new(0, 16).is_none());
    let world = GateAddress::for_world(WorldSeed::new(12345), 3).unwrap();
    assert_eq!(world.world(), WorldSeed::new(12345));
    assert!(GateAddress::for_world(WorldSeed::new(u64::from(PLANETS)), 0).is_none());
}

/// The gates of this column and the next: what stands beside each other.
fn side_by_side(column: u64) -> [u32; 4] {
    let first = (column * 2) as u32;
    let next = (((column + 1) % COLUMNS) * 2) as u32;
    [first, first + 1, next, next + 1]
}

#[test]
fn planets_are_neighbours_at_most_once() {
    // Within a sixteenth, gates up to 3 numbers apart stand side by side, and
    // their seeds differ by that many steps. No difference may come up in two
    // sixteenths, nor be 0 (a planet beside itself).
    let mut seen = std::collections::BTreeMap::new();
    for (k, step) in STEPS.iter().enumerate() {
        for apart in 1..=3u64 {
            for difference in [
                (step * apart) & MASK,
                (PLANETS as u64 - step * apart % PLANETS as u64) & MASK,
            ] {
                assert_ne!(difference, 0);
                if let Some(other) = seen.insert(difference, k) {
                    assert_eq!(other, k, "sixteenths {other} and {k} share neighbours");
                }
            }
        }
    }
}

#[test]
fn neighbours_lead_to_different_planets() {
    let edges = (0..16u64).map(|k| (k << PLANET_BITS) / 2);
    let mut rng = Seed::new(9).rng();
    let random = (0..20_000).map(|_| rng.range_u64(0, COLUMNS));
    for column in edges
        .flat_map(|c| (0..8).map(move |i| (c + COLUMNS - 4 + i) % COLUMNS))
        .chain(random)
    {
        let seeds = side_by_side(column).map(|n| GatePlace::from_number(n).address().seed());
        for i in 0..4 {
            for j in i + 1..4 {
                assert_ne!(seeds[i], seeds[j], "column {column}");
            }
        }
    }
}

#[test]
fn runes_are_all_different() {
    for (i, a) in RUNES.iter().enumerate() {
        assert!(!RUNES[i + 1..].contains(a));
        assert_ne!(*a, SEPARATOR);
    }
}

#[test]
fn codes_name_their_gates() {
    for number in numbers() {
        let place = GatePlace::from_number(number);
        let code = place.code();
        assert_eq!(code.place(), Some(place));
        let shown = code.to_string();
        assert_eq!(shown.chars().count(), CODE_LENGTH + 1);
        assert_eq!(shown.chars().nth(3), Some(SEPARATOR));
        assert_eq!(GateCode::parse(&shown), Some(code));
        assert_eq!(GateCode::parse(&shown.replace(SEPARATOR, " ")), Some(code));
    }
    assert_eq!(GateCode::parse("abc"), None);
    assert_eq!(
        GateCode::parse(&RUNES[..6].iter().collect::<String>()),
        None
    );
    assert_eq!(
        GateCode::parse(&RUNES[..8].iter().collect::<String>()),
        None
    );
}

#[test]
fn mistyped_codes_are_mostly_caught() {
    let (mut tried, mut caught) = (0, 0);
    for number in numbers().into_iter().take(2_000) {
        let place = GatePlace::from_number(number);
        let code = place.code();
        // The wrong design for the same gate is never accepted.
        let mut wrong = code;
        wrong.0[0] ^= 0b100;
        assert_eq!(wrong.place(), None);
        for i in 0..CODE_LENGTH {
            let mut typo = code;
            typo.0[i] = (typo.0[i] + 1) % 32;
            tried += 1;
            caught += usize::from(typo.place().is_none());
        }
    }
    let share = caught as f64 / tried as f64;
    assert!(share > 0.8, "caught {share}");
}

/// Exact values for fixed gates: a change here moves every gate, so it must be
/// on purpose.
#[test]
fn golden() {
    let place = GateAddress::new(12345, 0).unwrap().place();
    assert_eq!(place.number(), 175_562_835);
    assert_eq!(place.design(), GateDesign::PointedArch);
    assert_eq!(place.code().to_string(), "ᚻᚠᚠ᛫ᚠᛇᚢᚣ");
    let address = GatePlace::from_number(1_000_000).address();
    assert_eq!((address.seed(), address.entrance()), (123_989_860, 0));
}

#[test]
fn rune_glyphs_fit_and_differ() {
    let mut shapes: Vec<Vec<Stroke>> = Vec::new();
    for strokes in RUNE_STROKES {
        assert!(!strokes.is_empty());
        for &[x0, y0, x1, y1] in strokes {
            assert!(x0.max(x1) <= GLYPH_WIDTH && y0.max(y1) <= GLYPH_HEIGHT);
        }
        // The same strokes in any order or direction would look the same.
        let mut shape: Vec<Stroke> = strokes
            .iter()
            .map(|&[x0, y0, x1, y1]| {
                let (a, b) = ((x0, y0).min((x1, y1)), (x0, y0).max((x1, y1)));
                [a.0, a.1, b.0, b.1]
            })
            .collect();
        shape.sort();
        assert!(!shapes.contains(&shape), "two runes look the same");
        shapes.push(shape);
    }
}
