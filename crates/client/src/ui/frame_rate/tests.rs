use super::*;

#[test]
fn the_rate_is_frames_a_second_over_the_last_two_seconds() {
    let mut times = FrameTimes::default();
    assert_eq!(times.rate(), None);
    times.note(0.0);
    assert_eq!(times.rate(), None);
    // 30 frames a second for 3 seconds: only the last 2 count.
    for frame in 1..=90 {
        times.note(f64::from(frame) / 30.0);
    }
    let rate = times.rate().expect("frames noted");
    assert!((rate - 30.0).abs() < 0.01, "{rate}");
    assert!(times.0.len() <= 61);
}
