use super::*;

#[test]
fn digit_boxes_only_keep_digits() {
    assert_eq!(allowed("a1 b2", true), "12");
    assert_eq!(allowed("a1 b2", false), "a1 b2");
    assert_eq!(allowed("a\nb", false), "ab");
}
