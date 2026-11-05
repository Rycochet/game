use super::*;

const LOCAL_FILE: &str = "FROM rust AS build\nRUN cargo build\nFROM debian\nCOPY --from=build a b\n# --- Same in Dockerfile and Dockerfile.github from here to the end ---\nUSER hall\nENTRYPOINT [\"x\"]\n";
const GITHUB_FILE: &str = "FROM debian\nCOPY a b\n# --- Same in Dockerfile and Dockerfile.github from here to the end ---\nUSER hall\nENTRYPOINT [\"x\"]\n";

#[test]
fn same_shared_part_passes_whatever_comes_before() {
    assert_eq!(problem(LOCAL_FILE, GITHUB_FILE), None);
}

#[test]
fn line_endings_and_trailing_blank_lines_do_not_count() {
    let windows = GITHUB_FILE.replace('\n', "\r\n") + "\r\n\r\n";
    assert_eq!(problem(LOCAL_FILE, &windows), None);
}

#[test]
fn a_changed_line_fails_naming_it() {
    let changed = GITHUB_FILE.replace("USER hall", "USER root");
    let problem = problem(LOCAL_FILE, &changed).expect("should differ");
    assert!(problem.contains("Line 1"), "{problem}");
    assert!(problem.contains("USER root"), "{problem}");
}

#[test]
fn an_extra_line_in_one_fails() {
    let longer = format!("{GITHUB_FILE}EXPOSE 9000\n");
    let problem = problem(LOCAL_FILE, &longer).expect("should differ");
    assert!(problem.contains("Line 3"), "{problem}");
    assert!(problem.contains("(the end)"), "{problem}");
}

#[test]
fn a_missing_marker_fails() {
    let unmarked = GITHUB_FILE.replace("# --- Same in", "# --- Not in");
    assert!(problem(LOCAL_FILE, &unmarked).is_some());
}

#[test]
fn the_real_dockerfiles_match() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let read = |name| fs::read_to_string(root.join(name)).unwrap();
    assert_eq!(problem(&read(LOCAL), &read(GITHUB)), None);
}
