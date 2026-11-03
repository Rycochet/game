use super::*;

#[test]
fn starting_lasts_until_the_screen_has_been_drawn() {
    assert_eq!(
        next_step(LoadingStep::Starting, 0, 0),
        LoadingStep::Starting
    );
    assert_eq!(
        next_step(LoadingStep::Starting, FRAMES_TO_DRAW, 0),
        LoadingStep::Connecting
    );
}

#[test]
fn ready_only_once_settled_and_gone_when_the_connection_is() {
    let many = 1000;
    assert_eq!(
        next_step(LoadingStep::Connecting, many, FRAMES_TO_SETTLE),
        LoadingStep::Connecting
    );
    assert_eq!(
        next_step(LoadingStep::Connecting, many, FRAMES_TO_SETTLE + 1),
        LoadingStep::Ready
    );
    assert_eq!(
        next_step(LoadingStep::Ready, many, 0),
        LoadingStep::Connecting
    );
}

#[test]
fn title_stays_in_its_share_of_any_window() {
    for window in [Vec2::new(1280.0, 720.0), Vec2::new(375.0, 812.0)] {
        for (at_start, share) in [
            (false, TITLE_HEIGHT_SHARE),
            (true, START_TITLE_HEIGHT_SHARE),
        ] {
            let size = title_font_size(window, at_start);
            assert!(size * TITLE_EMS_WIDE <= window.x * TITLE_WIDTH_SHARE + 0.01);
            assert!(size * TITLE_EMS_HIGH <= window.y * share + 0.01);
        }
    }
}

#[test]
fn a_refusal_is_said_on_the_bar() {
    let refused = ServerStatus::Refused("too old".into());
    let (words, _) = bar_shows(LoadingStep::Connecting, &refused);
    assert_eq!(words, "too old");
}
