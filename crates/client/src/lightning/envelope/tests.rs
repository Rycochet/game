use super::*;

#[test]
fn a_flash_is_full_then_over() {
    assert_eq!(strength(0.0, false), Some(1.0));
    assert_eq!(strength(0.09, false), Some(1.0));
    assert_eq!(strength(FLASH_SECONDS, false), None);
}

#[test]
fn reduce_motion_is_dimmer_and_longer() {
    assert_eq!(strength(0.0, true), Some(0.0));
    let peak = strength(GENTLE_IN, true).unwrap();
    assert!((peak - GENTLE_MOST).abs() < 1e-6);
    assert!(peak < 0.5);
    let fading = strength(GENTLE_IN + GENTLE_OUT / 2.0, true).unwrap();
    assert!(fading > 0.0 && fading < peak);
    assert!(strength(0.3, true).is_some());
    assert_eq!(strength(GENTLE_IN + GENTLE_OUT, true), None);
}

#[test]
fn a_bolt_lingers_after_the_flash_and_is_gentler_with_reduce_motion() {
    assert_eq!(bolt_alpha(0.0, false), Some(1.0));
    let fading = bolt_alpha(FLASH_SECONDS + BOLT_FADE / 2.0, false).unwrap();
    assert!(fading > 0.3 && fading < 0.7);
    assert_eq!(bolt_alpha(FLASH_SECONDS + BOLT_FADE, false), None);
    let gentle = bolt_alpha(GENTLE_IN, true).unwrap();
    assert!((gentle - GENTLE_BOLT).abs() < 1e-6);
    assert_eq!(bolt_alpha(GENTLE_IN + GENTLE_OUT, true), None);
}
