use super::super::WeatherSample;
use super::*;
use crate::rng::Seed;
use crate::world::WorldParams;

fn field() -> WeatherField {
    WeatherField::new(&WorldParams::from_seed(Seed::new(12345)))
}

fn rain(rain: f64) -> WeatherSample {
    WeatherSample { cover: 1.0, rain }
}

/// How many of `seconds` seconds flash at the origin.
fn flashes(field: &WeatherField, weather: WeatherSample, temperature: f64, seconds: i64) -> usize {
    (0..seconds)
        .filter(|&second| field.lightning(0, 0, weather, temperature, second))
        .count()
}

#[test]
fn none_below_the_heaviest_rain() {
    assert_eq!(flashes(&field(), rain(LIGHTNING_RAIN), 0.6, 10_000), 0);
    assert_eq!(flashes(&field(), rain(0.5), 0.6, 10_000), 0);
}

#[test]
fn none_in_sleet_or_snow() {
    assert_eq!(
        flashes(&field(), rain(1.0), LIGHTNING_WARMEST - 0.01, 10_000),
        0
    );
}

#[test]
fn now_and_then_in_a_storm() {
    let field = field();
    // About one second in eleven at the worst, half that half way.
    let worst = flashes(&field, rain(1.0), 0.6, 10_000);
    assert!((700..1100).contains(&worst), "{worst}");
    let middle = flashes(&field, rain((1.0 + LIGHTNING_RAIN) / 2.0), 0.6, 10_000);
    assert!((300..600).contains(&middle), "{middle}");
}

#[test]
fn shared_across_an_area() {
    let field = field();
    for second in 0..2000 {
        let a = field.lightning(1, 2, rain(1.0), 0.6, second);
        let b = field.lightning(
            LIGHTNING_AREA - 1,
            LIGHTNING_AREA - 1,
            rain(1.0),
            0.6,
            second,
        );
        assert_eq!(a, b);
    }
}

// Golden values: if these change, players on older and newer servers see
// different storms (only a look, but it should stay put).
#[test]
fn golden_seconds() {
    let field = field();
    let seconds: Vec<i64> = (0..400)
        .filter(|&second| field.lightning(-70, 500, rain(0.97), 0.5, second))
        .collect();
    assert_eq!(
        format!("{seconds:?}"),
        "[37, 39, 44, 51, 61, 74, 96, 116, 133, 137, 142, 147, 159, 187, 194, 195, 198, 199, 213, 225, 237, 257, 264, 279, 309, 310, 330, 344, 349, 365, 367, 375, 397, 399]"
    );
}
