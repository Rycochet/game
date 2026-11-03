use super::*;

fn field() -> WeatherField {
    WeatherField::new(&WorldParams::from_seed(Seed::new(12345)))
}

// Golden values: if these change, all clients on older builds disagree.
#[test]
fn golden_samples() {
    let field = field();
    let samples: Vec<_> = [(0, 0, 0.0), (120, -80, 3.5), (1_000_000, 42, 1000.25)]
        .iter()
        .map(|&(x, y, hours)| field.sample(x, y, hours))
        .collect();
    assert_eq!(format!("{samples:?}"), GOLDEN_SAMPLES);
}

const GOLDEN_SAMPLES: &str = "[WeatherSample { cover: 0.6655160302734207, rain: 0.0 }, WeatherSample { cover: 0.0, rain: 0.0 }, WeatherSample { cover: 0.1131246678801845, rain: 0.0 }]";

#[test]
fn noise_stays_in_range_and_varies() {
    let noise = Simplex4::new(Seed::new(1));
    let (mut lowest, mut highest) = (f64::MAX, f64::MIN);
    for i in 0..200_000 {
        let v = i as f64 * 0.0137;
        let value = noise.get(v, v * 0.71 + 3.0, v * 0.37 - 9.0, v * 1.13);
        lowest = lowest.min(value);
        highest = highest.max(value);
    }
    assert!(lowest > -1.2 && highest < 1.2, "{lowest}..{highest}");
    assert!(lowest < -0.5 && highest > 0.5, "{lowest}..{highest}");
}

#[test]
fn wraps_east_west() {
    let field = field();
    let width = field.width as i64;
    assert_eq!(field.sample(3, 7, 2.5), field.sample(3 + width, 7, 2.5));
}

#[test]
fn the_sky_changes_gradually() {
    let field = field();
    let (a, b) = (field.sample(50, 50, 10.0), field.sample(50, 50, 10.01));
    assert!((a.cover - b.cover).abs() < 0.05);
}

/// Over a stretch of world and time there are clear skies, clouds and rain.
#[test]
fn there_is_a_range_of_weather() {
    let field = field();
    let (mut clear, mut cloudy, mut raining) = (0, 0, 0);
    for i in 0..4000i64 {
        let sample = field.sample(i * 37, i * 53 % 9000, i as f64 * 1.7);
        clear += usize::from(sample.cover < 0.1);
        cloudy += usize::from(sample.cover > 0.5);
        raining += usize::from(sample.rain > 0.1);
    }
    assert!(
        clear > 400 && cloudy > 400 && raining > 100,
        "{clear} {cloudy} {raining}"
    );
}
