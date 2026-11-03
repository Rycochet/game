use super::*;

#[test]
fn every_entry_is_its_own_size() {
    assert_eq!(size_options().len(), INTERFACE_PERCENTS.len() + 1);
    for index in 0..size_options().len() {
        assert_eq!(size_index(size_at(index)), index);
    }
    assert_eq!(size_options()[size_index(InterfaceSize::Auto)], "Auto");
    assert_eq!(
        size_options()[size_index(InterfaceSize::Percent(150))],
        "150%"
    );
}

#[test]
fn a_saved_size_not_offered_shows_the_next_one_up() {
    assert_eq!(
        size_at(size_index(InterfaceSize::Percent(110))),
        InterfaceSize::Percent(125)
    );
    assert_eq!(
        size_at(size_index(InterfaceSize::Percent(900))),
        InterfaceSize::Percent(300)
    );
}
