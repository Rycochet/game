use super::*;

fn resolution(width: u32, height: u32, hertz: u32) -> Option<Resolution> {
    Some(Resolution {
        width,
        height,
        refresh_millihertz: hertz * 1000,
    })
}

#[test]
fn a_resolution_is_found_by_itself_then_its_size() {
    let entries = [None, resolution(1920, 1080, 144), resolution(1280, 720, 60)];
    assert_eq!(resolution_index(&entries, None), 0);
    assert_eq!(resolution_index(&entries, resolution(1280, 720, 60)), 2);
    // `HOW_DEBUG_WINDOW` gives no refresh.
    assert_eq!(resolution_index(&entries, resolution(1920, 1080, 0)), 1);
    assert_eq!(resolution_index(&entries, resolution(800, 600, 60)), 0);
}

#[test]
fn without_a_monitor_there_is_only_the_desktop() {
    let (entries, labels) = resolution_entries(None);
    assert_eq!(entries, vec![None]);
    assert_eq!(labels, vec!["Desktop".to_string()]);
}
