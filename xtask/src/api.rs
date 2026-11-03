//! `cargo xtask api`: calls a method of a running game's debug API (Bevy's
//! remote protocol: JSON-RPC over HTTP on localhost) and prints the answer.
//! Plain `std`, like the rest of xtask, so it needs nothing installed.

use std::fs;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

/// The client's port (and a standalone server's); the native client's
/// embedded server is on the next one. Matches `server::debug_api`.
const CLIENT_PORT: u16 = 15702;

/// How long to wait for a file a method says is coming (a screenshot).
const FILE_WAIT: Duration = Duration::from_secs(15);

/// How long to wait for queued fake input to be played (`client.wait` can
/// ask for many frames).
const INPUT_WAIT: Duration = Duration::from_secs(120);

pub const USAGE: &str = "\
  api     Call a running game's debug API and print the answer:
            cargo xtask api [--server | --port N] <method> [params]
          params are key=value pairs (a value that isn't JSON is a string),
          or one JSON object. --server asks the native client's embedded
          server (the port after the client's). rpc.discover lists methods.
          The game must be built with `--features debug-api` and started
          with HOW_DEBUG_API=true. See docs/context/debug-api.md.";

pub fn run(args: &[String]) -> ExitCode {
    let request = match Request::parse(args) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("{error}\n\nUsage:\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    let body = match call(request.port, &request.body()) {
        Ok(body) => body,
        Err(error) => {
            eprintln!(
                "==> Nothing answered on port {} ({error}). Is the game running, built with \
                 `--features debug-api` and started with HOW_DEBUG_API=true?",
                request.port
            );
            return ExitCode::FAILURE;
        }
    };
    println!("{}", pretty(&body));
    if let Some(file) = string_field(&body, "wait_for_file") {
        let file = PathBuf::from(file);
        if !wait_for_file(&file) {
            eprintln!("==> {} didn't appear", file.display());
            return ExitCode::FAILURE;
        }
        println!("==> Saved {}", file.display());
    }
    if body.contains("\"wait_for_input\":true") {
        if !wait_for_input(request.port) {
            eprintln!("==> The input queued wasn't all played");
            return ExitCode::FAILURE;
        }
        println!("==> Played");
    }
    if body.contains("\"error\":") {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// What to send where.
#[derive(Debug, PartialEq)]
struct Request {
    port: u16,
    method: String,
    /// JSON, or none.
    params: Option<String>,
}

impl Request {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut port = CLIENT_PORT;
        let mut rest = args.iter();
        let method = loop {
            match rest.next().map(String::as_str) {
                Some("--server") => port = CLIENT_PORT + 1,
                Some("--port") => {
                    port = rest
                        .next()
                        .and_then(|port| port.parse().ok())
                        .ok_or("--port needs a port number")?;
                }
                Some(method) => break method.to_owned(),
                None => return Err("Which method?".into()),
            }
        };
        let rest: Vec<&String> = rest.collect();
        let params = match rest.as_slice() {
            [] => None,
            [json] if json.starts_with('{') || json.starts_with('[') => Some(json.to_string()),
            pairs => {
                let fields = pairs
                    .iter()
                    .map(|pair| {
                        let (key, value) = pair
                            .split_once('=')
                            .ok_or_else(|| format!("{pair} isn't key=value"))?;
                        Ok(format!("{}:{}", quote(key), json_value(value)))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Some(format!("{{{}}}", fields.join(",")))
            }
        };
        Ok(Self {
            port,
            method,
            params,
        })
    }

    /// The JSON-RPC request.
    fn body(&self) -> String {
        let params = self
            .params
            .as_ref()
            .map(|params| format!(",\"params\":{params}"))
            .unwrap_or_default();
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":{}{params}}}",
            quote(&self.method)
        )
    }
}

/// A value from the command line: as it is if it looks like JSON (numbers,
/// `true`, `false`, `null`, objects, lists, strings), otherwise as a string.
fn json_value(value: &str) -> String {
    let looks_like_json = value.parse::<f64>().is_ok()
        || matches!(value, "true" | "false" | "null")
        || value.starts_with(['{', '[', '"']);
    if looks_like_json {
        value.to_owned()
    } else {
        quote(value)
    }
}

/// A JSON string.
fn quote(text: &str) -> String {
    let mut quoted = String::from('"');
    for c in text.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            c if (c as u32) < 0x20 => quoted.push_str(&format!("\\u{:04x}", c as u32)),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// POSTs `body` and returns the answer's body.
fn call(port: u16, body: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port))?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    write!(
        stream,
        "POST / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    let mut answer = Vec::new();
    stream.read_to_end(&mut answer)?;
    let answer = String::from_utf8_lossy(&answer);
    let (head, body) = answer
        .split_once("\r\n\r\n")
        .ok_or_else(|| io::Error::other("not an HTTP answer"))?;
    let chunked = head
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked");
    Ok(if chunked {
        unchunk(body)
    } else {
        body.to_owned()
    })
}

/// The body of a chunked HTTP answer.
fn unchunk(mut body: &str) -> String {
    let mut whole = String::new();
    while let Some((size, rest)) = body.split_once("\r\n") {
        let Ok(size) = usize::from_str_radix(size.trim(), 16) else {
            break;
        };
        if size == 0 || rest.len() < size {
            break;
        }
        whole.push_str(&rest[..size]);
        body = rest[size..].trim_start_matches("\r\n");
    }
    whole
}

/// A string field anywhere in `json`, by name (enough for `wait_for_file`).
fn string_field(json: &str, name: &str) -> Option<String> {
    let start = json.find(&format!("\"{name}\":\""))? + name.len() + 4;
    let mut value = String::new();
    let mut chars = json[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(value),
            '\\' => match chars.next()? {
                'n' => value.push('\n'),
                't' => value.push('\t'),
                other => value.push(other),
            },
            c => value.push(c),
        }
    }
    None
}

/// A whole-number field anywhere in `json`, by name.
fn number_field(json: &str, name: &str) -> Option<u64> {
    let start = json.find(&format!("\"{name}\":"))? + name.len() + 3;
    let digits: String = json[start..]
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// Asks `client.input` until nothing queued is left to play.
fn wait_for_input(port: u16) -> bool {
    let ask = Request {
        port,
        method: "client.input".into(),
        params: None,
    }
    .body();
    let deadline = Instant::now() + INPUT_WAIT;
    while Instant::now() < deadline {
        match call(port, &ask).map(|body| number_field(&body, "pending")) {
            Ok(Some(0)) => return true,
            Ok(Some(_)) => thread::sleep(Duration::from_millis(50)),
            _ => return false,
        }
    }
    false
}

/// Waits until `file` exists and has stopped growing.
fn wait_for_file(file: &Path) -> bool {
    let deadline = Instant::now() + FILE_WAIT;
    let mut last_size = None;
    while Instant::now() < deadline {
        let size = fs::metadata(file).ok().map(|metadata| metadata.len());
        if size.is_some_and(|size| size > 0) && size == last_size {
            return true;
        }
        last_size = size;
        thread::sleep(Duration::from_millis(200));
    }
    false
}

/// `json` indented, for reading.
fn pretty(json: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let newline = |out: &mut String, depth: usize| {
        out.push('\n');
        out.push_str(&"  ".repeat(depth));
    };
    let mut chars = json.trim().chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            match (escaped, c) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            // Empty ones stay on one line.
            '{' | '[' if chars.peek().is_some_and(|next| matches!(next, '}' | ']')) => {
                out.push(c);
                out.extend(chars.next());
            }
            '{' | '[' => {
                depth += 1;
                out.push(c);
                newline(&mut out, depth);
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                newline(&mut out, depth);
                out.push(c);
            }
            ',' => {
                out.push(c);
                newline(&mut out, depth);
            }
            ':' => out.push_str(": "),
            c if c.is_whitespace() => {}
            c => out.push(c),
        }
    }
    out
}

/// Debug API methods named `tmp.` in the source: for trying things on a
/// branch, so `cargo xtask test` fails until they are removed or renamed.
pub fn temporary_methods(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    look_for_temporary(&root.join("crates"), &mut found);
    found
}

fn look_for_temporary(folder: &Path, found: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            look_for_temporary(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            for (number, line) in text.lines().enumerate() {
                if line.contains("\"tmp.") {
                    found.push(format!(
                        "{}:{}: {}",
                        path.display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
