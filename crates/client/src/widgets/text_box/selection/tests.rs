use super::*;

#[test]
fn the_band_is_paler_without_the_keyboard_but_still_seen() {
    let (with, without) = (band_colour(true).alpha(), band_colour(false).alpha());
    assert!(without < with);
    assert!(without > 0.1);
}
