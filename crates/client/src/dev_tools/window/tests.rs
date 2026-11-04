use super::*;

#[test]
fn every_kind_of_start_is_read() {
    assert_eq!(
        parse_window("full-screen,1920x1080"),
        Some(WindowStart::FullScreen(1920, 1080))
    );
    assert_eq!(parse_window("full-screen"), Some(WindowStart::Borderless));
    assert_eq!(parse_window(" Maximised "), Some(WindowStart::Maximised));
    assert_eq!(parse_window("maximized"), Some(WindowStart::Maximised));
    assert_eq!(
        parse_window("1280x720"),
        Some(WindowStart::Windowed(1280, 720))
    );
}

#[test]
fn nonsense_is_refused() {
    for text in [
        "",
        "full-screen,",
        "windowed",
        "1280",
        "0x720",
        "full-screen,big",
        "1280x720,full-screen",
    ] {
        assert_eq!(parse_window(text), None, "{text}");
    }
}
