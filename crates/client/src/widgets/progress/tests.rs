use super::*;

#[test]
fn a_bar_never_goes_past_its_ends() {
    assert_eq!(width_of(0.5), percent(50));
    assert_eq!(width_of(-1.0), percent(0));
    assert_eq!(width_of(2.0), percent(100));
}
