use super::super::RAISED_BACKGROUND;
use super::super::theme::{ERROR_EDGE, HOVER_EDGE};
use super::super::{ButtonState, INK};
use super::show::text_box_edge;
use super::*;

#[test]
fn only_usable_buttons_answer_the_pointer() {
    let plain = ButtonStyle::Plain;
    let away = look(ButtonState::Normal, plain, Touch::Away);
    assert_ne!(away.0, look(ButtonState::Normal, plain, Touch::Hovered).0);
    assert_ne!(away.0, look(ButtonState::Normal, plain, Touch::Pressed).0);
    let disabled = look(ButtonState::Disabled, plain, Touch::Away);
    assert_eq!(
        disabled.0,
        look(ButtonState::Disabled, plain, Touch::Hovered).0
    );
}

#[test]
fn a_quiet_button_has_a_box_only_under_the_pointer() {
    let quiet = ButtonStyle::Quiet;
    assert_eq!(look(ButtonState::Normal, quiet, Touch::Away).0, Color::NONE);
    assert_ne!(
        look(ButtonState::Normal, quiet, Touch::Hovered).0,
        Color::NONE
    );
    assert_eq!(
        look(ButtonState::Normal, ButtonStyle::Plain, Touch::Away).0,
        RAISED_BACKGROUND
    );
}

#[test]
fn a_chosen_folder_tab_runs_into_the_panel() {
    let chosen = look(ButtonState::Selected, ButtonStyle::Folder, Touch::Away);
    assert_eq!(chosen.1.bottom, chosen.0);
    assert_eq!(chosen.2, INK);
}

#[test]
fn an_error_shows_whoever_has_the_box() {
    assert_eq!(
        text_box_edge(true, true, true),
        BorderColor::from(ERROR_EDGE)
    );
    assert_eq!(text_box_edge(false, true, true), BorderColor::from(INK));
    assert_eq!(
        text_box_edge(false, false, true),
        BorderColor::from(HOVER_EDGE)
    );
}

#[test]
fn a_ripple_spreads_as_it_fades() {
    let (small, strong) = ripple_shape(0.0, 100.0);
    let (big, faint) = ripple_shape(RIPPLE_SECONDS / 2.0, 100.0);
    assert!(small < big && faint < strong);
    assert_eq!(ripple_shape(RIPPLE_SECONDS * 2.0, 100.0), (100.0, 0.0));
}
