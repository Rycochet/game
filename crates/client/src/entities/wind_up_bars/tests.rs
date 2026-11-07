use super::progress;
use crate::entities::health_bars::refilled_share;

#[test]
fn only_long_wind_ups_show_and_only_until_they_land() {
    assert_eq!(progress(0.0, 0.5, 0.2), None, "an attack is quick");
    assert_eq!(progress(0.0, 4.0, 1.0), Some(0.25));
    assert_eq!(progress(0.0, 4.0, 4.0), None);
}

#[test]
fn others_health_refills_after_a_calm() {
    assert_eq!(refilled_share(500, 0.0, 5.0), 0.5);
    // Ten seconds of calm, then the whole in five minutes.
    assert!((refilled_share(500, 0.0, 10.0 + 75.0) - 0.75).abs() < 0.001);
    assert_eq!(refilled_share(500, 0.0, 1000.0), 1.0);
}
