use super::*;

#[test]
fn the_variable_turns_it_on_or_names_a_port() {
    assert_eq!(port_from(None), None);
    assert_eq!(port_from(Some("true")), Some(DEFAULT_PORT));
    assert_eq!(port_from(Some("YES")), Some(DEFAULT_PORT));
    assert_eq!(port_from(Some("1")), Some(DEFAULT_PORT));
    assert_eq!(port_from(Some("15800")), Some(15800));
    assert_eq!(port_from(Some("false")), None);
    assert_eq!(port_from(Some("0")), None);
    assert_eq!(port_from(Some("nonsense")), None);
}
