use super::*;

#[test]
fn a_colour_from_a_slider_finds_its_place_again() {
    for kind in [BodyKind::Sun, BodyKind::Moon] {
        for step in [0.0, 0.25, 0.6, 0.85] {
            let found = nearest_along(kind, colour_along(kind, step));
            assert!((f64::from(found) - step).abs() < 0.02, "{kind:?} {step}");
        }
    }
}

#[test]
fn the_panel_is_good_markup() {
    SUNS_AND_MOONS.assert_good();
}
