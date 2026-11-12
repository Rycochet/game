use super::*;

#[test]
fn the_status_text_is_good_markup_starting_clear_of_the_warning_button() {
    STATUS.assert_good();
    let left = format!(r#"left="{STATUS_LEFT}""#);
    assert!(STATUS.source.contains(&left));
    // (Checked when compiling: it only moves left when it is alone.)
    const { assert!(STATUS_LEFT_ALONE < STATUS_LEFT) };
}
