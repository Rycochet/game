use super::*;
use crate::step_pad::BUTTON;

#[test]
fn the_pad_is_good_markup_with_every_button_a_finger_wide() {
    PAD.assert_good();
    let sized = format!(r#"width="{BUTTON}" height="{BUTTON}""#);
    for (_, name, _) in BUTTONS {
        let button = format!(r#"<button id="{name}" {sized}><img src="picture:pad_{name}"/>"#);
        assert!(PAD.source.contains(&button), "{name}");
    }
    // Three gaps, each a button in size.
    let gap = format!("<div {sized}/>");
    assert_eq!(PAD.source.matches(&gap).count(), 3);
}
