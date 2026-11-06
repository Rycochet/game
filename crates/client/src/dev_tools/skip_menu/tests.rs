use super::*;

fn names(list: &[&str]) -> impl Iterator<Item = String> {
    list.iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>()
        .into_iter()
}

#[test]
fn starting_variables_skip_the_menu() {
    assert!(sets_up_start(names(&["PATH", "HOW_DEBUG_POS"])));
    assert!(sets_up_start(names(&["HOW_DEBUG_SCREENSHOT"])));
}

#[test]
fn others_leave_it() {
    assert!(!sets_up_start(names(&[])));
    assert!(!sets_up_start(names(&[
        "HOW_DATA",
        "HOW_SERVER",
        "HOW_DEBUG_API",
        "HOW_DEBUG_TITLE"
    ])));
    assert!(!sets_up_start(names(&["HOW_DEBUG_SKIP_MENU"])));
    assert!(!sets_up_start(names(&["HOW_DEBUG_SKIP_CREATION"])));
}
