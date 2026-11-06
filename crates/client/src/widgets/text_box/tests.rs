use super::*;

#[test]
fn digit_boxes_only_keep_digits() {
    assert_eq!(allowed("a1 b2", true), "12");
    assert_eq!(allowed("a1 b2", false), "a1 b2");
    assert_eq!(allowed("a\nb", false), "ab");
}

#[test]
fn an_empty_box_keeps_a_lines_height() {
    // Bevy's own spacing, 1.2 of the font size, so a box stands as tall empty
    // as with writing in it.
    assert!((line_height(TextSize::Medium) - TextSize::Medium.pixels() * 1.2).abs() < 1e-4);
    assert!(line_height(TextSize::Small) < line_height(TextSize::Title));
}
