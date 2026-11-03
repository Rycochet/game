use std::fs;

use super::*;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| arg.to_string()).collect()
}

#[test]
fn pairs_become_a_json_object() {
    let request = Request::parse(&args(&[
        "client.screenshot",
        "path=a b.png",
        "x=3",
        "on=true",
    ]))
    .unwrap();
    assert_eq!(request.port, CLIENT_PORT);
    assert_eq!(
        request.params.as_deref(),
        Some(r#"{"path":"a b.png","x":3,"on":true}"#)
    );
    assert_eq!(
        request.body(),
        r#"{"jsonrpc":"2.0","id":1,"method":"client.screenshot","params":{"path":"a b.png","x":3,"on":true}}"#
    );
}

#[test]
fn json_params_ports_and_no_params() {
    let request = Request::parse(&args(&["--server", "server.state"])).unwrap();
    assert_eq!(request.port, CLIENT_PORT + 1);
    assert_eq!(request.params, None);
    assert_eq!(
        request.body(),
        r#"{"jsonrpc":"2.0","id":1,"method":"server.state"}"#
    );
    let request = Request::parse(&args(&["--port", "16000", "m", r#"{"a":[1]}"#])).unwrap();
    assert_eq!(request.port, 16000);
    assert_eq!(request.params.as_deref(), Some(r#"{"a":[1]}"#));
    assert!(Request::parse(&args(&["--port"])).is_err());
    assert!(Request::parse(&args(&["m", "nokey"])).is_err());
}

#[test]
fn windows_paths_are_quoted_and_read_back() {
    assert_eq!(quote(r"C:\shots\a.png"), r#""C:\\shots\\a.png""#);
    let answer = r#"{"result":{"path":"C:\\shots\\a.png","wait_for_file":"C:\\shots\\a.png"}}"#;
    assert_eq!(
        string_field(answer, "wait_for_file").as_deref(),
        Some(r"C:\shots\a.png")
    );
    assert_eq!(string_field(answer, "missing"), None);
}

#[test]
fn temporary_methods_are_found_in_any_crate_folder() {
    let root = std::env::temp_dir().join(format!("xtask-tmp-{}", std::process::id()));
    let folder = root.join("crates/client/src/somewhere");
    fs::create_dir_all(&folder).unwrap();
    fs::write(
        folder.join("kept.rs"),
        "app.add_debug_method(\"client.kept\", kept);",
    )
    .unwrap();
    assert!(temporary_methods(&root).is_empty());
    fs::write(
        folder.join("try.rs"),
        "app.add_debug_method(\"tmp.try\", try_it);",
    )
    .unwrap();
    let found = temporary_methods(&root);
    fs::remove_dir_all(&root).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("try.rs:1"), "{found:?}");
}

#[test]
fn numbers_are_read_back() {
    let answer = r#"{"result":{"frame":120,"pending": 3}}"#;
    assert_eq!(number_field(answer, "pending"), Some(3));
    assert_eq!(number_field(answer, "frame"), Some(120));
    assert_eq!(number_field(answer, "missing"), None);
}

#[test]
fn chunked_answers_are_joined() {
    assert_eq!(
        unchunk("5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n"),
        "hello world"
    );
}

#[test]
fn pretty_keeps_strings_whole() {
    assert_eq!(
        pretty(r#"{"a":[1,2],"b":"x, {y}","c":[]}"#),
        "{\n  \"a\": [\n    1,\n    2\n  ],\n  \"b\": \"x, {y}\",\n  \"c\": []\n}"
    );
}
