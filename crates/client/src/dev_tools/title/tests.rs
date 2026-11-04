use super::*;

#[test]
fn prefix_goes_before_with_a_space() {
    assert_eq!(prefixed("Dev:", "Hall of Worlds"), "Dev: Hall of Worlds");
    assert_eq!(prefixed(" Dev: ", "Hall of Worlds"), "Dev: Hall of Worlds");
}

#[test]
fn blank_prefix_leaves_the_title() {
    assert_eq!(prefixed("", "Hall of Worlds"), "Hall of Worlds");
    assert_eq!(prefixed("  ", "Hall of Worlds"), "Hall of Worlds");
}
