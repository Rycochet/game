use super::*;

#[test]
fn only_tools_keep_drawing() {
    assert!(!keeps_drawing(false, false, true));
    assert!(keeps_drawing(true, false, true));
    assert!(keeps_drawing(false, true, true));
}

#[test]
fn the_variable_turns_it_off() {
    assert!(!keeps_drawing(true, true, false));
}
