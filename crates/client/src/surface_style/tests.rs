use shared::climate::ClimateSample;
use shared::surface::SurfaceSample;

use super::*;

#[test]
fn hsl_matches_known_colours() {
    assert_eq!(hsl_to_rgba(0.0, 0.0, 0.55), [140, 140, 140, 255]); // #8c8c8c
    assert_eq!(hsl_to_rgba(232.0, 1.0, 0.29), [0, 20, 148, 255]); // ~#001493
    assert_eq!(hsl_to_rgba(0.0, 1.0, 0.5), [255, 0, 0, 255]);
}

#[test]
fn ground_colour_follows_the_biome_or_snow() {
    let style = SurfaceStyle::default();
    let colour = |temperature, moisture| {
        let climate = ClimateSample {
            temperature,
            moisture,
        };
        let sample = SurfaceSample {
            height: 60.0,
            jitter: 0.0,
        };
        style.ground_colour(sample, climate, 50.0, 128.0)
    };
    let [r, g, b, _] = colour(0.9, 0.05);
    assert!(r > g && g > b, "desert is sandy: {r} {g} {b}");
    let [r, g, b, _] = colour(0.45, 0.95);
    assert!(g > r && g > b, "rainforest is green: {r} {g} {b}");
    let [r, g, b, _] = colour(0.0, 0.5);
    assert!(r == g && g == b, "snow has no hue: {r} {g} {b}");
}

#[test]
fn shore_is_the_biome_colour_turned_and_darkened() {
    let style = SurfaceStyle::default();
    let climate = ClimateSample {
        temperature: 0.6,
        moisture: 0.65,
    };
    let shade = |height| {
        let sample = SurfaceSample {
            height,
            jitter: 2.0,
        };
        style.shade(sample, climate, height, 50.0, 128.0, 0.0)
    };
    let (land, land_lightness) = shade(50.5);
    let (shore, shore_lightness) = shade(49.5);
    assert_eq!(shore.hue, land.hue + SHORE_HUE_TURN);
    assert_eq!(shore.saturation, land.saturation);
    assert!((shore_lightness - land_lightness * SHORE_LIGHTNESS).abs() < 1e-9);
}
