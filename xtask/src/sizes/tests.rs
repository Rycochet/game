use super::count::code_lines;
use super::list::parse;

#[test]
fn test_modules_are_not_counted() {
    let text = "fn a() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn b() {}\n}\n";
    assert_eq!(code_lines(text), 2);
}

#[test]
fn code_after_a_test_module_is_counted() {
    let text = "#[cfg(test)]\nmod tests {\n}\nfn a() {}\n";
    assert_eq!(code_lines(text), 1);
}

#[test]
fn other_test_only_lines_are_counted() {
    let text = "#[cfg(test)]\nmod tests;\n#[cfg(test)]\nfn helper() {}\n";
    assert_eq!(code_lines(text), 4);
}

#[test]
fn entries_parse_with_and_without_a_reason() {
    let (path, entry) = parse("crates/a.rs 400").unwrap();
    assert_eq!(
        (path.as_str(), entry.lines, entry.reason.as_str()),
        ("crates/a.rs", 400, "")
    );
    let (_, entry) = parse("crates/b.rs 350 # one table").unwrap();
    assert_eq!((entry.lines, entry.reason.as_str()), (350, "# one table"));
    assert!(parse("# a comment").is_none());
    assert!(parse("").is_none());
}
