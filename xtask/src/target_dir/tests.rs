use super::from_metadata;
use std::path::PathBuf;

#[test]
fn a_windows_path_loses_its_doubled_backslashes() {
    let json = r#"{"packages":[],"target_directory":"C:\\Projects\\game\\target","version":1}"#;
    assert_eq!(
        from_metadata(json),
        Some(PathBuf::from(r"C:\Projects\game\target"))
    );
}

#[test]
fn a_path_set_elsewhere_is_read_as_given() {
    let json = r#"{"target_directory":"D:/.cargo-target","workspace_root":"/x"}"#;
    assert_eq!(from_metadata(json), Some(PathBuf::from("D:/.cargo-target")));
}

#[test]
fn no_target_directory_gives_nothing() {
    assert_eq!(from_metadata(r#"{"packages":[]}"#), None);
}
