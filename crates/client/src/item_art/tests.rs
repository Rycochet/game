use super::*;

#[test]
fn pictures_get_more_detail_on_a_large_interface() {
    assert_eq!(detail_for(1.0), 1);
    assert_eq!(detail_for(SHARP_UP_TO), 1);
    assert_eq!(detail_for(2.4), 2);
    assert_eq!(detail_for(9.0), MOST_DETAIL);
}
