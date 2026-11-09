use super::tests::{ask, serve_securely};
use super::*;
use shared::protocol::{ServerVersion, VERSION_PATH};
use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use crate::storage::tests::TestFolder;

/// A plain web server with no game running, beating `heartbeat` only when a
/// test does, and serving the web client in `web` if given (with no history).
fn serve_plain(heartbeat: Heartbeat, web: Option<PathBuf>) -> SocketAddr {
    let web = web.map(|files| WebClient {
        files,
        history: None,
    });
    serve_web(heartbeat, web)
}

/// `serve_plain` for any web client.
pub(super) fn serve_web(heartbeat: Heartbeat, web: Option<WebClient>) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (new_connections, _accepted) = mpsc::channel();
    let sockets = Sockets {
        new_connections,
        clock: ServerClock::new(405.0),
        connections: Arc::default(),
        heartbeat,
        ping_every: PING_EVERY,
        silent_after: SILENT_AFTER,
        logins: None,
    };
    thread::spawn(move || {
        serve(
            Some(listener),
            None,
            web,
            sockets,
            Stop::Signal(Arc::default()),
        )
    });
    address
}

fn get(address: SocketAddr, path: &str) -> String {
    ask(TcpStream::connect(address).unwrap(), path)
}

#[test]
fn healthy_only_while_the_game_ticks() {
    let heartbeat = Heartbeat::default();
    let address = serve_plain(heartbeat.clone(), None);
    let answer = get(address, HEALTH_PATH);
    assert!(answer.starts_with("HTTP/1.1 503"), "{answer}");
    assert!(check_health(address).is_err());

    heartbeat.beat();
    let answer = get(address, HEALTH_PATH);
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(answer.ends_with("ok\n"), "{answer}");
    assert_eq!(check_health(address), Ok(()));
}

#[test]
fn the_health_check_needs_a_server() {
    let free = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    assert!(check_health(free).is_err());
}

#[test]
fn the_version_says_what_this_is() {
    let address = serve_plain(Heartbeat::default(), None);
    let answer = get(address, VERSION_PATH);
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    let lower = answer.to_ascii_lowercase();
    assert!(lower.contains("access-control-allow-origin: *"), "{answer}");
    assert!(lower.contains("content-type: application/json"), "{answer}");
    let body = answer.split("\r\n\r\n").nth(1).unwrap();
    assert_eq!(ServerVersion::from_json(body), Some(ServerVersion::this()));
}

#[test]
fn the_plain_port_answers_health_and_version_while_secure() {
    let (plain, _secure, _accepted, _folder) = serve_securely();
    // No game is ticking, but it is the game's answer rather than "use TLS".
    let answer = get(plain, HEALTH_PATH);
    assert!(answer.starts_with("HTTP/1.1 503"), "{answer}");
    let answer = get(plain, VERSION_PATH);
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
}

#[test]
fn the_web_client_is_served_under_web() {
    let folder = TestFolder::new();
    fs::create_dir_all(folder.0.join("assets")).unwrap();
    fs::write(folder.0.join("index.html"), "the page").unwrap();
    fs::write(folder.0.join("assets/a.txt"), "an asset").unwrap();
    let address = serve_plain(Heartbeat::default(), Some(folder.0.clone()));

    for path in ["/", "/web"] {
        let answer = get(address, path);
        assert!(answer.starts_with("HTTP/1.1 307"), "{path}: {answer}");
        assert!(
            answer.to_ascii_lowercase().contains("location: /web/\r\n"),
            "{path}: {answer}"
        );
    }
    let answer = get(address, "/web/");
    assert!(answer.ends_with("the page"), "{answer}");
    let answer = get(address, "/web/assets/a.txt");
    assert!(answer.ends_with("an asset"), "{answer}");
    let answer = get(address, "/index.html");
    assert!(answer.starts_with("HTTP/1.1 404"), "{answer}");
}

#[test]
fn the_version_gives_the_server_name() {
    let heartbeat = Heartbeat::default();
    heartbeat.call("Kim's Keep");
    let address = serve_plain(heartbeat, None);
    let answer = get(address, VERSION_PATH);
    let body = answer.split("\r\n\r\n").nth(1).unwrap();
    let version = ServerVersion::from_json(body).unwrap();
    assert_eq!(version.name.as_deref(), Some("Kim's Keep"));
}
