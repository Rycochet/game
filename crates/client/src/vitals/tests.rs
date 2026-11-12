use super::{BOTTOM_GAP, VITALS};

#[test]
fn the_circle_is_good_markup_with_its_gap_under_it() {
    VITALS.assert_good();
    let bottom = format!(r#"bottom="{BOTTOM_GAP}""#);
    assert!(VITALS.source.contains(&bottom));
}
