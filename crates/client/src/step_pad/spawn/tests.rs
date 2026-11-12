use super::*;
use crate::step_pad::{BUTTON, CORNER_GAP, GAP, TOGGLE};

#[test]
fn the_pad_is_good_markup_with_every_button_a_finger_wide() {
    PAD.assert_good();
    let sized = format!(r#"width="{BUTTON}" height="{BUTTON}""#);
    for (_, name, _) in BUTTONS {
        let button = format!(r#"<button id="{name}" {sized}><img src="picture:pad_{name}"/>"#);
        assert!(PAD.source.contains(&button), "{name}");
    }
    // One gap in the middle, a button in size.
    let gap = format!("<div {sized}/>");
    assert_eq!(PAD.source.matches(&gap).count(), 1);
}

#[test]
fn the_pad_starts_just_above_its_toggle_button() {
    TOGGLE_BUTTON.assert_good();
    let corner =
        format!(r#"right="{CORNER_GAP}" bottom="{CORNER_GAP}" width="{TOGGLE}" height="{TOGGLE}""#);
    assert!(TOGGLE_BUTTON.source.contains(&corner));
    let above = format!(
        r#"grip right="{CORNER_GAP}" bottom="{}""#,
        CORNER_GAP + TOGGLE + GAP
    );
    assert!(PAD.source.contains(&above));
}
