use super::*;

#[test]
fn an_id_is_written_form_then_name() {
    assert_eq!(
        UiId::new("settings.video", "resolution").to_string(),
        "settings.video/resolution"
    );
    let bag = Id::for_test(7);
    let id = UiId::of("bag", bag, "scroll");
    assert_eq!(id.to_string(), format!("bag#{bag}/scroll"));
    let listed = UiId::new("connect", "server").at(2);
    assert_eq!(listed.to_string(), "connect/server.2");
}

#[test]
fn a_form_is_found_by_its_name_or_with_which_one() {
    let bag = Id::for_test(7);
    let id = UiId::of("bag", bag, "scroll");
    assert!(id.in_form("bag"));
    assert!(id.in_form(&format!("bag#{bag}")));
    assert!(!id.in_form("bag#nothing"));
    assert!(!UiId::new("connect", "address").in_form("con"));
}

#[test]
fn ids_given_twice_are_found_once_each() {
    let ids = [
        UiId::new("a", "x"),
        UiId::new("a", "y"),
        UiId::new("a", "x"),
        UiId::new("a", "x"),
        UiId::new("b", "x"),
    ];
    assert_eq!(duplicate_ids(&ids), ["a/x"]);
}
