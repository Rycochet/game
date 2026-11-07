use super::{FLASH_SECONDS, TokenFlash};

#[test]
fn a_flash_shows_for_its_time_then_ends() {
    let mut flash = TokenFlash::default();
    let frame = FLASH_SECONDS / 3.0 + 0.001;
    assert!(flash.tick(frame));
    assert!(flash.tick(frame));
    assert!(flash.tick(frame));
    assert!(!flash.tick(frame));
}

#[test]
fn a_flash_is_seen_even_in_one_long_frame() {
    let mut flash = TokenFlash::default();
    assert!(flash.tick(1.0));
    assert!(!flash.tick(1.0));
}
