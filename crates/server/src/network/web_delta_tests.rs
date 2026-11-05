//! Deltas of the web client's wasm, asked for over HTTP as a browser would.

use super::health_tests::serve_web;
use super::*;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use web_delta::{DELTAS, decode, delta_name, encode, hex, sha256};

use crate::storage::tests::TestFolder;

const WASM: &str = "hall-of-worlds-0123456789abcdef_bg.wasm";

/// Builds that share most of their bytes, as two builds of the game do.
fn builds() -> (Vec<u8>, Vec<u8>) {
    let old: Vec<u8> = (0..200_000u32)
        .map(|n| (n.wrapping_mul(2_654_435_761) >> 13) as u8)
        .collect();
    let mut new = old.clone();
    new[5000..5100].fill(7);
    (old, new)
}

/// The wasm's headers (lower case) and body, asking with these headers.
fn get_wasm(address: SocketAddr, headers: &str) -> (String, Vec<u8>) {
    let mut stream = TcpStream::connect(address).unwrap();
    write!(
        stream,
        "GET /web/{WASM} HTTP/1.1\r\nHost: localhost\r\n{headers}Connection: close\r\n\r\n"
    )
    .unwrap();
    let mut answer = Vec::new();
    stream.read_to_end(&mut answer).unwrap();
    let end = answer
        .windows(4)
        .position(|four| four == b"\r\n\r\n")
        .unwrap();
    let head = String::from_utf8_lossy(&answer[..end]).to_ascii_lowercase();
    (head, answer[end + 4..].to_vec())
}

fn asking_for_a_delta_from(old: &[u8]) -> String {
    format!(
        "Accept-Encoding: gzip, br, zstd, dcz\r\nAvailable-Dictionary: :{}:\r\n",
        STANDARD.encode(sha256(old))
    )
}

#[test]
fn a_browser_with_an_older_build_is_sent_a_delta() {
    let folder = TestFolder::new();
    let (web, history) = (folder.0.join("web"), folder.0.join("history"));
    let (old, new) = builds();
    fs::create_dir_all(&web).unwrap();
    fs::create_dir_all(&history).unwrap();
    fs::write(web.join(WASM), &new).unwrap();
    fs::write(history.join(format!("{}.wasm", hex(&sha256(&old)))), &old).unwrap();
    let client = WebClient {
        files: web,
        history: Some(history.clone()),
    };
    let address = serve_web(Heartbeat::default(), Some(client));

    // The whole file to a browser without it, telling it to keep it.
    let (head, body) = get_wasm(address, "Accept-Encoding: gzip, br, zstd, dcz\r\n");
    assert!(
        head.contains("use-as-dictionary: match=\"/web/*_bg.wasm\""),
        "{head}"
    );
    assert!(
        head.contains("vary: accept-encoding, available-dictionary"),
        "{head}"
    );
    assert_eq!(body, new);

    // Made on a thread of its own, so it may take a moment.
    let started = Instant::now();
    let (head, body) = loop {
        let (head, body) = get_wasm(address, &asking_for_a_delta_from(&old));
        if head.contains("content-encoding: dcz") || started.elapsed() > Duration::from_secs(20) {
            break (head, body);
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(head.contains("content-encoding: dcz"), "{head}");
    assert!(head.contains("use-as-dictionary"), "{head}");
    assert!(body.len() < 2_000, "{} bytes", body.len());
    assert_eq!(decode(&old, &body).unwrap(), new);
    // Kept, with the current build, for the next start.
    let kept = fs::read_dir(&history).unwrap().count();
    assert_eq!(kept, 3, "the old build, the new one and the delta");

    // Not to a browser that doesn't take dcz, nor for a build it never had.
    let (head, _) = get_wasm(address, &asking_for_a_delta_from(&old).replace(", dcz", ""));
    assert!(!head.contains("content-encoding: dcz"), "{head}");
    let (head, _) = get_wasm(address, &asking_for_a_delta_from(b"never served"));
    assert!(!head.contains("content-encoding: dcz"), "{head}");
}

#[test]
fn deltas_made_beforehand_are_sent_too() {
    let folder = TestFolder::new();
    let web = folder.0.clone();
    let (old, new) = builds();
    fs::create_dir_all(web.join(DELTAS)).unwrap();
    fs::write(web.join(WASM), &new).unwrap();
    let delta = encode(&old, &new).unwrap();
    fs::write(web.join(DELTAS).join(delta_name(&sha256(&old))), &delta).unwrap();
    let client = WebClient {
        files: web,
        history: None,
    };
    let address = serve_web(Heartbeat::default(), Some(client));

    let (head, body) = get_wasm(address, &asking_for_a_delta_from(&old));
    assert!(head.contains("content-encoding: dcz"), "{head}");
    assert_eq!(body, delta);
}
