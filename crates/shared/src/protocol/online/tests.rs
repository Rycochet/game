use super::*;

#[test]
fn the_old_default_port_moves_to_the_new_one() {
    let mut settings = OnlineSettings {
        port: OLD_DEFAULT_PORT,
        ..Default::default()
    };
    settings.move_off_old_default();
    assert_eq!(settings.port, DEFAULT_PORT);
}

#[test]
fn a_chosen_port_is_kept() {
    let mut settings = OnlineSettings {
        port: 9000,
        ..Default::default()
    };
    settings.move_off_old_default();
    assert_eq!(settings.port, 9000);
}
